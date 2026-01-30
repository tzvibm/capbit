//! Core database operations for Capbit
//!
//! Refactored design:
//! - Relationship types: strings (unlimited, readable)
//! - Capability masks: bitmasks (O(1) evaluation)
//!
//! Write strategies:
//! - Single-op (default): One transaction per operation, simple but high contention
//! - Explicit transactions: User controls transaction boundaries for batching/atomicity
//! - Buffered writes: Background coalescing for high-throughput fire-and-forget
//!
//! Storage patterns:
//! - `subject/rel_type/object` → epoch (relationships)
//! - `object/rel_type` → cap_mask (capability definitions)
//! - `subject/object/source` → epoch (inheritance)

use std::borrow::Cow;
use std::path::Path;
use std::sync::OnceLock;

use heed::types::*;
use heed::{Database, Env, EnvOpenOptions};
use serde::{Deserialize, Serialize};

/// Global environment singleton
static ENV: OnceLock<Env> = OnceLock::new();

/// Sub-databases
struct Databases {
    /// subject/rel_type/object -> epoch
    relationships: Database<Str, U64<byteorder::BigEndian>>,
    /// Reverse: object/rel_type/subject -> epoch
    relationships_rev: Database<Str, U64<byteorder::BigEndian>>,
    /// entity/rel_type -> cap_mask (capability definitions)
    capabilities: Database<Str, U64<byteorder::BigEndian>>,
    /// subject/object/source -> epoch (inheritance)
    /// Query: "Who does subject inherit from for object?"
    inheritance: Database<Str, U64<byteorder::BigEndian>>,
    /// source/object/subject -> epoch
    /// Query: "Who inherits from source for object?"
    inheritance_by_source: Database<Str, U64<byteorder::BigEndian>>,
    /// object/source/subject -> epoch
    /// Query: "What inheritance rules affect object?" and "Who inherits from source for object?"
    inheritance_by_object: Database<Str, U64<byteorder::BigEndian>>,
    /// Labels for capability bits: entity/cap_bit -> label
    cap_labels: Database<Str, Str>,
}

static DBS: OnceLock<Databases> = OnceLock::new();

/// Error type for core operations
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

impl From<String> for CapbitError {
    fn from(s: String) -> Self {
        CapbitError { message: s }
    }
}

impl From<&str> for CapbitError {
    fn from(s: &str) -> Self {
        CapbitError { message: s.to_string() }
    }
}

pub type Result<T> = std::result::Result<T, CapbitError>;

/// Initialize the LMDB environment
/// Returns Ok(()) if already initialized (idempotent)
pub fn init(db_path: &str) -> Result<()> {
    // Already initialized - return success (idempotent)
    if ENV.get().is_some() {
        return Ok(());
    }

    let path = Path::new(db_path);
    std::fs::create_dir_all(path).map_err(|e| CapbitError::from(e.to_string()))?;

    let env = unsafe {
        EnvOpenOptions::new()
            .map_size(10 * 1024 * 1024 * 1024) // 10GB for scale
            .max_dbs(10)
            .open(path)
            .map_err(|e| CapbitError::from(e.to_string()))?
    };

    let mut wtxn = env.write_txn().map_err(|e| CapbitError::from(e.to_string()))?;

    let dbs = Databases {
        relationships: env
            .create_database(&mut wtxn, Some("relationships"))
            .map_err(|e| CapbitError::from(e.to_string()))?,
        relationships_rev: env
            .create_database(&mut wtxn, Some("relationships_rev"))
            .map_err(|e| CapbitError::from(e.to_string()))?,
        capabilities: env
            .create_database(&mut wtxn, Some("capabilities"))
            .map_err(|e| CapbitError::from(e.to_string()))?,
        inheritance: env
            .create_database(&mut wtxn, Some("inheritance"))
            .map_err(|e| CapbitError::from(e.to_string()))?,
        inheritance_by_source: env
            .create_database(&mut wtxn, Some("inheritance_by_source"))
            .map_err(|e| CapbitError::from(e.to_string()))?,
        inheritance_by_object: env
            .create_database(&mut wtxn, Some("inheritance_by_object"))
            .map_err(|e| CapbitError::from(e.to_string()))?,
        cap_labels: env
            .create_database(&mut wtxn, Some("cap_labels"))
            .map_err(|e| CapbitError::from(e.to_string()))?,
    };

    wtxn.commit().map_err(|e| CapbitError::from(e.to_string()))?;

    let _ = ENV.set(env);
    let _ = DBS.set(dbs);

    Ok(())
}

