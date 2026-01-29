//! Capbit: Entity-Relationship Bitmask Access Control System
//!
//! A minimal, high-performance access control system where everything is an entity,
//! relationships are bitmasks, and capability semantics are defined per-entity.
//!
//! ## Path Patterns
//!
//! Six patterns define the entire system:
//!
//! | Pattern | Purpose |
//! |---------|---------|
//! | `entity/rel_mask/entity` | Relationship between entities |
//! | `entity/policy/entity` | Conditional relationship (code outputs rel_mask) |
//! | `entity/rel_mask/cap_mask` | Capability definition (per-entity) |
//! | `entity/entity/entity` | Inheritance reference |
//! | `entity/rel_mask/label` | Human-readable relationship name |
//! | `entity/cap_mask/label` | Human-readable capability name |

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::path::Path;
use std::sync::OnceLock;

use heed::types::*;
use heed::{Database, Env, EnvOpenOptions};

/// Global environment singleton
static ENV: OnceLock<Env> = OnceLock::new();

/// Sub-databases for the six path patterns + reverse indices
struct Databases {
    /// entity/rel_mask/entity -> epoch (relationships)
    relationships: Database<Str, U64<byteorder::BigEndian>>,
    /// Reverse: entity/rel_mask/entity (for "who has access to X" queries)
    relationships_rev: Database<Str, U64<byteorder::BigEndian>>,
    /// entity/policy/entity -> epoch (conditional relationships)
    policies: Database<Str, U64<byteorder::BigEndian>>,
    /// Reverse policies
    policies_rev: Database<Str, U64<byteorder::BigEndian>>,
    /// entity/rel_mask/cap_mask -> epoch (capability definitions)
    capabilities: Database<Str, U64<byteorder::BigEndian>>,
    /// entity/entity/entity -> epoch (inheritance)
    inheritance: Database<Str, U64<byteorder::BigEndian>>,
    /// Reverse inheritance
    inheritance_rev: Database<Str, U64<byteorder::BigEndian>>,
    /// Labels: entity/rel_mask/label and entity/cap_mask/label
    labels: Database<Str, Str>,
}

static DBS: OnceLock<Databases> = OnceLock::new();

/// Initialize the LMDB environment
#[napi]
pub fn init(db_path: String) -> Result<()> {
    let path = Path::new(&db_path);
    std::fs::create_dir_all(path).map_err(|e| Error::from_reason(e.to_string()))?;

    let env = unsafe {
        EnvOpenOptions::new()
            .map_size(1024 * 1024 * 1024) // 1GB
            .max_dbs(10)
            .open(path)
            .map_err(|e| Error::from_reason(e.to_string()))?
    };

    let mut wtxn = env.write_txn().map_err(|e| Error::from_reason(e.to_string()))?;

    let dbs = Databases {
        relationships: env
            .create_database(&mut wtxn, Some("relationships"))
            .map_err(|e| Error::from_reason(e.to_string()))?,
        relationships_rev: env
            .create_database(&mut wtxn, Some("relationships_rev"))
            .map_err(|e| Error::from_reason(e.to_string()))?,
        policies: env
            .create_database(&mut wtxn, Some("policies"))
            .map_err(|e| Error::from_reason(e.to_string()))?,
        policies_rev: env
            .create_database(&mut wtxn, Some("policies_rev"))
            .map_err(|e| Error::from_reason(e.to_string()))?,
        capabilities: env
            .create_database(&mut wtxn, Some("capabilities"))
            .map_err(|e| Error::from_reason(e.to_string()))?,
        inheritance: env
            .create_database(&mut wtxn, Some("inheritance"))
            .map_err(|e| Error::from_reason(e.to_string()))?,
        inheritance_rev: env
            .create_database(&mut wtxn, Some("inheritance_rev"))
            .map_err(|e| Error::from_reason(e.to_string()))?,
        labels: env
            .create_database(&mut wtxn, Some("labels"))
            .map_err(|e| Error::from_reason(e.to_string()))?,
    };

    wtxn.commit().map_err(|e| Error::from_reason(e.to_string()))?;

    ENV.set(env).map_err(|_| Error::from_reason("Environment already initialized"))?;
    DBS.set(dbs).map_err(|_| Error::from_reason("Databases already initialized"))?;

    Ok(())
}

