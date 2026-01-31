//! Core database operations for Capbit
//!
//! High-performance access control with string-based relationships and bitmask capabilities.

use std::path::Path;
use std::sync::OnceLock;

use heed::types::{Bytes, Str, U64};
use heed::{Database, Env, EnvOpenOptions, RoTxn, RwTxn};
use serde::{Deserialize, Serialize};

use crate::keys::{build_key, build_prefix, parse_key};

// ============================================================================
// Database Setup
// ============================================================================

static ENV: OnceLock<Env> = OnceLock::new();

use std::sync::Mutex;
static TEST_LOCK: Mutex<()> = Mutex::new(());

pub(crate) struct Databases {
    // Composite keys use Bytes (length-prefixed binary format)
    pub relationships: Database<Bytes, U64<byteorder::BigEndian>>,
    pub relationships_rev: Database<Bytes, U64<byteorder::BigEndian>>,
    pub capabilities: Database<Bytes, U64<byteorder::BigEndian>>,
    pub inheritance: Database<Bytes, U64<byteorder::BigEndian>>,
    pub inheritance_by_source: Database<Bytes, U64<byteorder::BigEndian>>,
    pub inheritance_by_object: Database<Bytes, U64<byteorder::BigEndian>>,
    pub cap_labels: Database<Bytes, Str>,
    // Single-key databases stay as Str
    pub types: Database<Str, U64<byteorder::BigEndian>>,
    pub entities: Database<Str, U64<byteorder::BigEndian>>,
    pub meta: Database<Str, Str>,
    // v3: sessions (single entity key, stay as Str)
    pub sessions: Database<Str, Str>,
    pub sessions_by_entity: Database<Str, Str>,
    // v3: credentials
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

// No validation needed - length-prefixed keys allow any bytes

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
        relationships: env.create_database(&mut wtxn, Some("relationships")).map_err(err)?,
        relationships_rev: env.create_database(&mut wtxn, Some("relationships_rev")).map_err(err)?,
        capabilities: env.create_database(&mut wtxn, Some("capabilities")).map_err(err)?,
        inheritance: env.create_database(&mut wtxn, Some("inheritance")).map_err(err)?,
        inheritance_by_source: env.create_database(&mut wtxn, Some("inheritance_by_source")).map_err(err)?,
        inheritance_by_object: env.create_database(&mut wtxn, Some("inheritance_by_object")).map_err(err)?,
        cap_labels: env.create_database(&mut wtxn, Some("cap_labels")).map_err(err)?,
        types: env.create_database(&mut wtxn, Some("types")).map_err(err)?,
        entities: env.create_database(&mut wtxn, Some("entities")).map_err(err)?,
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
        dbs.relationships.clear(txn).map_err(err)?;
        dbs.relationships_rev.clear(txn).map_err(err)?;
        dbs.capabilities.clear(txn).map_err(err)?;
        dbs.inheritance.clear(txn).map_err(err)?;
        dbs.inheritance_by_source.clear(txn).map_err(err)?;
        dbs.inheritance_by_object.clear(txn).map_err(err)?;
        dbs.cap_labels.clear(txn).map_err(err)?;
        dbs.types.clear(txn).map_err(err)?;
        dbs.entities.clear(txn).map_err(err)?;
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
        Err(poisoned) => poisoned.into_inner(), // Recover from poisoned state
    }
}

// ============================================================================
// Internal Operations (take transaction)
// ============================================================================

pub(crate) fn set_relationship_in(txn: &mut RwTxn, dbs: &Databases, subject: &str, rel_type: &str, object: &str) -> Result<u64> {
    let epoch = current_epoch();
    let key = build_key(&[subject, rel_type, object]);
    let key_rev = build_key(&[object, rel_type, subject]);
    dbs.relationships.put(txn, &key, &epoch).map_err(err)?;
    dbs.relationships_rev.put(txn, &key_rev, &epoch).map_err(err)?;
    Ok(epoch)
}

