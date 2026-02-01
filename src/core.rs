//! Core database operations for Capbit - Normalized Schema
//!
//! Uses interleaved bits for dual O(log n) lookup via thread-local mode switching.
//! Entities and grants reference by numeric ID, enabling O(1) rename.

use std::path::Path;
use std::sync::OnceLock;

use heed::types::{Bytes, Str, U64};
use heed::{Database, Env, EnvOpenOptions, RoTxn, RwTxn};
use serde::{Deserialize, Serialize};

use crate::keys::{
    set_mode_first, set_mode_second,
    entity_key, grant_key, capability_key, inheritance_key, cap_label_key,
    parse_entity_key, parse_grant_key, parse_capability_key, parse_inheritance_key, parse_cap_label_key,
    entity_type_prefix, grant_seeker_prefix, grant_seeker_scope_prefix,
    inheritance_seeker_prefix, inheritance_seeker_scope_prefix,
};

// ============================================================================
// Database Setup - Normalized Schema
// ============================================================================

static ENV: OnceLock<Env> = OnceLock::new();

use std::sync::Mutex;
static TEST_LOCK: Mutex<()> = Mutex::new(());

/// Normalized database structure
/// - entities: [type_id:4][interleaved(entity_id, name_hash):8] → name:String
/// - grants: [interleaved(seeker_id, scope_id):8][role_id:4] → cap_mask:u64
/// - roles: [role_id:4] → label:String
/// - capabilities: [scope_type:4][scope_id:4][role_id:4] → cap_mask:u64
/// - inheritance: [interleaved(seeker_id, scope_id):8][source_id:4] → epoch:u64
/// - cap_labels: [scope_type:4][scope_id:4][bit:4] → label:String
/// - types: type_name:String → type_id:u32
/// - meta: key:String → value:String
pub(crate) struct Databases {
    // Normalized tables with fixed-width keys
    pub entities: Database<Bytes, Str>,
    pub grants: Database<Bytes, U64<byteorder::BigEndian>>,
    pub roles: Database<Bytes, Str>,  // [role_id:4] → label
    pub capabilities: Database<Bytes, U64<byteorder::BigEndian>>,
    pub inheritance: Database<Bytes, U64<byteorder::BigEndian>>,
    pub cap_labels: Database<Bytes, Str>,

    // Lookup tables (string → ID)
    pub types: Database<Str, U64<byteorder::BigEndian>>,  // type_name → type_id
    pub types_rev: Database<Bytes, Str>,  // [type_id:4] → type_name
    pub roles_by_name: Database<Str, U64<byteorder::BigEndian>>,  // role_label → role_id

    // Metadata and sessions (keep string keys for simplicity)
    pub meta: Database<Str, Str>,
    pub sessions: Database<Str, Str>,
    pub sessions_by_entity: Database<Str, Str>,
    pub credentials: Database<Str, Str>,
}

static DBS: OnceLock<Databases> = OnceLock::new();

// ============================================================================
// Error Handling
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapbitError {
    pub message: String,
}

impl std::fmt::Display for CapbitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CapbitError {}

pub type Result<T> = std::result::Result<T, CapbitError>;

fn err<E: std::error::Error>(e: E) -> CapbitError {
    CapbitError { message: e.to_string() }
}

// ============================================================================
// Transaction Helpers
// ============================================================================

fn get_env() -> Result<&'static Env> {
    ENV.get().ok_or_else(|| CapbitError { message: "Database not initialized".into() })
}

fn get_dbs() -> Result<&'static Databases> {
    DBS.get().ok_or_else(|| CapbitError { message: "Database not initialized".into() })
}

pub(crate) fn with_read_txn<T, F>(f: F) -> Result<T>
where
    F: FnOnce(&RoTxn, &Databases) -> Result<T>,
{
    let env = get_env()?;
    let dbs = get_dbs()?;
    let txn = env.read_txn().map_err(err)?;
    f(&txn, dbs)
}

pub(crate) fn with_write_txn<T, F>(f: F) -> Result<T>
where
    F: FnOnce(&mut RwTxn, &Databases) -> Result<T>,
{
    let env = get_env()?;
    let dbs = get_dbs()?;
    let mut txn = env.write_txn().map_err(err)?;
    let result = f(&mut txn, dbs)?;
    txn.commit().map_err(err)?;
    Ok(result)
}

pub(crate) fn current_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

// ============================================================================
// Initialization
// ============================================================================

pub fn init(db_path: &str) -> Result<()> {
    if ENV.get().is_some() {
        return Ok(());
    }

    let path = Path::new(db_path);
    std::fs::create_dir_all(path).map_err(err)?;

    let env = unsafe {
        EnvOpenOptions::new()
            .map_size(10 * 1024 * 1024 * 1024)
            .max_dbs(20)
            .open(path)
            .map_err(err)?
    };

    let mut wtxn = env.write_txn().map_err(err)?;

    let dbs = Databases {
        // Normalized tables
        entities: env.create_database(&mut wtxn, Some("entities_v2")).map_err(err)?,
        grants: env.create_database(&mut wtxn, Some("grants_v2")).map_err(err)?,
        roles: env.create_database(&mut wtxn, Some("roles_v2")).map_err(err)?,
        capabilities: env.create_database(&mut wtxn, Some("capabilities_v2")).map_err(err)?,
        inheritance: env.create_database(&mut wtxn, Some("inheritance_v2")).map_err(err)?,
        cap_labels: env.create_database(&mut wtxn, Some("cap_labels_v2")).map_err(err)?,

        // Lookup tables
        types: env.create_database(&mut wtxn, Some("types_v2")).map_err(err)?,
        types_rev: env.create_database(&mut wtxn, Some("types_rev_v2")).map_err(err)?,
        roles_by_name: env.create_database(&mut wtxn, Some("roles_by_name_v2")).map_err(err)?,

        // Session/auth tables (unchanged)
        meta: env.create_database(&mut wtxn, Some("meta")).map_err(err)?,
        sessions: env.create_database(&mut wtxn, Some("sessions")).map_err(err)?,
        sessions_by_entity: env.create_database(&mut wtxn, Some("sessions_by_entity")).map_err(err)?,
        credentials: env.create_database(&mut wtxn, Some("credentials")).map_err(err)?,
    };

    wtxn.commit().map_err(err)?;
    let _ = ENV.set(env);
    let _ = DBS.set(dbs);
    Ok(())
}