fn get_env() -> Result<&'static Env> {
    ENV.get().ok_or_else(|| Error::from_reason("Database not initialized. Call init() first."))
}

fn get_dbs() -> Result<&'static Databases> {
    DBS.get().ok_or_else(|| Error::from_reason("Database not initialized. Call init() first."))
}

fn current_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

// ============================================================================
// Relationship Operations (entity/rel_mask/entity)
// ============================================================================

/// Set a relationship between two entities
/// Path: subject/rel_mask/object -> epoch
#[napi]
pub fn set_relationship(subject: String, rel_mask: i64, object: String) -> Result<i64> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let mut wtxn = env.write_txn().map_err(|e| Error::from_reason(e.to_string()))?;

    let epoch = current_epoch();
    let forward_key = format!("{}/{:016x}/{}", subject, rel_mask as u64, object);
    let reverse_key = format!("{}/{:016x}/{}", object, rel_mask as u64, subject);

    dbs.relationships
        .put(&mut wtxn, &forward_key, &epoch)
        .map_err(|e| Error::from_reason(e.to_string()))?;
    dbs.relationships_rev
        .put(&mut wtxn, &reverse_key, &epoch)
        .map_err(|e| Error::from_reason(e.to_string()))?;

    wtxn.commit().map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(epoch as i64)
}

/// Get all relationship masks between subject and object
#[napi]
pub fn get_relationships(subject: String, object: String) -> Result<Vec<i64>> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let rtxn = env.read_txn().map_err(|e| Error::from_reason(e.to_string()))?;

    let prefix = format!("{}/", subject);
    let suffix = format!("/{}", object);
    let mut results = Vec::new();

    let iter = dbs
        .relationships
        .prefix_iter(&rtxn, &prefix)
        .map_err(|e| Error::from_reason(e.to_string()))?;

    for item in iter {
        let (key, _epoch) = item.map_err(|e| Error::from_reason(e.to_string()))?;
        if key.ends_with(&suffix) {
            // Extract rel_mask from key: subject/rel_mask/object
            let parts: Vec<&str> = key.split('/').collect();
            if parts.len() == 3 {
                if let Ok(mask) = u64::from_str_radix(parts[1], 16) {
                    results.push(mask as i64);
                }
            }
        }
    }

    Ok(results)
}

/// Delete a relationship
#[napi]
pub fn delete_relationship(subject: String, rel_mask: i64, object: String) -> Result<bool> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let mut wtxn = env.write_txn().map_err(|e| Error::from_reason(e.to_string()))?;

    let forward_key = format!("{}/{:016x}/{}", subject, rel_mask as u64, object);
    let reverse_key = format!("{}/{:016x}/{}", object, rel_mask as u64, subject);

    let deleted = dbs
        .relationships
        .delete(&mut wtxn, &forward_key)
        .map_err(|e| Error::from_reason(e.to_string()))?;
    dbs.relationships_rev
        .delete(&mut wtxn, &reverse_key)
        .map_err(|e| Error::from_reason(e.to_string()))?;

    wtxn.commit().map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(deleted)
}

// ============================================================================
// Capability Operations (entity/rel_mask/cap_mask)
// ============================================================================

/// Define what capabilities a relationship grants on an entity
/// Path: entity/rel_mask/cap_mask -> epoch
#[napi]
pub fn set_capability(entity: String, rel_mask: i64, cap_mask: i64) -> Result<i64> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let mut wtxn = env.write_txn().map_err(|e| Error::from_reason(e.to_string()))?;

    let epoch = current_epoch();
    let key = format!("{}/{:016x}/{:016x}", entity, rel_mask as u64, cap_mask as u64);

    dbs.capabilities
        .put(&mut wtxn, &key, &epoch)
        .map_err(|e| Error::from_reason(e.to_string()))?;

    wtxn.commit().map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(epoch as i64)
}