fn get_env() -> Result<&'static Env> {
    ENV.get().ok_or_else(|| CapbitError::from("Database not initialized. Call init() first."))
}

fn get_dbs() -> Result<&'static Databases> {
    DBS.get().ok_or_else(|| CapbitError::from("Database not initialized. Call init() first."))
}

fn current_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

// Helper to escape forward slashes in entity IDs and rel_types
// Optimized: only allocates if escaping is needed
fn escape_key_part(s: &str) -> Cow<'_, str> {
    if s.contains('/') || s.contains('\\') {
        Cow::Owned(s.replace('\\', "\\\\").replace('/', "\\/"))
    } else {
        Cow::Borrowed(s)
    }
}

fn unescape_key_part(s: &str) -> Cow<'_, str> {
    if s.contains('\\') {
        Cow::Owned(s.replace("\\/", "/").replace("\\\\", "\\"))
    } else {
        Cow::Borrowed(s)
    }
}

// ============================================================================
// Relationship Operations (subject/rel_type/object)
// ============================================================================

/// Set a relationship between two entities
/// rel_type is now a string (e.g., "editor", "viewer", "member")
pub fn set_relationship(subject: &str, rel_type: &str, object: &str) -> Result<u64> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let mut wtxn = env.write_txn().map_err(|e| CapbitError::from(e.to_string()))?;

    let epoch = current_epoch();
    let subj_esc = escape_key_part(subject);
    let rel_esc = escape_key_part(rel_type);
    let obj_esc = escape_key_part(object);

    let forward_key = format!("{}/{}/{}", subj_esc, rel_esc, obj_esc);
    let reverse_key = format!("{}/{}/{}", obj_esc, rel_esc, subj_esc);

    dbs.relationships
        .put(&mut wtxn, &forward_key, &epoch)
        .map_err(|e| CapbitError::from(e.to_string()))?;
    dbs.relationships_rev
        .put(&mut wtxn, &reverse_key, &epoch)
        .map_err(|e| CapbitError::from(e.to_string()))?;

    wtxn.commit().map_err(|e| CapbitError::from(e.to_string()))?;
    Ok(epoch)
}

/// Get all relationship types between subject and object
/// Returns Vec of relation type strings (e.g., ["editor", "viewer"])
pub fn get_relationships(subject: &str, object: &str) -> Result<Vec<String>> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let rtxn = env.read_txn().map_err(|e| CapbitError::from(e.to_string()))?;

    let subj_esc = escape_key_part(subject);
    let obj_esc = escape_key_part(object);
    let prefix = format!("{}/", subj_esc);
    let suffix = format!("/{}", obj_esc);
    let mut results = Vec::new();

    let iter = dbs
        .relationships
        .prefix_iter(&rtxn, &prefix)
        .map_err(|e| CapbitError::from(e.to_string()))?;

    for item in iter {
        let (key, _epoch) = item.map_err(|e| CapbitError::from(e.to_string()))?;
        if key.ends_with(&suffix) {
            // Extract rel_type from key: subject/rel_type/object
            let without_prefix = &key[prefix.len()..];
            let without_suffix = &without_prefix[..without_prefix.len() - suffix.len()];
            results.push(unescape_key_part(without_suffix).into_owned());
        }
    }

    Ok(results)
}

/// Delete a relationship
pub fn delete_relationship(subject: &str, rel_type: &str, object: &str) -> Result<bool> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let mut wtxn = env.write_txn().map_err(|e| CapbitError::from(e.to_string()))?;

    let subj_esc = escape_key_part(subject);
    let rel_esc = escape_key_part(rel_type);
    let obj_esc = escape_key_part(object);

    let forward_key = format!("{}/{}/{}", subj_esc, rel_esc, obj_esc);
    let reverse_key = format!("{}/{}/{}", obj_esc, rel_esc, subj_esc);

    let deleted = dbs
        .relationships
        .delete(&mut wtxn, &forward_key)
        .map_err(|e| CapbitError::from(e.to_string()))?;
    dbs.relationships_rev
        .delete(&mut wtxn, &reverse_key)
        .map_err(|e| CapbitError::from(e.to_string()))?;

    wtxn.commit().map_err(|e| CapbitError::from(e.to_string()))?;
    Ok(deleted)
}