pub fn close() {}

/// Clear all data from all databases. Used for testing.
pub fn clear_all() -> Result<()> {
    with_write_txn(|txn, dbs| {
        dbs.entities.clear(txn).map_err(err)?;
        dbs.grants.clear(txn).map_err(err)?;
        dbs.roles.clear(txn).map_err(err)?;
        dbs.capabilities.clear(txn).map_err(err)?;
        dbs.inheritance.clear(txn).map_err(err)?;
        dbs.cap_labels.clear(txn).map_err(err)?;
        dbs.types.clear(txn).map_err(err)?;
        dbs.types_rev.clear(txn).map_err(err)?;
        dbs.roles_by_name.clear(txn).map_err(err)?;
        dbs.meta.clear(txn).map_err(err)?;
        dbs.sessions.clear(txn).map_err(err)?;
        dbs.sessions_by_entity.clear(txn).map_err(err)?;
        dbs.credentials.clear(txn).map_err(err)?;
        Ok(())
    })
}

/// Get the test lock for serializing tests
pub fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    match TEST_LOCK.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

// ============================================================================
// ID Generation
// ============================================================================

fn next_id(txn: &mut RwTxn, dbs: &Databases, key: &str) -> Result<u32> {
    let current = dbs.meta.get(txn, key).map_err(err)?
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    let next = current + 1;
    dbs.meta.put(txn, key, &next.to_string()).map_err(err)?;
    Ok(next)
}

fn next_type_id(txn: &mut RwTxn, dbs: &Databases) -> Result<u32> {
    next_id(txn, dbs, "next_type_id")
}

fn next_entity_id(txn: &mut RwTxn, dbs: &Databases, type_id: u32) -> Result<u32> {
    let key = format!("next_entity_id:{}", type_id);
    next_id(txn, dbs, &key)
}

fn next_role_id(txn: &mut RwTxn, dbs: &Databases) -> Result<u32> {
    next_id(txn, dbs, "next_role_id")
}

// ============================================================================
// Type Operations
// ============================================================================

/// Resolve type name to type_id
pub fn resolve_type(txn: &RoTxn, dbs: &Databases, type_name: &str) -> Result<u32> {
    dbs.types.get(txn, type_name).map_err(err)?
        .map(|id| id as u32)
        .ok_or_else(|| CapbitError { message: format!("Type '{}' not found", type_name) })
}

/// Resolve type_id to type name
pub fn resolve_type_name(txn: &RoTxn, dbs: &Databases, type_id: u32) -> Result<String> {
    let key = type_id.to_be_bytes();
    dbs.types_rev.get(txn, &key).map_err(err)?
        .map(|s| s.to_string())
        .ok_or_else(|| CapbitError { message: format!("Type ID {} not found", type_id) })
}

pub(crate) fn create_type_in(txn: &mut RwTxn, dbs: &Databases, type_name: &str) -> Result<u32> {
    // Check if already exists
    if dbs.types.get(txn, type_name).map_err(err)?.is_some() {
        return Err(CapbitError { message: format!("Type '{}' already exists", type_name) });
    }

    let type_id = next_type_id(txn, dbs)?;
    dbs.types.put(txn, type_name, &(type_id as u64)).map_err(err)?;
    dbs.types_rev.put(txn, &type_id.to_be_bytes(), type_name).map_err(err)?;
    Ok(type_id)
}

pub(crate) fn delete_type_in(txn: &mut RwTxn, dbs: &Databases, type_name: &str) -> Result<bool> {
    let type_id = match dbs.types.get(txn, type_name).map_err(err)? {
        Some(id) => id as u32,
        None => return Ok(false),
    };

    dbs.types.delete(txn, type_name).map_err(err)?;
    dbs.types_rev.delete(txn, &type_id.to_be_bytes()).map_err(err)?;

    // Also delete the _type:X entity if it exists
    let type_entity = format!("_type:{}", type_name);
    let _ = delete_entity_by_label_in(txn, dbs, &type_entity);

    Ok(true)
}

// ============================================================================
// Role Operations
// ============================================================================

/// Resolve role label to role_id, creating if needed
fn resolve_or_create_role(txn: &mut RwTxn, dbs: &Databases, label: &str) -> Result<u32> {
    if let Some(id) = dbs.roles_by_name.get(txn, label).map_err(err)? {
        return Ok(id as u32);
    }

    let role_id = next_role_id(txn, dbs)?;
    let key = role_id.to_be_bytes();
    dbs.roles.put(txn, &key[..], label).map_err(err)?;
    dbs.roles_by_name.put(txn, label, &(role_id as u64)).map_err(err)?;
    Ok(role_id)
}

/// Resolve role_id to label
pub fn resolve_role_label(txn: &RoTxn, dbs: &Databases, role_id: u32) -> Result<String> {
    let key = role_id.to_be_bytes();
    dbs.roles.get(txn, &key[..]).map_err(err)?
        .map(|s| s.to_string())
        .ok_or_else(|| CapbitError { message: format!("Role ID {} not found", role_id) })
}

/// Resolve role label to role_id
pub fn resolve_role(txn: &RoTxn, dbs: &Databases, label: &str) -> Result<u32> {
    dbs.roles_by_name.get(txn, label).map_err(err)?
        .map(|id| id as u32)
        .ok_or_else(|| CapbitError { message: format!("Role '{}' not found", label) })
}