/// Get capability mask for a relationship on an entity
#[napi]
pub fn get_capability(entity: String, rel_mask: i64) -> Result<Option<i64>> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let rtxn = env.read_txn().map_err(|e| Error::from_reason(e.to_string()))?;

    let prefix = format!("{}/{:016x}/", entity, rel_mask as u64);

    let iter = dbs
        .capabilities
        .prefix_iter(&rtxn, &prefix)
        .map_err(|e| Error::from_reason(e.to_string()))?;

    for item in iter {
        let (key, _epoch) = item.map_err(|e| Error::from_reason(e.to_string()))?;
        let parts: Vec<&str> = key.split('/').collect();
        if parts.len() == 3 {
            if let Ok(cap) = u64::from_str_radix(parts[2], 16) {
                return Ok(Some(cap as i64));
            }
        }
    }

    Ok(None)
}

// ============================================================================
// Inheritance Operations (entity/entity/entity)
// ============================================================================

/// Set inheritance: subject inherits source's relationship to object
/// Path: subject/object/source -> epoch
#[napi]
pub fn set_inheritance(subject: String, object: String, source: String) -> Result<i64> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let mut wtxn = env.write_txn().map_err(|e| Error::from_reason(e.to_string()))?;

    let epoch = current_epoch();
    let forward_key = format!("{}/{}/{}", subject, object, source);
    let reverse_key = format!("{}/{}/{}", source, object, subject);

    dbs.inheritance
        .put(&mut wtxn, &forward_key, &epoch)
        .map_err(|e| Error::from_reason(e.to_string()))?;
    dbs.inheritance_rev
        .put(&mut wtxn, &reverse_key, &epoch)
        .map_err(|e| Error::from_reason(e.to_string()))?;

    wtxn.commit().map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(epoch as i64)
}

/// Get inheritance sources for subject's relationship to object
#[napi]
pub fn get_inheritance(subject: String, object: String) -> Result<Vec<String>> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let rtxn = env.read_txn().map_err(|e| Error::from_reason(e.to_string()))?;

    let prefix = format!("{}/{}/", subject, object);
    let mut results = Vec::new();

    let iter = dbs
        .inheritance
        .prefix_iter(&rtxn, &prefix)
        .map_err(|e| Error::from_reason(e.to_string()))?;

    for item in iter {
        let (key, _epoch) = item.map_err(|e| Error::from_reason(e.to_string()))?;
        let parts: Vec<&str> = key.split('/').collect();
        if parts.len() == 3 {
            results.push(parts[2].to_string());
        }
    }

    Ok(results)
}

// ============================================================================
// Label Operations
// ============================================================================

/// Set a label for a relationship mask on an entity
#[napi]
pub fn set_rel_label(entity: String, rel_mask: i64, label: String) -> Result<()> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let mut wtxn = env.write_txn().map_err(|e| Error::from_reason(e.to_string()))?;

    let key = format!("{}/rel/{:016x}", entity, rel_mask as u64);

    dbs.labels
        .put(&mut wtxn, &key, &label)
        .map_err(|e| Error::from_reason(e.to_string()))?;

    wtxn.commit().map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(())
}

/// Set a label for a capability mask on an entity
#[napi]
pub fn set_cap_label(entity: String, cap_mask: i64, label: String) -> Result<()> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let mut wtxn = env.write_txn().map_err(|e| Error::from_reason(e.to_string()))?;

    let key = format!("{}/cap/{:016x}", entity, cap_mask as u64);

    dbs.labels
        .put(&mut wtxn, &key, &label)
        .map_err(|e| Error::from_reason(e.to_string()))?;

    wtxn.commit().map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(())
}

// ============================================================================
// Access Evaluation
// ============================================================================

