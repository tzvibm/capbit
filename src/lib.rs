//! Capbit: Entity-Relationship Access Control System
//!
//! A minimal, high-performance access control system where:
//! - Everything is an entity (opaque IDs)
//! - Relationships are strings (unlimited types: "editor", "viewer", etc.)
//! - Capabilities are bitmasks (O(1) evaluation)
//!
//! ## Storage Patterns
//!
//! | Pattern | Purpose |
//! |---------|---------|
//! | `subject/rel_type/object` | Relationship between entities |
//! | `object/rel_type` → cap_mask | Capability definition |
//! | `subject/object/source` | Inheritance reference |

pub mod core;

use napi::bindgen_prelude::*;
use napi_derive::napi;

// ============================================================================
// NAPI Bindings - Thin wrappers around core module
// ============================================================================

fn to_napi_error(e: core::CapbitError) -> Error {
    Error::from_reason(e.message)
}

/// Initialize the LMDB environment
#[napi]
pub fn init(db_path: String) -> Result<()> {
    core::init(&db_path).map_err(to_napi_error)
}

// ============================================================================
// Relationship Operations
// ============================================================================

/// Set a relationship between two entities
/// rel_type is a string (e.g., "editor", "viewer", "member")
#[napi]
pub fn set_relationship(subject: String, rel_type: String, object: String) -> Result<i64> {
    core::set_relationship(&subject, &rel_type, &object)
        .map(|e| e as i64)
        .map_err(to_napi_error)
}

/// Get all relationship types between subject and object
/// Returns array of relation type strings (e.g., ["editor", "viewer"])
#[napi]
pub fn get_relationships(subject: String, object: String) -> Result<Vec<String>> {
    core::get_relationships(&subject, &object).map_err(to_napi_error)
}

/// Delete a relationship
#[napi]
pub fn delete_relationship(subject: String, rel_type: String, object: String) -> Result<bool> {
    core::delete_relationship(&subject, &rel_type, &object).map_err(to_napi_error)
}

// ============================================================================
// Capability Operations
// ============================================================================

/// Define what capabilities a relationship type grants on an entity
/// e.g., setCapability("slack", "editor", READ | WRITE | DELETE)
#[napi]
pub fn set_capability(entity: String, rel_type: String, cap_mask: i64) -> Result<i64> {
    core::set_capability(&entity, &rel_type, cap_mask as u64)
        .map(|e| e as i64)
        .map_err(to_napi_error)
}

/// Get capability mask for a relationship type on an entity
#[napi]
pub fn get_capability(entity: String, rel_type: String) -> Result<Option<i64>> {
    core::get_capability(&entity, &rel_type)
        .map(|o| o.map(|c| c as i64))
        .map_err(to_napi_error)
}

// ============================================================================
// Inheritance Operations
// ============================================================================

/// Set inheritance: subject inherits source's relationship to object
#[napi]
pub fn set_inheritance(subject: String, object: String, source: String) -> Result<i64> {
    core::set_inheritance(&subject, &object, &source)
        .map(|e| e as i64)
        .map_err(to_napi_error)
}

/// Get inheritance sources for subject's relationship to object
#[napi]
pub fn get_inheritance(subject: String, object: String) -> Result<Vec<String>> {
    core::get_inheritance(&subject, &object).map_err(to_napi_error)
}

/// Delete an inheritance rule
#[napi]
pub fn delete_inheritance(subject: String, object: String, source: String) -> Result<bool> {
    core::delete_inheritance(&subject, &object, &source).map_err(to_napi_error)
}

/// Get all subjects that inherit from source for a specific object
#[napi]
pub fn get_inheritors_from_source(source: String, object: String) -> Result<Vec<String>> {
    core::get_inheritors_from_source(&source, &object).map_err(to_napi_error)
}

/// Get all inheritance rules for an object
/// Returns array of [source, subject] pairs
#[napi]
pub fn get_inheritance_for_object(object: String) -> Result<Vec<Vec<String>>> {
    core::get_inheritance_for_object(&object)
        .map(|v| v.into_iter().map(|(src, subj)| vec![src, subj]).collect())
        .map_err(to_napi_error)
}

// ============================================================================
// Label Operations
// ============================================================================

/// Set a label for a capability bit on an entity
/// e.g., setCapLabel("myapp", 0x01, "read")
#[napi]
pub fn set_cap_label(entity: String, cap_bit: i64, label: String) -> Result<()> {
    core::set_cap_label(&entity, cap_bit as u64, &label).map_err(to_napi_error)
}

/// Get label for a capability bit
#[napi]
pub fn get_cap_label(entity: String, cap_bit: i64) -> Result<Option<String>> {
    core::get_cap_label(&entity, cap_bit as u64).map_err(to_napi_error)
}

// ============================================================================
// Access Evaluation
// ============================================================================

/// Check what capabilities subject has on object
/// Returns the effective capability mask (0 if no access)
#[napi]
pub fn check_access(subject: String, object: String, max_depth: Option<i32>) -> Result<i64> {
    core::check_access(&subject, &object, max_depth.map(|d| d as usize))
        .map(|c| c as i64)
        .map_err(to_napi_error)
}