// ============================================================================
// Entity Operations
// ============================================================================

/// Resolve entity label (type:name) to (type_id, entity_id)
pub fn resolve_entity(txn: &RoTxn, dbs: &Databases, label: &str) -> Result<(u32, u32)> {
    let (type_name, name) = parse_entity_id(label)?;
    let type_id = resolve_type(txn, dbs, type_name)?;

    // Search by name using mode=second
    set_mode_second();
    let prefix = entity_type_prefix(type_id);

    for item in dbs.entities.prefix_iter(txn, &prefix).map_err(err)? {
        let (key, stored_name) = item.map_err(err)?;
        if stored_name == name {
            let (_, entity_id, _) = parse_entity_key(key).ok_or_else(||
                CapbitError { message: "Invalid entity key".into() })?;
            return Ok((type_id, entity_id));
        }
    }

    Err(CapbitError { message: format!("Entity '{}' not found", label) })
}

/// Resolve (type_id, entity_id) to entity label
pub fn resolve_entity_label(txn: &RoTxn, dbs: &Databases, type_id: u32, entity_id: u32) -> Result<String> {
    let type_name = resolve_type_name(txn, dbs, type_id)?;

    // Search by ID using mode=first
    set_mode_first();
    let prefix = entity_type_prefix(type_id);

    for item in dbs.entities.prefix_iter(txn, &prefix).map_err(err)? {
        let (key, name) = item.map_err(err)?;
        let (_, eid, _) = parse_entity_key(key).ok_or_else(||
            CapbitError { message: "Invalid entity key".into() })?;
        if eid == entity_id {
            return Ok(format!("{}:{}", type_name, name));
        }
    }

    Err(CapbitError { message: format!("Entity {}:{} not found", type_id, entity_id) })
}

/// Parse entity ID string "type:id" into (type, id)
pub fn parse_entity_id(entity_id: &str) -> Result<(&str, &str)> {
    entity_id.split_once(':').ok_or_else(|| CapbitError {
        message: format!("Invalid entity ID '{}': must be 'type:id' format", entity_id),
    })
}

pub(crate) fn create_entity_in(txn: &mut RwTxn, dbs: &Databases, entity_label: &str) -> Result<(u32, u32)> {
    let (type_name, name) = parse_entity_id(entity_label)?;

    // Get or create type_id (allow internal types like _type)
    let type_id = if type_name.starts_with('_') {
        // Internal types - create if not exists
        match dbs.types.get(txn, type_name).map_err(err)? {
            Some(id) => id as u32,
            None => create_type_in(txn, dbs, type_name)?,
        }
    } else {
        // Regular types - must exist
        resolve_type(txn, dbs, type_name)?
    };

    // Check if entity already exists (by name)
    set_mode_second();
    let prefix = entity_type_prefix(type_id);
    for item in dbs.entities.prefix_iter(txn, &prefix).map_err(err)? {
        let (_, stored_name) = item.map_err(err)?;
        if stored_name == name {
            return Err(CapbitError { message: format!("Entity '{}' already exists", entity_label) });
        }
    }

    // Create entity
    let entity_id = next_entity_id(txn, dbs, type_id)?;
    let key = entity_key(type_id, entity_id, name);
    dbs.entities.put(txn, &key, name).map_err(err)?;

    Ok((type_id, entity_id))
}

pub(crate) fn delete_entity_in(txn: &mut RwTxn, dbs: &Databases, type_id: u32, entity_id: u32) -> Result<bool> {
    // Find entity by ID
    set_mode_first();
    let prefix = entity_type_prefix(type_id);

    let mut found_key = None;
    for item in dbs.entities.prefix_iter(txn, &prefix).map_err(err)? {
        let (key, _) = item.map_err(err)?;
        let (_, eid, _) = parse_entity_key(key).ok_or_else(||
            CapbitError { message: "Invalid entity key".into() })?;
        if eid == entity_id {
            found_key = Some(key.to_vec());
            break;
        }
    }

    let key = match found_key {
        Some(k) => k,
        None => return Ok(false),
    };

    // Delete entity
    dbs.entities.delete(txn, &key).map_err(err)?;

    // Cascade delete grants where this entity is seeker
    let grants_to_delete: Vec<_> = dbs.grants.iter(txn).map_err(err)?
        .filter_map(|r| r.ok())
        .filter_map(|(k, _)| {
            let (seeker_type, seeker_id, _, _, _) = parse_grant_key(k)?;
            if seeker_type == type_id && seeker_id == entity_id { Some(k.to_vec()) } else { None }
        })
        .collect();
    for k in grants_to_delete {
        dbs.grants.delete(txn, &k).map_err(err)?;
    }

    // Cascade delete grants where this entity is scope
    let grants_to_delete: Vec<_> = dbs.grants.iter(txn).map_err(err)?
        .filter_map(|r| r.ok())
        .filter_map(|(k, _)| {
            let (_, _, scope_type, scope_id, _) = parse_grant_key(k)?;
            if scope_type == type_id && scope_id == entity_id { Some(k.to_vec()) } else { None }
        })
        .collect();
    for k in grants_to_delete {
        dbs.grants.delete(txn, &k).map_err(err)?;
    }

    // Cascade delete inheritance records
    let inherit_to_delete: Vec<_> = dbs.inheritance.iter(txn).map_err(err)?
        .filter_map(|r| r.ok())
        .filter_map(|(k, _)| {
            let (skt, sk, sct, sc, srct, src) = parse_inheritance_key(k)?;
            // Match if any entity matches (type + id)
            if (skt == type_id && sk == entity_id) || (sct == type_id && sc == entity_id) || (srct == type_id && src == entity_id) {
                Some(k.to_vec())
            } else {
                None
            }
        })
        .collect();
    for k in inherit_to_delete {
        dbs.inheritance.delete(txn, &k).map_err(err)?;
    }

    // Cascade delete capabilities defined on this entity
    let caps_to_delete: Vec<_> = dbs.capabilities.iter(txn).map_err(err)?
        .filter_map(|r| r.ok())
        .filter_map(|(k, _)| {
            let (scope_type, scope_id, _) = parse_capability_key(k)?;
            if scope_type == type_id && scope_id == entity_id {
                Some(k.to_vec())
            } else {
                None
            }
        })
        .collect();
    for k in caps_to_delete {
        dbs.capabilities.delete(txn, &k).map_err(err)?;
    }

    Ok(true)
}