pub(crate) fn delete_relationship_in(txn: &mut RwTxn, dbs: &Databases, subject: &str, rel_type: &str, object: &str) -> Result<bool> {
    let key = build_key(&[subject, rel_type, object]);
    let key_rev = build_key(&[object, rel_type, subject]);
    let deleted = dbs.relationships.delete(txn, &key).map_err(err)?;
    dbs.relationships_rev.delete(txn, &key_rev).map_err(err)?;
    Ok(deleted)
}

pub(crate) fn set_capability_in(txn: &mut RwTxn, dbs: &Databases, entity: &str, rel_type: &str, cap_mask: u64) -> Result<u64> {
    let epoch = current_epoch();
    let key = build_key(&[entity, rel_type]);
    dbs.capabilities.put(txn, &key, &cap_mask).map_err(err)?;
    Ok(epoch)
}

pub(crate) fn set_inheritance_in(txn: &mut RwTxn, dbs: &Databases, subject: &str, object: &str, source: &str) -> Result<u64> {
    let epoch = current_epoch();
    let key1 = build_key(&[subject, object, source]);
    let key2 = build_key(&[source, object, subject]);
    let key3 = build_key(&[object, source, subject]);
    dbs.inheritance.put(txn, &key1, &epoch).map_err(err)?;
    dbs.inheritance_by_source.put(txn, &key2, &epoch).map_err(err)?;
    dbs.inheritance_by_object.put(txn, &key3, &epoch).map_err(err)?;
    Ok(epoch)
}

pub(crate) fn delete_inheritance_in(txn: &mut RwTxn, dbs: &Databases, subject: &str, object: &str, source: &str) -> Result<bool> {
    let key1 = build_key(&[subject, object, source]);
    let key2 = build_key(&[source, object, subject]);
    let key3 = build_key(&[object, source, subject]);
    let deleted = dbs.inheritance.delete(txn, &key1).map_err(err)?;
    dbs.inheritance_by_source.delete(txn, &key2).map_err(err)?;
    dbs.inheritance_by_object.delete(txn, &key3).map_err(err)?;
    Ok(deleted)
}

pub(crate) fn set_cap_label_in(txn: &mut RwTxn, dbs: &Databases, entity: &str, cap_bit: u64, label: &str) -> Result<()> {
    let bit_str = format!("{:016x}", cap_bit);
    let key = build_key(&[entity, &bit_str]);
    dbs.cap_labels.put(txn, &key, label).map_err(err)?;
    Ok(())
}

// ============================================================================
// Internal Operations - Types & Entities (v2)
// ============================================================================

/// Parse entity ID string "type:id" into (type, id)
pub fn parse_entity_id(entity_id: &str) -> Result<(&str, &str)> {
    entity_id.split_once(':').ok_or_else(|| CapbitError {
        message: format!("Invalid entity ID '{}': must be 'type:id' format", entity_id),
    })
}

pub(crate) fn create_type_in(txn: &mut RwTxn, dbs: &Databases, type_name: &str) -> Result<u64> {
    let epoch = current_epoch();
    if dbs.types.get(txn, type_name).map_err(err)?.is_some() {
        return Err(CapbitError { message: format!("Type '{}' already exists", type_name) });
    }
    dbs.types.put(txn, type_name, &epoch).map_err(err)?;
    Ok(epoch)
}

fn type_exists_in(txn: &RoTxn, dbs: &Databases, type_name: &str) -> Result<bool> {
    Ok(dbs.types.get(txn, type_name).map_err(err)?.is_some())
}

/// Create entity from "type:id" string
pub(crate) fn create_entity_in(txn: &mut RwTxn, dbs: &Databases, entity_id: &str) -> Result<u64> {
    let (type_name, _id) = parse_entity_id(entity_id)?;

    // Check type exists (except for _type: entities during bootstrap)
    if !type_name.starts_with('_') {
        if !type_exists_in(txn, dbs, type_name)? {
            return Err(CapbitError { message: format!("Type '{}' does not exist", type_name) });
        }
    }

    let epoch = current_epoch();
    if dbs.entities.get(txn, entity_id).map_err(err)?.is_some() {
        return Err(CapbitError { message: format!("Entity '{}' already exists", entity_id) });
    }
    dbs.entities.put(txn, entity_id, &epoch).map_err(err)?;
    Ok(epoch)
}