// ============================================================================
// Capability Operations (entity/rel_type -> cap_mask)
// ============================================================================

/// Define what capabilities a relationship type grants on an entity
/// e.g., set_capability("slack", "editor", READ | WRITE | DELETE)
pub fn set_capability(entity: &str, rel_type: &str, cap_mask: u64) -> Result<u64> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let mut wtxn = env.write_txn().map_err(|e| CapbitError::from(e.to_string()))?;

    let epoch = current_epoch();
    let ent_esc = escape_key_part(entity);
    let rel_esc = escape_key_part(rel_type);
    let key = format!("{}/{}", ent_esc, rel_esc);

    dbs.capabilities
        .put(&mut wtxn, &key, &cap_mask)
        .map_err(|e| CapbitError::from(e.to_string()))?;

    wtxn.commit().map_err(|e| CapbitError::from(e.to_string()))?;
    Ok(epoch)
}

/// Get capability mask for a relationship type on an entity
pub fn get_capability(entity: &str, rel_type: &str) -> Result<Option<u64>> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let rtxn = env.read_txn().map_err(|e| CapbitError::from(e.to_string()))?;

    let ent_esc = escape_key_part(entity);
    let rel_esc = escape_key_part(rel_type);
    let key = format!("{}/{}", ent_esc, rel_esc);

    let result = dbs
        .capabilities
        .get(&rtxn, &key)
        .map_err(|e| CapbitError::from(e.to_string()))?;

    Ok(result)
}

// ============================================================================
// Inheritance Operations (subject/object/source)
// ============================================================================

/// Set inheritance: subject inherits source's relationship to object
pub fn set_inheritance(subject: &str, object: &str, source: &str) -> Result<u64> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let mut wtxn = env.write_txn().map_err(|e| CapbitError::from(e.to_string()))?;

    let epoch = current_epoch();
    let subj_esc = escape_key_part(subject);
    let obj_esc = escape_key_part(object);
    let src_esc = escape_key_part(source);

    // Three indexes for different query patterns:
    // 1. subject/object/source - "Who does subject inherit from for object?"
    let by_subject_key = format!("{}/{}/{}", subj_esc, obj_esc, src_esc);
    // 2. source/object/subject - "Who inherits from source for object?"
    let by_source_key = format!("{}/{}/{}", src_esc, obj_esc, subj_esc);
    // 3. object/source/subject - "What inheritance rules affect object?"
    let by_object_key = format!("{}/{}/{}", obj_esc, src_esc, subj_esc);

    dbs.inheritance
        .put(&mut wtxn, &by_subject_key, &epoch)
        .map_err(|e| CapbitError::from(e.to_string()))?;
    dbs.inheritance_by_source
        .put(&mut wtxn, &by_source_key, &epoch)
        .map_err(|e| CapbitError::from(e.to_string()))?;
    dbs.inheritance_by_object
        .put(&mut wtxn, &by_object_key, &epoch)
        .map_err(|e| CapbitError::from(e.to_string()))?;

    wtxn.commit().map_err(|e| CapbitError::from(e.to_string()))?;
    Ok(epoch)
}

/// Get inheritance sources for subject's relationship to object
pub fn get_inheritance(subject: &str, object: &str) -> Result<Vec<String>> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let rtxn = env.read_txn().map_err(|e| CapbitError::from(e.to_string()))?;

    let subj_esc = escape_key_part(subject);
    let obj_esc = escape_key_part(object);
    let prefix = format!("{}/{}/", subj_esc, obj_esc);
    let mut results = Vec::new();

    let iter = dbs
        .inheritance
        .prefix_iter(&rtxn, &prefix)
        .map_err(|e| CapbitError::from(e.to_string()))?;

    for item in iter {
        let (key, _epoch) = item.map_err(|e| CapbitError::from(e.to_string()))?;
        let source = &key[prefix.len()..];
        results.push(unescape_key_part(source).into_owned());
    }

    Ok(results)
}