/// Check if subject can perform action on object
/// Returns the effective capability mask (0 if no access)
#[napi]
pub fn check_access(subject: String, object: String, max_depth: Option<i32>) -> Result<i64> {
    let env = get_env()?;
    let dbs = get_dbs()?;
    let rtxn = env.read_txn().map_err(|e| Error::from_reason(e.to_string()))?;
    let depth_limit = max_depth.unwrap_or(3) as usize;

    let mut effective_cap: u64 = 0;

    // Step 1: Get direct relationships
    let prefix = format!("{}/", subject);
    let suffix = format!("/{}", object);

    let iter = dbs
        .relationships
        .prefix_iter(&rtxn, &prefix)
        .map_err(|e| Error::from_reason(e.to_string()))?;

    for item in iter {
        let (key, _epoch) = item.map_err(|e| Error::from_reason(e.to_string()))?;
        if key.ends_with(&suffix) {
            let parts: Vec<&str> = key.split('/').collect();
            if parts.len() == 3 {
                if let Ok(rel_mask) = u64::from_str_radix(parts[1], 16) {
                    // Step 2: Get capabilities for this relationship on object
                    let cap_prefix = format!("{}/{:016x}/", object, rel_mask);
                    let cap_iter = dbs
                        .capabilities
                        .prefix_iter(&rtxn, &cap_prefix)
                        .map_err(|e| Error::from_reason(e.to_string()))?;

                    for cap_item in cap_iter {
                        let (cap_key, _) = cap_item.map_err(|e| Error::from_reason(e.to_string()))?;
                        let cap_parts: Vec<&str> = cap_key.split('/').collect();
                        if cap_parts.len() == 3 {
                            if let Ok(cap) = u64::from_str_radix(cap_parts[2], 16) {
                                effective_cap |= cap;
                            }
                        }
                    }
                }
            }
        }
    }

    // Step 3: Check inheritance (depth-limited)
    if depth_limit > 0 {
        let inherit_prefix = format!("{}/{}/", subject, object);
        let inherit_iter = dbs
            .inheritance
            .prefix_iter(&rtxn, &inherit_prefix)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        for item in inherit_iter {
            let (key, _epoch) = item.map_err(|e| Error::from_reason(e.to_string()))?;
            let parts: Vec<&str> = key.split('/').collect();
            if parts.len() == 3 {
                let source = parts[2];
                // Recursively check source's access (simplified - should use proper recursion)
                let source_prefix = format!("{}/", source);
                let source_suffix = format!("/{}", object);

                let source_iter = dbs
                    .relationships
                    .prefix_iter(&rtxn, &source_prefix)
                    .map_err(|e| Error::from_reason(e.to_string()))?;

                for source_item in source_iter {
                    let (source_key, _) =
                        source_item.map_err(|e| Error::from_reason(e.to_string()))?;
                    if source_key.ends_with(&source_suffix) {
                        let source_parts: Vec<&str> = source_key.split('/').collect();
                        if source_parts.len() == 3 {
                            if let Ok(rel_mask) = u64::from_str_radix(source_parts[1], 16) {
                                let cap_prefix = format!("{}/{:016x}/", object, rel_mask);
                                let cap_iter = dbs
                                    .capabilities
                                    .prefix_iter(&rtxn, &cap_prefix)
                                    .map_err(|e| Error::from_reason(e.to_string()))?;

                                for cap_item in cap_iter {
                                    let (cap_key, _) =
                                        cap_item.map_err(|e| Error::from_reason(e.to_string()))?;
                                    let cap_parts: Vec<&str> = cap_key.split('/').collect();
                                    if cap_parts.len() == 3 {
                                        if let Ok(cap) = u64::from_str_radix(cap_parts[2], 16) {
                                            effective_cap |= cap;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(effective_cap as i64)
}

/// Check if subject has specific capability on object
#[napi]
pub fn has_capability(subject: String, object: String, required_cap: i64) -> Result<bool> {
    let effective = check_access(subject, object, None)?;
    Ok((effective & required_cap) == required_cap)
}

// ============================================================================
// Utility
// ============================================================================

/// Close the database (for cleanup)
#[napi]
pub fn close() -> Result<()> {
    // LMDB handles cleanup on drop
    Ok(())
}