/// Delete entity by label (for API compatibility)
pub(crate) fn delete_entity_by_label_in(txn: &mut RwTxn, dbs: &Databases, label: &str) -> Result<bool> {
    let (type_id, entity_id) = resolve_entity(txn, dbs, label)?;
    delete_entity_in(txn, dbs, type_id, entity_id)
}

pub(crate) fn entity_exists_in(txn: &RoTxn, dbs: &Databases, label: &str) -> Result<bool> {
    match resolve_entity(txn, dbs, label) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Rename entity (O(1) - grants unchanged!)
pub fn rename_entity_in(txn: &mut RwTxn, dbs: &Databases, type_id: u32, entity_id: u32, new_name: &str) -> Result<()> {
    // Find old key by ID
    set_mode_first();
    let prefix = entity_type_prefix(type_id);

    let mut old_key = None;
    for item in dbs.entities.prefix_iter(txn, &prefix).map_err(err)? {
        let (key, _) = item.map_err(err)?;
        let (_, eid, _) = parse_entity_key(key).ok_or_else(||
            CapbitError { message: "Invalid entity key".into() })?;
        if eid == entity_id {
            old_key = Some(key.to_vec());
            break;
        }
    }

    let old_key = old_key.ok_or_else(||
        CapbitError { message: format!("Entity {}:{} not found", type_id, entity_id) })?;

    // Check new name doesn't conflict
    set_mode_second();
    for item in dbs.entities.prefix_iter(txn, &prefix).map_err(err)? {
        let (key, stored_name) = item.map_err(err)?;
        if stored_name == new_name && key != old_key.as_slice() {
            return Err(CapbitError { message: format!("Entity name '{}' already exists", new_name) });
        }
    }

    // Delete old, insert new
    dbs.entities.delete(txn, &old_key).map_err(err)?;
    let new_key = entity_key(type_id, entity_id, new_name);
    dbs.entities.put(txn, &new_key, new_name).map_err(err)?;

    // Grants use entity_id - UNCHANGED!
    Ok(())
}

// ============================================================================
// Grant/Relationship Operations
// ============================================================================

pub(crate) fn set_relationship_in(txn: &mut RwTxn, dbs: &Databases, subject: &str, rel_type: &str, object: &str) -> Result<u64> {
    let (seeker_type, seeker_id) = resolve_entity(txn, dbs, subject)?;
    let (scope_type, scope_id) = resolve_entity(txn, dbs, object)?;
    let role_id = resolve_or_create_role(txn, dbs, rel_type)?;

    // Store epoch as value - capability is looked up dynamically via role_id in key
    let key = grant_key(seeker_type, seeker_id, scope_type, scope_id, role_id);
    let epoch = current_epoch();
    dbs.grants.put(txn, &key, &epoch).map_err(err)?;
    Ok(epoch)
}

pub(crate) fn delete_relationship_in(txn: &mut RwTxn, dbs: &Databases, subject: &str, rel_type: &str, object: &str) -> Result<bool> {
    let (seeker_type, seeker_id) = resolve_entity(txn, dbs, subject)?;
    let (scope_type, scope_id) = resolve_entity(txn, dbs, object)?;
    let role_id = resolve_role(txn, dbs, rel_type)?;

    let key = grant_key(seeker_type, seeker_id, scope_type, scope_id, role_id);
    Ok(dbs.grants.delete(txn, &key).map_err(err)?)
}

fn get_capability_mask_in(txn: &RoTxn, dbs: &Databases, scope_type: u32, scope_id: u32, role_id: u32) -> Result<u64> {
    let key = capability_key(scope_type, scope_id, role_id);
    Ok(dbs.capabilities.get(txn, &key).map_err(err)?.unwrap_or(0))
}

// ============================================================================
// Capability Operations
// ============================================================================

pub(crate) fn set_capability_in(txn: &mut RwTxn, dbs: &Databases, entity: &str, rel_type: &str, cap_mask: u64) -> Result<u64> {
    let (scope_type, scope_id) = resolve_entity(txn, dbs, entity)?;
    let role_id = resolve_or_create_role(txn, dbs, rel_type)?;

    let key = capability_key(scope_type, scope_id, role_id);
    dbs.capabilities.put(txn, &key, &cap_mask).map_err(err)?;
    Ok(current_epoch())
}

pub(crate) fn delete_capability_in(txn: &mut RwTxn, dbs: &Databases, entity: &str, rel_type: &str) -> Result<bool> {
    let (scope_type, scope_id) = resolve_entity(txn, dbs, entity)?;
    let role_id = resolve_role(txn, dbs, rel_type)?;

    let key = capability_key(scope_type, scope_id, role_id);
    Ok(dbs.capabilities.delete(txn, &key).map_err(err)?)
}

// ============================================================================
// Inheritance Operations
// ============================================================================

pub(crate) fn set_inheritance_in(txn: &mut RwTxn, dbs: &Databases, subject: &str, object: &str, source: &str) -> Result<u64> {
    let (seeker_type, seeker_id) = resolve_entity(txn, dbs, subject)?;
    let (scope_type, scope_id) = resolve_entity(txn, dbs, object)?;
    let (source_type, source_id) = resolve_entity(txn, dbs, source)?;

    let key = inheritance_key(seeker_type, seeker_id, scope_type, scope_id, source_type, source_id);
    let epoch = current_epoch();
    dbs.inheritance.put(txn, &key, &epoch).map_err(err)?;
    Ok(epoch)
}

pub(crate) fn delete_inheritance_in(txn: &mut RwTxn, dbs: &Databases, subject: &str, object: &str, source: &str) -> Result<bool> {
    let (seeker_type, seeker_id) = resolve_entity(txn, dbs, subject)?;
    let (scope_type, scope_id) = resolve_entity(txn, dbs, object)?;
    let (source_type, source_id) = resolve_entity(txn, dbs, source)?;

    let key = inheritance_key(seeker_type, seeker_id, scope_type, scope_id, source_type, source_id);
    Ok(dbs.inheritance.delete(txn, &key).map_err(err)?)
}

// ============================================================================
// Cap Label Operations
// ============================================================================

pub(crate) fn set_cap_label_in(txn: &mut RwTxn, dbs: &Databases, entity: &str, cap_bit: u64, label: &str) -> Result<()> {
    let (scope_type, scope_id) = resolve_entity(txn, dbs, entity)?;
    let key = cap_label_key(scope_type, scope_id, cap_bit as u32);
    dbs.cap_labels.put(txn, &key, label).map_err(err)?;
    Ok(())
}

pub(crate) fn delete_cap_label_in(txn: &mut RwTxn, dbs: &Databases, entity: &str, cap_bit: u64) -> Result<bool> {
    let (scope_type, scope_id) = resolve_entity(txn, dbs, entity)?;
    let key = cap_label_key(scope_type, scope_id, cap_bit as u32);
    Ok(dbs.cap_labels.delete(txn, &key).map_err(err)?)
}

// ============================================================================
// Meta Operations
// ============================================================================

pub(crate) fn set_meta_in(txn: &mut RwTxn, dbs: &Databases, key: &str, value: &str) -> Result<()> {
    dbs.meta.put(txn, key, value).map_err(err)?;
    Ok(())
}

pub(crate) fn get_meta_in(txn: &RoTxn, dbs: &Databases, key: &str) -> Result<Option<String>> {
    Ok(dbs.meta.get(txn, key).map_err(err)?.map(|s| s.to_string()))
}

// ============================================================================
// Public API - Helpers
// ============================================================================

pub fn entity_exists(entity_id: &str) -> Result<bool> {
    with_read_txn(|txn, dbs| entity_exists_in(txn, dbs, entity_id))
}

pub fn type_exists(type_name: &str) -> Result<bool> {
    with_read_txn(|txn, dbs| {
        Ok(dbs.types.get(txn, type_name).map_err(err)?.is_some())
    })
}

pub fn get_meta(key: &str) -> Result<Option<String>> {
    with_read_txn(|txn, dbs| get_meta_in(txn, dbs, key))
}

// ============================================================================
// Public API - Relationships
// ============================================================================

pub fn set_relationship(subject: &str, rel_type: &str, object: &str) -> Result<u64> {
    with_write_txn(|txn, dbs| set_relationship_in(txn, dbs, subject, rel_type, object))
}

pub fn get_relationships(subject: &str, object: &str) -> Result<Vec<String>> {
    with_read_txn(|txn, dbs| {
        let (seeker_type, seeker_id) = resolve_entity(txn, dbs, subject)?;
        let (scope_type, scope_id) = resolve_entity(txn, dbs, object)?;

        let mut results = Vec::new();
        // O(log n) prefix scan on (seeker, scope)
        let prefix = grant_seeker_scope_prefix(seeker_type, seeker_id, scope_type, scope_id);
        for item in dbs.grants.prefix_iter(txn, &prefix).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            let (skt, sk, sct, sc, role_id) = match parse_grant_key(key) {
                Some(v) => v,
                None => continue,
            };
            // Verify prefix match
            if skt != seeker_type || sk != seeker_id || sct != scope_type || sc != scope_id {
                break; // Past our prefix
            }
            if let Ok(label) = resolve_role_label(txn, dbs, role_id) {
                results.push(label);
            }
        }
        Ok(results)
    })
}