/// Delete an inheritance rule
pub fn delete_inheritance(subject: &str, object: &str, source: &str) -> Result<bool> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let mut wtxn = env.write_txn().map_err(|e| CapbitError::from(e.to_string()))?;

    let subj_esc = escape_key_part(subject);
    let obj_esc = escape_key_part(object);
    let src_esc = escape_key_part(source);

    let by_subject_key = format!("{}/{}/{}", subj_esc, obj_esc, src_esc);
    let by_source_key = format!("{}/{}/{}", src_esc, obj_esc, subj_esc);
    let by_object_key = format!("{}/{}/{}", obj_esc, src_esc, subj_esc);

    let deleted = dbs
        .inheritance
        .delete(&mut wtxn, &by_subject_key)
        .map_err(|e| CapbitError::from(e.to_string()))?;
    dbs.inheritance_by_source
        .delete(&mut wtxn, &by_source_key)
        .map_err(|e| CapbitError::from(e.to_string()))?;
    dbs.inheritance_by_object
        .delete(&mut wtxn, &by_object_key)
        .map_err(|e| CapbitError::from(e.to_string()))?;

    wtxn.commit().map_err(|e| CapbitError::from(e.to_string()))?;
    Ok(deleted)
}

/// Get all subjects that inherit from source for a specific object
pub fn get_inheritors_from_source(source: &str, object: &str) -> Result<Vec<String>> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let rtxn = env.read_txn().map_err(|e| CapbitError::from(e.to_string()))?;

    let src_esc = escape_key_part(source);
    let obj_esc = escape_key_part(object);
    let prefix = format!("{}/{}/", src_esc, obj_esc);
    let mut results = Vec::new();

    let iter = dbs
        .inheritance_by_source
        .prefix_iter(&rtxn, &prefix)
        .map_err(|e| CapbitError::from(e.to_string()))?;

    for item in iter {
        let (key, _epoch) = item.map_err(|e| CapbitError::from(e.to_string()))?;
        let subject = &key[prefix.len()..];
        results.push(unescape_key_part(subject).into_owned());
    }

    Ok(results)
}

/// Get all inheritance rules for an object
/// Returns Vec of (source, subject) pairs - "subject inherits from source for this object"
pub fn get_inheritance_for_object(object: &str) -> Result<Vec<(String, String)>> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let rtxn = env.read_txn().map_err(|e| CapbitError::from(e.to_string()))?;

    let obj_esc = escape_key_part(object);
    let prefix = format!("{}/", obj_esc);
    let mut results = Vec::new();

    let iter = dbs
        .inheritance_by_object
        .prefix_iter(&rtxn, &prefix)
        .map_err(|e| CapbitError::from(e.to_string()))?;

    for item in iter {
        let (key, _epoch) = item.map_err(|e| CapbitError::from(e.to_string()))?;
        let rest = &key[prefix.len()..];
        // key format: object/source/subject
        if let Some(slash_pos) = rest.find('/') {
            let source = unescape_key_part(&rest[..slash_pos]).into_owned();
            let subject = unescape_key_part(&rest[slash_pos + 1..]).into_owned();
            results.push((source, subject));
        }
    }

    Ok(results)
}

// ============================================================================
// Label Operations (for capability bits)
// ============================================================================

/// Set a label for a capability bit on an entity
/// e.g., set_cap_label("myapp", 0x01, "read")
pub fn set_cap_label(entity: &str, cap_bit: u64, label: &str) -> Result<()> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let mut wtxn = env.write_txn().map_err(|e| CapbitError::from(e.to_string()))?;

    let ent_esc = escape_key_part(entity);
    let key = format!("{}/{:016x}", ent_esc, cap_bit);

    dbs.cap_labels
        .put(&mut wtxn, &key, label)
        .map_err(|e| CapbitError::from(e.to_string()))?;

    wtxn.commit().map_err(|e| CapbitError::from(e.to_string()))?;
    Ok(())
}

/// Get label for a capability bit
pub fn get_cap_label(entity: &str, cap_bit: u64) -> Result<Option<String>> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let rtxn = env.read_txn().map_err(|e| CapbitError::from(e.to_string()))?;

    let ent_esc = escape_key_part(entity);
    let key = format!("{}/{:016x}", ent_esc, cap_bit);

    let result = dbs
        .cap_labels
        .get(&rtxn, &key)
        .map_err(|e| CapbitError::from(e.to_string()))?
        .map(|s| s.to_string());

    Ok(result)
}