pub(crate) fn delete_entity_in(txn: &mut RwTxn, dbs: &Databases, entity_id: &str) -> Result<bool> {
    Ok(dbs.entities.delete(txn, entity_id).map_err(err)?)
}

pub(crate) fn entity_exists_in(txn: &RoTxn, dbs: &Databases, entity_id: &str) -> Result<bool> {
    Ok(dbs.entities.get(txn, entity_id).map_err(err)?.is_some())
}

pub(crate) fn set_meta_in(txn: &mut RwTxn, dbs: &Databases, key: &str, value: &str) -> Result<()> {
    dbs.meta.put(txn, key, value).map_err(err)?;
    Ok(())
}

pub(crate) fn get_meta_in(txn: &RoTxn, dbs: &Databases, key: &str) -> Result<Option<String>> {
    Ok(dbs.meta.get(txn, key).map_err(err)?.map(|s| s.to_string()))
}


// Public helpers
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
        let prefix = build_prefix(&[subject]);
        let mut results = Vec::new();

        for item in dbs.relationships.prefix_iter(txn, &prefix).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            let parts = parse_key(key);
            if parts.len() == 3 && parts[2] == object {
                results.push(parts[1].to_string());
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
        let key = build_key(&[entity, rel_type]);
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
        let prefix = build_prefix(&[subject, object]);
        let mut results = Vec::new();

        for item in dbs.inheritance.prefix_iter(txn, &prefix).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            let parts = parse_key(key);
            if parts.len() == 3 {
                results.push(parts[2].to_string());
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
        let prefix = build_prefix(&[source, object]);
        let mut results = Vec::new();

        for item in dbs.inheritance_by_source.prefix_iter(txn, &prefix).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            let parts = parse_key(key);
            if parts.len() == 3 {
                results.push(parts[2].to_string());
            }
        }
        Ok(results)
    })
}