pub fn delete_relationship(subject: &str, rel_type: &str, object: &str) -> Result<bool> {
    with_write_txn(|txn, dbs| delete_relationship_in(txn, dbs, subject, rel_type, object))
}

// ============================================================================
// Public API - Capabilities
// ============================================================================

pub fn set_capability(entity: &str, rel_type: &str, cap_mask: u64) -> Result<u64> {
    with_write_txn(|txn, dbs| set_capability_in(txn, dbs, entity, rel_type, cap_mask))
}

pub fn get_capability(entity: &str, rel_type: &str) -> Result<Option<u64>> {
    with_read_txn(|txn, dbs| {
        let (scope_type, scope_id) = resolve_entity(txn, dbs, entity)?;
        let role_id = match resolve_role(txn, dbs, rel_type) {
            Ok(id) => id,
            Err(_) => return Ok(None),
        };
        let key = capability_key(scope_type, scope_id, role_id);
        Ok(dbs.capabilities.get(txn, &key).map_err(err)?)
    })
}

// ============================================================================
// Public API - Inheritance
// ============================================================================

pub fn set_inheritance(subject: &str, object: &str, source: &str) -> Result<u64> {
    with_write_txn(|txn, dbs| set_inheritance_in(txn, dbs, subject, object, source))
}