// ============================================================================
// Access Evaluation
// ============================================================================

/// Check what capabilities subject has on object
/// Returns the effective capability mask (0 if no access)
pub fn check_access(subject: &str, object: &str, max_depth: Option<usize>) -> Result<u64> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let rtxn = env.read_txn().map_err(|e| CapbitError::from(e.to_string()))?;
    let depth_limit = max_depth.unwrap_or(100);

    let mut effective_cap: u64 = 0;
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Use a stack for iterative depth-first traversal
    let mut stack: Vec<(String, usize)> = vec![(subject.to_string(), 0)];

    let obj_esc = escape_key_part(object);

    while let Some((current_subject, depth)) = stack.pop() {
        // Prevent cycles
        let visit_key = format!("{}:{}", current_subject, object);
        if visited.contains(&visit_key) {
            continue;
        }
        visited.insert(visit_key);

        let subj_esc = escape_key_part(&current_subject);

        // Step 1: Get direct relationships for current_subject -> object
        let prefix = format!("{}/", subj_esc);
        let suffix = format!("/{}", obj_esc);

        let iter = dbs
            .relationships
            .prefix_iter(&rtxn, &prefix)
            .map_err(|e| CapbitError::from(e.to_string()))?;

        for item in iter {
            let (key, _epoch) = item.map_err(|e| CapbitError::from(e.to_string()))?;
            if key.ends_with(&suffix) {
                // Extract rel_type
                let without_prefix = &key[prefix.len()..];
                let rel_type_esc = &without_prefix[..without_prefix.len() - suffix.len()];

                // Step 2: Get capabilities for this relationship type on object
                let cap_key = format!("{}/{}", obj_esc, rel_type_esc);
                if let Some(cap_mask) = dbs
                    .capabilities
                    .get(&rtxn, &cap_key)
                    .map_err(|e| CapbitError::from(e.to_string()))?
                {
                    effective_cap |= cap_mask;
                }
            }
        }

        // Step 3: Check inheritance - add sources to stack
        if depth < depth_limit {
            let inherit_prefix = format!("{}/{}/", subj_esc, obj_esc);
            let inherit_iter = dbs
                .inheritance
                .prefix_iter(&rtxn, &inherit_prefix)
                .map_err(|e| CapbitError::from(e.to_string()))?;

            for item in inherit_iter {
                let (key, _epoch) = item.map_err(|e| CapbitError::from(e.to_string()))?;
                let source_esc = &key[inherit_prefix.len()..];
                let source = unescape_key_part(source_esc).into_owned();
                stack.push((source, depth + 1));
            }
        }
    }

    Ok(effective_cap)
}

/// Check if subject has specific capability on object
pub fn has_capability(subject: &str, object: &str, required_cap: u64) -> Result<bool> {
    let effective = check_access(subject, object, None)?;
    Ok((effective & required_cap) == required_cap)
}

// ============================================================================
// Batch Operations
// ============================================================================

/// Batch set relationships
/// Each entry is (subject, rel_type, object)
pub fn batch_set_relationships(entries: &[(String, String, String)]) -> Result<u64> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let mut wtxn = env.write_txn().map_err(|e| CapbitError::from(e.to_string()))?;

    let epoch = current_epoch();
    let mut count = 0u64;

    for (subject, rel_type, object) in entries {
        let subj_esc = escape_key_part(subject);
        let rel_esc = escape_key_part(rel_type);
        let obj_esc = escape_key_part(object);

        let forward_key = format!("{}/{}/{}", subj_esc, rel_esc, obj_esc);
        let reverse_key = format!("{}/{}/{}", obj_esc, rel_esc, subj_esc);

        dbs.relationships
            .put(&mut wtxn, &forward_key, &epoch)
            .map_err(|e| CapbitError::from(e.to_string()))?;
        dbs.relationships_rev
            .put(&mut wtxn, &reverse_key, &epoch)
            .map_err(|e| CapbitError::from(e.to_string()))?;

        count += 1;
    }

    wtxn.commit().map_err(|e| CapbitError::from(e.to_string()))?;
    Ok(count)
}