/// Check if subject has specific capability on object
#[napi]
pub fn has_capability(subject: String, object: String, required_cap: i64) -> Result<bool> {
    core::has_capability(&subject, &object, required_cap as u64).map_err(to_napi_error)
}

// ============================================================================
// Batch Operations
// ============================================================================

/// Batch set relationships
/// Each entry is [subject, rel_type, object]
#[napi]
pub fn batch_set_relationships(entries: Vec<Vec<String>>) -> Result<i64> {
    let parsed: Vec<(String, String, String)> = entries
        .into_iter()
        .filter_map(|e| {
            if e.len() != 3 {
                return None;
            }
            Some((e[0].clone(), e[1].clone(), e[2].clone()))
        })
        .collect();

    core::batch_set_relationships(&parsed)
        .map(|c| c as i64)
        .map_err(to_napi_error)
}

/// Batch set capabilities
/// Each entry is [entity, rel_type, cap_mask]
#[napi]
pub fn batch_set_capabilities(entries: Vec<Vec<String>>) -> Result<i64> {
    let parsed: Vec<(String, String, u64)> = entries
        .into_iter()
        .filter_map(|e| {
            if e.len() != 3 {
                return None;
            }
            let cap_mask: u64 = e[2].parse().ok()?;
            Some((e[0].clone(), e[1].clone(), cap_mask))
        })
        .collect();

    core::batch_set_capabilities(&parsed)
        .map(|c| c as i64)
        .map_err(to_napi_error)
}

/// Batch set inheritance
/// Each entry is [subject, object, source]
#[napi]
pub fn batch_set_inheritance(entries: Vec<Vec<String>>) -> Result<i64> {
    let parsed: Vec<(String, String, String)> = entries
        .into_iter()
        .filter_map(|e| {
            if e.len() != 3 {
                return None;
            }
            Some((e[0].clone(), e[1].clone(), e[2].clone()))
        })
        .collect();

    core::batch_set_inheritance(&parsed)
        .map(|c| c as i64)
        .map_err(to_napi_error)
}

// ============================================================================
// Query Operations
// ============================================================================

/// List all entities that subject has any relationship with
/// Returns array of [object, rel_type] pairs
#[napi]
pub fn list_accessible(subject: String) -> Result<Vec<Vec<String>>> {
    core::list_accessible(&subject)
        .map(|v| v.into_iter().map(|(obj, rel)| vec![obj, rel]).collect())
        .map_err(to_napi_error)
}

/// List all subjects that have any relationship to object
/// Returns array of [subject, rel_type] pairs
#[napi]
pub fn list_subjects(object: String) -> Result<Vec<Vec<String>>> {
    core::list_subjects(&object)
        .map(|v| v.into_iter().map(|(subj, rel)| vec![subj, rel]).collect())
        .map_err(to_napi_error)
}

// ============================================================================
// Utility
// ============================================================================

/// Close the database (for cleanup)
#[napi]
pub fn close() -> Result<()> {
    core::close();
    Ok(())
}

// ============================================================================
// WriteBatch - Explicit Transaction API
// ============================================================================

/// A batch of write operations executed in a single transaction.
/// Provides atomicity and better performance for multiple writes.
#[napi]
pub struct WriteBatch {
    inner: core::WriteBatch,
}

#[napi]
impl WriteBatch {
    /// Create a new empty batch
    #[napi(constructor)]
    pub fn new() -> Self {
        Self { inner: core::WriteBatch::new() }
    }

    /// Add a relationship operation
    #[napi]
    pub fn set_relationship(&mut self, subject: String, rel_type: String, object: String) -> &Self {
        self.inner.set_relationship(&subject, &rel_type, &object);
        self
    }

    /// Add a delete relationship operation
    #[napi]
    pub fn delete_relationship(&mut self, subject: String, rel_type: String, object: String) -> &Self {
        self.inner.delete_relationship(&subject, &rel_type, &object);
        self
    }

    /// Add a capability operation
    #[napi]
    pub fn set_capability(&mut self, entity: String, rel_type: String, cap_mask: i64) -> &Self {
        self.inner.set_capability(&entity, &rel_type, cap_mask as u64);
        self
    }

    /// Add an inheritance operation
    #[napi]
    pub fn set_inheritance(&mut self, subject: String, object: String, source: String) -> &Self {
        self.inner.set_inheritance(&subject, &object, &source);
        self
    }

    /// Add a delete inheritance operation
    #[napi]
    pub fn delete_inheritance(&mut self, subject: String, object: String, source: String) -> &Self {
        self.inner.delete_inheritance(&subject, &object, &source);
        self
    }

    /// Add a capability label operation
    #[napi]
    pub fn set_cap_label(&mut self, entity: String, cap_bit: i64, label: String) -> &Self {
        self.inner.set_cap_label(&entity, cap_bit as u64, &label);
        self
    }

    /// Get the number of operations in the batch
    #[napi(getter)]
    pub fn length(&self) -> u32 {
        self.inner.len() as u32
    }

    /// Clear all operations from the batch
    #[napi]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Execute all operations in a single transaction
    /// Returns the epoch timestamp of the transaction
    #[napi]
    pub fn execute(&self) -> Result<i64> {
        self.inner.execute().map(|e| e as i64).map_err(to_napi_error)
    }
}