pub fn get_inheritance(subject: &str, object: &str) -> Result<Vec<String>> {
    with_read_txn(|txn, dbs| {
        let (seeker_type, seeker_id) = resolve_entity(txn, dbs, subject)?;
        let (scope_type, scope_id) = resolve_entity(txn, dbs, object)?;

        let mut results = Vec::new();
        // O(log n) prefix scan on (seeker, scope)
        let prefix = inheritance_seeker_scope_prefix(seeker_type, seeker_id, scope_type, scope_id);
        for item in dbs.inheritance.prefix_iter(txn, &prefix).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            let (skt, sk, sct, sc, src_type, src_id) = match parse_inheritance_key(key) {
                Some(v) => v,
                None => continue,
            };
            // Verify prefix match
            if skt != seeker_type || sk != seeker_id || sct != scope_type || sc != scope_id {
                break; // Past our prefix
            }
            if let Ok(label) = resolve_entity_label(txn, dbs, src_type, src_id) {
                results.push(label);
            }
        }
        Ok(results)
    })
}

pub fn delete_inheritance(subject: &str, object: &str, source: &str) -> Result<bool> {
    with_write_txn(|txn, dbs| delete_inheritance_in(txn, dbs, subject, object, source))
}

pub fn get_inheritors_from_source(source: &str, object: &str) -> Result<Vec<String>> {
    with_read_txn(|txn, dbs| {
        let (source_type, source_id) = resolve_entity(txn, dbs, source)?;
        let (scope_type, scope_id) = resolve_entity(txn, dbs, object)?;

        let mut results = Vec::new();
        for item in dbs.inheritance.iter(txn).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            let (skt, sk, sct, sc, srct, src) = match parse_inheritance_key(key) {
                Some(v) => v,
                None => continue,
            };
            if srct == source_type && src == source_id && sct == scope_type && sc == scope_id {
                if let Ok(label) = resolve_entity_label(txn, dbs, skt, sk) {
                    results.push(label);
                }
            }
        }
        Ok(results)
    })
}

pub fn get_inheritance_for_object(object: &str) -> Result<Vec<(String, String)>> {
    with_read_txn(|txn, dbs| {
        let (scope_type, scope_id) = resolve_entity(txn, dbs, object)?;

        let mut results = Vec::new();
        for item in dbs.inheritance.iter(txn).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            let (skt, sk, sct, sc, srct, src) = match parse_inheritance_key(key) {
                Some(v) => v,
                None => continue,
            };
            if sct == scope_type && sc == scope_id {
                if let (Ok(seeker_label), Ok(source_label)) = (
                    resolve_entity_label(txn, dbs, skt, sk),
                    resolve_entity_label(txn, dbs, srct, src)
                ) {
                    results.push((source_label, seeker_label));
                }
            }
        }
        Ok(results)
    })
}

// ============================================================================
// Public API - Labels
// ============================================================================

pub fn set_cap_label(entity: &str, cap_bit: u64, label: &str) -> Result<()> {
    with_write_txn(|txn, dbs| set_cap_label_in(txn, dbs, entity, cap_bit, label))
}

pub fn get_cap_label(entity: &str, cap_bit: u64) -> Result<Option<String>> {
    with_read_txn(|txn, dbs| {
        let (scope_type, scope_id) = resolve_entity(txn, dbs, entity)?;
        let key = cap_label_key(scope_type, scope_id, cap_bit as u32);
        Ok(dbs.cap_labels.get(txn, &key).map_err(err)?.map(|s| s.to_string()))
    })
}

pub fn delete_cap_label(entity: &str, cap_bit: u64) -> Result<bool> {
    with_write_txn(|txn, dbs| delete_cap_label_in(txn, dbs, entity, cap_bit))
}

// ============================================================================
// Access Checks
// ============================================================================

pub fn check_access(subject: &str, object: &str, max_depth: Option<usize>) -> Result<u64> {
    with_read_txn(|txn, dbs| {
        let (seeker_type, seeker_id) = resolve_entity(txn, dbs, subject)?;
        let (scope_type, scope_id) = resolve_entity(txn, dbs, object)?;

        let depth_limit = max_depth.unwrap_or(100);
        let mut effective_cap: u64 = 0;
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![(seeker_type, seeker_id, 0usize)];

        // Get type-level scope for type grants
        let type_scope = if !object.starts_with("_type:") {
            // Find _type:X entity
            let (type_name, _) = parse_entity_id(object)?;
            let type_entity = format!("_type:{}", type_name);
            resolve_entity(txn, dbs, &type_entity).ok()
        } else {
            None
        };

        while let Some((current_seeker_type, current_seeker_id, depth)) = stack.pop() {
            let visit_key = (current_seeker_type, current_seeker_id, scope_type, scope_id);
            if !visited.insert(visit_key) {
                continue;
            }

            // O(log n) prefix scan: find all grants from current_seeker
            let prefix = grant_seeker_prefix(current_seeker_type, current_seeker_id);
            for item in dbs.grants.prefix_iter(txn, &prefix).map_err(err)? {
                let (key, _epoch) = item.map_err(err)?;
                let (skt, sk, sct, sc, role_id) = match parse_grant_key(key) {
                    Some(v) => v,
                    None => continue,
                };

                // Verify prefix match (should always match due to prefix_iter)
                if skt != current_seeker_type || sk != current_seeker_id {
                    break; // Past our prefix
                }

                // Direct match to object
                if sct == scope_type && sc == scope_id {
                    // Look up current capability for this role
                    let cap_mask = get_capability_mask_in(txn, dbs, sct, sc, role_id)?;
                    effective_cap |= cap_mask;
                }
                // Type-level match
                else if let Some((type_scope_type, type_scope_id)) = type_scope {
                    if sct == type_scope_type && sc == type_scope_id {
                        // Look up current capability for this role on the type scope
                        let cap_mask = get_capability_mask_in(txn, dbs, sct, sc, role_id)?;
                        effective_cap |= cap_mask;
                    }
                }
            }

            // O(log n) prefix scan: find inheritances for current_seeker
            if depth < depth_limit {
                let inh_prefix = inheritance_seeker_prefix(current_seeker_type, current_seeker_id);
                for item in dbs.inheritance.prefix_iter(txn, &inh_prefix).map_err(err)? {
                    let (key, _) = item.map_err(err)?;
                    let (skt, sk, sct, sc, srct, src) = match parse_inheritance_key(key) {
                        Some(v) => v,
                        None => continue,
                    };

                    // Verify prefix match
                    if skt != current_seeker_type || sk != current_seeker_id {
                        break; // Past our prefix
                    }

                    if sct == scope_type && sc == scope_id {
                        stack.push((srct, src, depth + 1));
                    }
                }
            }
        }

        Ok(effective_cap)
    })
}