/// Batch set capabilities
/// Each entry is (entity, rel_type, cap_mask)
pub fn batch_set_capabilities(entries: &[(String, String, u64)]) -> Result<u64> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let mut wtxn = env.write_txn().map_err(|e| CapbitError::from(e.to_string()))?;

    let mut count = 0u64;

    for (entity, rel_type, cap_mask) in entries {
        let ent_esc = escape_key_part(entity);
        let rel_esc = escape_key_part(rel_type);
        let key = format!("{}/{}", ent_esc, rel_esc);

        dbs.capabilities
            .put(&mut wtxn, &key, cap_mask)
            .map_err(|e| CapbitError::from(e.to_string()))?;

        count += 1;
    }

    wtxn.commit().map_err(|e| CapbitError::from(e.to_string()))?;
    Ok(count)
}

/// Batch set inheritance
/// Each entry is (subject, object, source)
pub fn batch_set_inheritance(entries: &[(String, String, String)]) -> Result<u64> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let mut wtxn = env.write_txn().map_err(|e| CapbitError::from(e.to_string()))?;

    let epoch = current_epoch();
    let mut count = 0u64;

    for (subject, object, source) in entries {
        let subj_esc = escape_key_part(subject);
        let obj_esc = escape_key_part(object);
        let src_esc = escape_key_part(source);

        // Three indexes for different query patterns
        let by_subject_key = format!("{}/{}/{}", subj_esc, obj_esc, src_esc);
        let by_source_key = format!("{}/{}/{}", src_esc, obj_esc, subj_esc);
        let by_object_key = format!("{}/{}/{}", obj_esc, src_esc, subj_esc);

        dbs.inheritance
            .put(&mut wtxn, &by_subject_key, &epoch)
            .map_err(|e| CapbitError::from(e.to_string()))?;
        dbs.inheritance_by_source
            .put(&mut wtxn, &by_source_key, &epoch)
            .map_err(|e| CapbitError::from(e.to_string()))?;
        dbs.inheritance_by_object
            .put(&mut wtxn, &by_object_key, &epoch)
            .map_err(|e| CapbitError::from(e.to_string()))?;

        count += 1;
    }

    wtxn.commit().map_err(|e| CapbitError::from(e.to_string()))?;
    Ok(count)
}

// ============================================================================
// Query Operations
// ============================================================================

/// List all entities that subject has any relationship with
pub fn list_accessible(subject: &str) -> Result<Vec<(String, String)>> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let rtxn = env.read_txn().map_err(|e| CapbitError::from(e.to_string()))?;

    let subj_esc = escape_key_part(subject);
    let prefix = format!("{}/", subj_esc);
    let mut results = Vec::new();

    let iter = dbs
        .relationships
        .prefix_iter(&rtxn, &prefix)
        .map_err(|e| CapbitError::from(e.to_string()))?;

    for item in iter {
        let (key, _epoch) = item.map_err(|e| CapbitError::from(e.to_string()))?;
        let rest = &key[prefix.len()..];
        // Find the last / to split rel_type and object
        if let Some(slash_pos) = rest.rfind('/') {
            let rel_type = unescape_key_part(&rest[..slash_pos]).into_owned();
            let object = unescape_key_part(&rest[slash_pos + 1..]).into_owned();
            results.push((object, rel_type));
        }
    }

    Ok(results)
}

/// List all subjects that have any relationship to object
pub fn list_subjects(object: &str) -> Result<Vec<(String, String)>> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let rtxn = env.read_txn().map_err(|e| CapbitError::from(e.to_string()))?;

    let obj_esc = escape_key_part(object);
    let prefix = format!("{}/", obj_esc);
    let mut results = Vec::new();

    let iter = dbs
        .relationships_rev
        .prefix_iter(&rtxn, &prefix)
        .map_err(|e| CapbitError::from(e.to_string()))?;

    for item in iter {
        let (key, _epoch) = item.map_err(|e| CapbitError::from(e.to_string()))?;
        let rest = &key[prefix.len()..];
        if let Some(slash_pos) = rest.rfind('/') {
            let rel_type = unescape_key_part(&rest[..slash_pos]).into_owned();
            let subject = unescape_key_part(&rest[slash_pos + 1..]).into_owned();
            results.push((subject, rel_type));
        }
    }

    Ok(results)
}

pub fn close() {
    // LMDB handles cleanup on drop
}