pub fn get_inheritance_for_object(object: &str) -> Result<Vec<(String, String)>> {
    with_read_txn(|txn, dbs| {
        let prefix = build_prefix(&[object]);
        let mut results = Vec::new();

        for item in dbs.inheritance_by_object.prefix_iter(txn, &prefix).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            let parts = parse_key(key);
            if parts.len() == 3 {
                results.push((parts[1].to_string(), parts[2].to_string()));
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
        let bit_str = format!("{:016x}", cap_bit);
        let key = build_key(&[entity, &bit_str]);
        Ok(dbs.cap_labels.get(txn, &key).map_err(err)?.map(|s| s.to_string()))
    })
}

// ============================================================================
// Public API - Access Checks
// ============================================================================

pub fn check_access(subject: &str, object: &str, max_depth: Option<usize>) -> Result<u64> {
    with_read_txn(|txn, dbs| {
        let depth_limit = max_depth.unwrap_or(100);
        let mut effective_cap: u64 = 0;
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![(subject.to_string(), 0usize)];

        // Check if object is a typed entity (e.g., "team:engineering") and get type scope
        // Skip if object is already a type entity (starts with "_type:")
        let type_scope = if !object.starts_with("_type:") {
            object.split_once(':').map(|(t, _)| format!("_type:{}", t))
        } else {
            None
        };

        while let Some((current, depth)) = stack.pop() {
            let visit_key = format!("{}:{}", current, object);
            if !visited.insert(visit_key) {
                continue;
            }

            let prefix = build_prefix(&[&current]);

            // Get relationships - check both direct object and type-level in one pass
            for item in dbs.relationships.prefix_iter(txn, &prefix).map_err(err)? {
                let (key, _) = item.map_err(err)?;
                let parts = parse_key(key);
                if parts.len() != 3 { continue; }

                let rel = parts[1];
                let target = parts[2];

                // Direct match to object
                if target == object {
                    let cap_key = build_key(&[object, rel]);
                    if let Some(cap) = dbs.capabilities.get(txn, &cap_key).map_err(err)? {
                        effective_cap |= cap;
                    }
                }
                // Type-level match (e.g., grants on _type:team apply to all teams)
                else if let Some(ref type_scope_str) = type_scope {
                    if target == type_scope_str {
                        let cap_key = build_key(&[type_scope_str, rel]);
                        if let Some(cap) = dbs.capabilities.get(txn, &cap_key).map_err(err)? {
                            effective_cap |= cap;
                        }
                    }
                }
            }

            // Get inheritance sources
            if depth < depth_limit {
                let inherit_prefix = build_prefix(&[&current, object]);
                for item in dbs.inheritance.prefix_iter(txn, &inherit_prefix).map_err(err)? {
                    let (key, _) = item.map_err(err)?;
                    let parts = parse_key(key);
                    if parts.len() == 3 {
                        stack.push((parts[2].to_string(), depth + 1));
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
// Public API - Batch Operations
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
// Public API - Query Operations
// ============================================================================

fn list_relations_by_prefix(
    txn: &RoTxn,
    db: &Database<Bytes, U64<byteorder::BigEndian>>,
    prefix_key: &str,
) -> Result<Vec<(String, String)>> {
    let prefix = build_prefix(&[prefix_key]);
    let mut results = Vec::new();
    for item in db.prefix_iter(txn, &prefix).map_err(err)? {
        let (key, _) = item.map_err(err)?;
        let parts = parse_key(key);
        if parts.len() == 3 {
            results.push((parts[2].to_string(), parts[1].to_string()));
        }
    }
    Ok(results)
}

pub fn list_accessible(subject: &str) -> Result<Vec<(String, String)>> {
    with_read_txn(|txn, dbs| list_relations_by_prefix(txn, &dbs.relationships, subject))
}

pub fn list_subjects(object: &str) -> Result<Vec<(String, String)>> {
    with_read_txn(|txn, dbs| list_relations_by_prefix(txn, &dbs.relationships_rev, object))
}

// ============================================================================
// WriteBatch - Explicit Transaction API
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { ops: Vec::with_capacity(capacity) }
    }

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

// ============================================================================
// List All Functions (for admin/demo purposes)
// ============================================================================

/// List all registered entities
pub fn list_all_entities() -> Result<Vec<String>> {
    with_read_txn(|txn, dbs| {
        let mut results = Vec::new();
        for item in dbs.entities.iter(txn).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            results.push(key.to_string());
        }
        Ok(results)
    })
}

/// List all registered types
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

/// List all relationships (grants)
pub fn list_all_grants() -> Result<Vec<(String, String, String)>> {
    with_read_txn(|txn, dbs| {
        let mut results = Vec::new();
        for item in dbs.relationships.iter(txn).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            let parts = parse_key(key);
            if parts.len() == 3 {
                results.push((parts[0].to_string(), parts[1].to_string(), parts[2].to_string()));
            }
        }
        Ok(results)
    })
}

/// List all capability definitions
pub fn list_all_capabilities() -> Result<Vec<(String, String, u64)>> {
    with_read_txn(|txn, dbs| {
        let mut results = Vec::new();
        for item in dbs.capabilities.iter(txn).map_err(err)? {
            let (key, cap_mask) = item.map_err(err)?;
            let parts = parse_key(key);
            if parts.len() == 2 {
                results.push((parts[0].to_string(), parts[1].to_string(), cap_mask));
            }
        }
        Ok(results)
    })
}

/// List all capability bit labels
pub fn list_all_cap_labels() -> Result<Vec<(String, u64, String)>> {
    with_read_txn(|txn, dbs| {
        let mut results = Vec::new();
        for item in dbs.cap_labels.iter(txn).map_err(err)? {
            let (key, label) = item.map_err(err)?;
            let parts = parse_key(key);
            if parts.len() == 2 {
                if let Ok(bit) = u64::from_str_radix(parts[1], 16) {
                    results.push((parts[0].to_string(), bit, label.to_string()));
                }
            }
        }
        Ok(results)
    })
}

/// List all delegations (inheritance records)
pub fn list_all_delegations() -> Result<Vec<(String, String, String)>> {
    with_read_txn(|txn, dbs| {
        let mut results = Vec::new();
        for item in dbs.inheritance.iter(txn).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            let parts = parse_key(key);
            if parts.len() == 3 {
                // subject, object, source
                results.push((parts[0].to_string(), parts[1].to_string(), parts[2].to_string()));
            }
        }
        Ok(results)
    })
}