pub fn has_capability(subject: &str, object: &str, required_cap: u64) -> Result<bool> {
    Ok((check_access(subject, object, None)? & required_cap) == required_cap)
}

// ============================================================================
// Batch Operations
// ============================================================================

pub fn batch_set_relationships(entries: &[(String, String, String)]) -> Result<u64> {
    with_write_txn(|txn, dbs| {
        for (subject, rel_type, object) in entries {
            set_relationship_in(txn, dbs, subject, rel_type, object)?;
        }
        Ok(entries.len() as u64)
    })
}

pub fn batch_set_capabilities(entries: &[(String, String, u64)]) -> Result<u64> {
    with_write_txn(|txn, dbs| {
        for (entity, rel_type, cap_mask) in entries {
            set_capability_in(txn, dbs, entity, rel_type, *cap_mask)?;
        }
        Ok(entries.len() as u64)
    })
}

pub fn batch_set_inheritance(entries: &[(String, String, String)]) -> Result<u64> {
    with_write_txn(|txn, dbs| {
        for (subject, object, source) in entries {
            set_inheritance_in(txn, dbs, subject, object, source)?;
        }
        Ok(entries.len() as u64)
    })
}

// ============================================================================
// Query Operations
// ============================================================================

pub fn list_accessible(subject: &str) -> Result<Vec<(String, String)>> {
    with_read_txn(|txn, dbs| {
        let (seeker_type, seeker_id) = resolve_entity(txn, dbs, subject)?;

        let mut results = Vec::new();
        // O(log n) prefix scan on seeker
        let prefix = grant_seeker_prefix(seeker_type, seeker_id);
        for item in dbs.grants.prefix_iter(txn, &prefix).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            let (skt, sk, sct, sc, role_id) = match parse_grant_key(key) {
                Some(v) => v,
                None => continue,
            };
            // Verify prefix match
            if skt != seeker_type || sk != seeker_id {
                break; // Past our prefix
            }
            if let (Ok(scope_label), Ok(role_label)) = (
                resolve_entity_label(txn, dbs, sct, sc),
                resolve_role_label(txn, dbs, role_id)
            ) {
                results.push((scope_label, role_label));
            }
        }
        Ok(results)
    })
}

pub fn list_subjects(object: &str) -> Result<Vec<(String, String)>> {
    with_read_txn(|txn, dbs| {
        let (scope_type, scope_id) = resolve_entity(txn, dbs, object)?;

        let mut results = Vec::new();
        for item in dbs.grants.iter(txn).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            let (skt, sk, sct, sc, role_id) = match parse_grant_key(key) {
                Some(v) => v,
                None => continue,
            };
            if sct == scope_type && sc == scope_id {
                if let (Ok(seeker_label), Ok(role_label)) = (
                    resolve_entity_label(txn, dbs, skt, sk),
                    resolve_role_label(txn, dbs, role_id)
                ) {
                    results.push((seeker_label, role_label));
                }
            }
        }
        Ok(results)
    })
}

// ============================================================================
// List All Functions
// ============================================================================

pub fn list_all_entities() -> Result<Vec<String>> {
    with_read_txn(|txn, dbs| {
        let mut results = Vec::new();
        for item in dbs.entities.iter(txn).map_err(err)? {
            let (key, name) = item.map_err(err)?;
            let (type_id, _, _) = match parse_entity_key(key) {
                Some(v) => v,
                None => continue,
            };
            if let Ok(type_name) = resolve_type_name(txn, dbs, type_id) {
                results.push(format!("{}:{}", type_name, name));
            }
        }
        Ok(results)
    })
}

pub fn list_all_types() -> Result<Vec<String>> {
    with_read_txn(|txn, dbs| {
        let mut results = Vec::new();
        for item in dbs.types.iter(txn).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            results.push(key.to_string());
        }
        Ok(results)
    })
}

pub fn list_all_grants() -> Result<Vec<(String, String, String)>> {
    with_read_txn(|txn, dbs| {
        let mut results = Vec::new();
        for item in dbs.grants.iter(txn).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            let (skt, sk, sct, sc, role_id) = match parse_grant_key(key) {
                Some(v) => v,
                None => continue,
            };
            if let (Ok(seeker), Ok(scope), Ok(role)) = (
                resolve_entity_label(txn, dbs, skt, sk),
                resolve_entity_label(txn, dbs, sct, sc),
                resolve_role_label(txn, dbs, role_id)
            ) {
                results.push((seeker, role, scope));
            }
        }
        Ok(results)
    })
}

pub fn list_all_capabilities() -> Result<Vec<(String, String, u64)>> {
    with_read_txn(|txn, dbs| {
        let mut results = Vec::new();
        for item in dbs.capabilities.iter(txn).map_err(err)? {
            let (key, cap_mask) = item.map_err(err)?;
            let (scope_type, scope_id, role_id) = match parse_capability_key(key) {
                Some(v) => v,
                None => continue,
            };
            if let (Ok(type_name), Ok(role_label)) = (
                resolve_type_name(txn, dbs, scope_type),
                resolve_role_label(txn, dbs, role_id)
            ) {
                // Find entity name
                for item2 in dbs.entities.iter(txn).map_err(err)? {
                    let (ekey, name) = item2.map_err(err)?;
                    let (tid, eid, _) = match parse_entity_key(ekey) {
                        Some(v) => v,
                        None => continue,
                    };
                    if tid == scope_type && eid == scope_id {
                        results.push((format!("{}:{}", type_name, name), role_label.clone(), cap_mask));
                        break;
                    }
                }
            }
        }
        Ok(results)
    })
}