// ============================================================================
// WriteBatch - Explicit Transaction API
// ============================================================================

/// Operation types for WriteBatch
#[derive(Debug, Clone)]
pub enum WriteOp {
    SetRelationship { subject: String, rel_type: String, object: String },
    DeleteRelationship { subject: String, rel_type: String, object: String },
    SetCapability { entity: String, rel_type: String, cap_mask: u64 },
    SetInheritance { subject: String, object: String, source: String },
    DeleteInheritance { subject: String, object: String, source: String },
    SetCapLabel { entity: String, cap_bit: u64, label: String },
}

/// A batch of write operations to be executed in a single transaction.
///
/// This provides:
/// - Atomicity: All operations succeed or all fail
/// - Performance: Single transaction for multiple operations (reduces lock contention)
///
/// # Example
/// ```ignore
/// let mut batch = WriteBatch::new();
/// batch.set_relationship("john", "editor", "doc1");
/// batch.set_relationship("john", "viewer", "doc2");
/// batch.set_capability("doc1", "editor", READ | WRITE);
/// batch.execute()?; // All operations in one transaction
/// ```
#[derive(Debug, Clone, Default)]
pub struct WriteBatch {
    ops: Vec<WriteOp>,
}

impl WriteBatch {
    /// Create a new empty batch
    pub fn new() -> Self {
        Self { ops: Vec::new() }
    }