pub fn list_all_cap_labels() -> Result<Vec<(String, u64, String)>> {
    with_read_txn(|txn, dbs| {
        let mut results = Vec::new();
        for item in dbs.cap_labels.iter(txn).map_err(err)? {
            let (key, label) = item.map_err(err)?;
            let (scope_type, scope_id, bit) = match parse_cap_label_key(key) {
                Some(v) => v,
                None => continue,
            };
            if let Ok(type_name) = resolve_type_name(txn, dbs, scope_type) {
                // Find entity name
                for item2 in dbs.entities.iter(txn).map_err(err)? {
                    let (ekey, name) = item2.map_err(err)?;
                    let (tid, eid, _) = match parse_entity_key(ekey) {
                        Some(v) => v,
                        None => continue,
                    };
                    if tid == scope_type && eid == scope_id {
                        results.push((format!("{}:{}", type_name, name), bit as u64, label.to_string()));
                        break;
                    }
                }
            }
        }
        Ok(results)
    })
}

pub fn list_all_delegations() -> Result<Vec<(String, String, String)>> {
    with_read_txn(|txn, dbs| {
        let mut results = Vec::new();
        for item in dbs.inheritance.iter(txn).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            let (skt, sk, sct, sc, srct, src) = match parse_inheritance_key(key) {
                Some(v) => v,
                None => continue,
            };
            if let (Ok(seeker), Ok(scope), Ok(source)) = (
                resolve_entity_label(txn, dbs, skt, sk),
                resolve_entity_label(txn, dbs, sct, sc),
                resolve_entity_label(txn, dbs, srct, src)
            ) {
                results.push((seeker, scope, source));
            }
        }
        Ok(results)
    })
}

// ============================================================================
// WriteBatch
// ============================================================================

#[derive(Debug, Clone)]
pub enum WriteOp {
    SetRelationship { subject: String, rel_type: String, object: String },
    DeleteRelationship { subject: String, rel_type: String, object: String },
    SetCapability { entity: String, rel_type: String, cap_mask: u64 },
    SetInheritance { subject: String, object: String, source: String },
    DeleteInheritance { subject: String, object: String, source: String },
    SetCapLabel { entity: String, cap_bit: u64, label: String },
}

#[derive(Debug, Clone, Default)]
pub struct WriteBatch {
    ops: Vec<WriteOp>,
}

impl WriteBatch {
    pub fn new() -> Self { Self::default() }
    pub fn with_capacity(capacity: usize) -> Self { Self { ops: Vec::with_capacity(capacity) } }

    pub fn set_relationship(&mut self, subject: &str, rel_type: &str, object: &str) -> &mut Self {
        self.ops.push(WriteOp::SetRelationship {
            subject: subject.into(), rel_type: rel_type.into(), object: object.into(),
        });
        self
    }

    pub fn delete_relationship(&mut self, subject: &str, rel_type: &str, object: &str) -> &mut Self {
        self.ops.push(WriteOp::DeleteRelationship {
            subject: subject.into(), rel_type: rel_type.into(), object: object.into(),
        });
        self
    }

    pub fn set_capability(&mut self, entity: &str, rel_type: &str, cap_mask: u64) -> &mut Self {
        self.ops.push(WriteOp::SetCapability {
            entity: entity.into(), rel_type: rel_type.into(), cap_mask,
        });
        self
    }

    pub fn set_inheritance(&mut self, subject: &str, object: &str, source: &str) -> &mut Self {
        self.ops.push(WriteOp::SetInheritance {
            subject: subject.into(), object: object.into(), source: source.into(),
        });
        self
    }

    pub fn delete_inheritance(&mut self, subject: &str, object: &str, source: &str) -> &mut Self {
        self.ops.push(WriteOp::DeleteInheritance {
            subject: subject.into(), object: object.into(), source: source.into(),
        });
        self
    }

    pub fn set_cap_label(&mut self, entity: &str, cap_bit: u64, label: &str) -> &mut Self {
        self.ops.push(WriteOp::SetCapLabel {
            entity: entity.into(), cap_bit, label: label.into(),
        });
        self
    }

    pub fn len(&self) -> usize { self.ops.len() }
    pub fn is_empty(&self) -> bool { self.ops.is_empty() }
    pub fn clear(&mut self) { self.ops.clear(); }

    pub fn execute(&self) -> Result<u64> {
        if self.ops.is_empty() {
            return Ok(current_epoch());
        }

        with_write_txn(|txn, dbs| {
            for op in &self.ops {
                match op {
                    WriteOp::SetRelationship { subject, rel_type, object } => {
                        set_relationship_in(txn, dbs, subject, rel_type, object)?;
                    }
                    WriteOp::DeleteRelationship { subject, rel_type, object } => {
                        delete_relationship_in(txn, dbs, subject, rel_type, object)?;
                    }
                    WriteOp::SetCapability { entity, rel_type, cap_mask } => {
                        set_capability_in(txn, dbs, entity, rel_type, *cap_mask)?;
                    }
                    WriteOp::SetInheritance { subject, object, source } => {
                        set_inheritance_in(txn, dbs, subject, object, source)?;
                    }
                    WriteOp::DeleteInheritance { subject, object, source } => {
                        delete_inheritance_in(txn, dbs, subject, object, source)?;
                    }
                    WriteOp::SetCapLabel { entity, cap_bit, label } => {
                        set_cap_label_in(txn, dbs, entity, *cap_bit, label)?;
                    }
                }
            }
            Ok(current_epoch())
        })
    }
}

pub fn write_batch() -> WriteBatch {
    WriteBatch::new()
}