    /// Create a batch with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self { ops: Vec::with_capacity(capacity) }
    }

    /// Add a relationship operation
    pub fn set_relationship(&mut self, subject: &str, rel_type: &str, object: &str) -> &mut Self {
        self.ops.push(WriteOp::SetRelationship {
            subject: subject.to_string(),
            rel_type: rel_type.to_string(),
            object: object.to_string(),
        });
        self
    }

    /// Add a delete relationship operation
    pub fn delete_relationship(&mut self, subject: &str, rel_type: &str, object: &str) -> &mut Self {
        self.ops.push(WriteOp::DeleteRelationship {
            subject: subject.to_string(),
            rel_type: rel_type.to_string(),
            object: object.to_string(),
        });
        self
    }

    /// Add a capability operation
    pub fn set_capability(&mut self, entity: &str, rel_type: &str, cap_mask: u64) -> &mut Self {
        self.ops.push(WriteOp::SetCapability {
            entity: entity.to_string(),
            rel_type: rel_type.to_string(),
            cap_mask,
        });
        self
    }

    /// Add an inheritance operation
    pub fn set_inheritance(&mut self, subject: &str, object: &str, source: &str) -> &mut Self {
        self.ops.push(WriteOp::SetInheritance {
            subject: subject.to_string(),
            object: object.to_string(),
            source: source.to_string(),
        });
        self
    }

    /// Add a delete inheritance operation
    pub fn delete_inheritance(&mut self, subject: &str, object: &str, source: &str) -> &mut Self {
        self.ops.push(WriteOp::DeleteInheritance {
            subject: subject.to_string(),
            object: object.to_string(),
            source: source.to_string(),
        });
        self
    }

    /// Add a capability label operation
    pub fn set_cap_label(&mut self, entity: &str, cap_bit: u64, label: &str) -> &mut Self {
        self.ops.push(WriteOp::SetCapLabel {
            entity: entity.to_string(),
            cap_bit,
            label: label.to_string(),
        });
        self
    }

    /// Get the number of operations in the batch
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Clear all operations from the batch
    pub fn clear(&mut self) {
        self.ops.clear();
    }

    /// Execute all operations in a single transaction
    /// Returns the epoch timestamp of the transaction
    pub fn execute(&self) -> Result<u64> {
        if self.ops.is_empty() {
            return Ok(current_epoch());
        }

        let env = get_env()?;
        let dbs = get_dbs()?;
        let mut wtxn = env.write_txn().map_err(|e| CapbitError::from(e.to_string()))?;
        let epoch = current_epoch();

        for op in &self.ops {
            match op {
                WriteOp::SetRelationship { subject, rel_type, object } => {
                    let subj_esc = escape_key_part(subject);
                    let rel_esc = escape_key_part(rel_type);
                    let obj_esc = escape_key_part(object);

                    let forward_key = format!("{}/{}/{}", subj_esc, rel_esc, obj_esc);
                    let reverse_key = format!("{}/{}/{}", obj_esc, rel_esc, subj_esc);

                    dbs.relationships
                        .put(&mut wtxn, &forward_key, &epoch)
                        .map_err(|e| CapbitError::from(e.to_string()))?;
                    dbs.relationships_rev
                        .put(&mut wtxn, &reverse_key, &epoch)
                        .map_err(|e| CapbitError::from(e.to_string()))?;
                }
                WriteOp::DeleteRelationship { subject, rel_type, object } => {
                    let subj_esc = escape_key_part(subject);
                    let rel_esc = escape_key_part(rel_type);
                    let obj_esc = escape_key_part(object);

                    let forward_key = format!("{}/{}/{}", subj_esc, rel_esc, obj_esc);
                    let reverse_key = format!("{}/{}/{}", obj_esc, rel_esc, subj_esc);

                    dbs.relationships
                        .delete(&mut wtxn, &forward_key)
                        .map_err(|e| CapbitError::from(e.to_string()))?;
                    dbs.relationships_rev
                        .delete(&mut wtxn, &reverse_key)
                        .map_err(|e| CapbitError::from(e.to_string()))?;
                }
                WriteOp::SetCapability { entity, rel_type, cap_mask } => {
                    let ent_esc = escape_key_part(entity);
                    let rel_esc = escape_key_part(rel_type);
                    let key = format!("{}/{}", ent_esc, rel_esc);

                    dbs.capabilities
                        .put(&mut wtxn, &key, cap_mask)
                        .map_err(|e| CapbitError::from(e.to_string()))?;
                }
                WriteOp::SetInheritance { subject, object, source } => {
                    let subj_esc = escape_key_part(subject);
                    let obj_esc = escape_key_part(object);
                    let src_esc = escape_key_part(source);

                    let by_subject_key = format!("{}/{}/{}", subj_esc, obj_esc, src_esc);
                    let by_source_key = format!("{}/{}/{}", src_esc, obj_esc, subj_esc);
                    let by_object_key = format!("{}/{}/{}", obj_esc, src_esc, subj_esc);

                    dbs.inheritance
                        .put(&mut wtxn, &by_subject_key, &epoch)
                        .map_err(|e| CapbitError::from(e.to_string()))?;
                    dbs.inheritance_by_source
                        .put(&mut wtxn, &by_source_key, &epoch)
                        .map_err(|e| CapbitError::from(e.to_string()))?;
                    dbs.inheritance_by_object
                        .put(&mut wtxn, &by_object_key, &epoch)
                        .map_err(|e| CapbitError::from(e.to_string()))?;
                }
                WriteOp::DeleteInheritance { subject, object, source } => {
                    let subj_esc = escape_key_part(subject);
                    let obj_esc = escape_key_part(object);
                    let src_esc = escape_key_part(source);

                    let by_subject_key = format!("{}/{}/{}", subj_esc, obj_esc, src_esc);
                    let by_source_key = format!("{}/{}/{}", src_esc, obj_esc, subj_esc);
                    let by_object_key = format!("{}/{}/{}", obj_esc, src_esc, subj_esc);

                    dbs.inheritance
                        .delete(&mut wtxn, &by_subject_key)
                        .map_err(|e| CapbitError::from(e.to_string()))?;
                    dbs.inheritance_by_source
                        .delete(&mut wtxn, &by_source_key)
                        .map_err(|e| CapbitError::from(e.to_string()))?;
                    dbs.inheritance_by_object
                        .delete(&mut wtxn, &by_object_key)
                        .map_err(|e| CapbitError::from(e.to_string()))?;
                }
                WriteOp::SetCapLabel { entity, cap_bit, label } => {
                    let ent_esc = escape_key_part(entity);
                    let key = format!("{}/{:016x}", ent_esc, cap_bit);

                    dbs.cap_labels
                        .put(&mut wtxn, &key, label.as_str())
                        .map_err(|e| CapbitError::from(e.to_string()))?;
                }
            }
        }

        wtxn.commit().map_err(|e| CapbitError::from(e.to_string()))?;
        Ok(epoch)
    }
}

/// Create a new WriteBatch (convenience function)
pub fn write_batch() -> WriteBatch {
    WriteBatch::new()
}
