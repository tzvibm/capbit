//! Core database operations for Capbit
//!
//! High-performance access control with string-based relationships and bitmask capabilities.

use std::borrow::Cow;
use std::path::Path;
use std::sync::OnceLock;

use heed::types::*;
use heed::{Database, Env, EnvOpenOptions, RoTxn, RwTxn};
use serde::{Deserialize, Serialize};

// ============================================================================
// Database Setup
// ============================================================================

static ENV: OnceLock<Env> = OnceLock::new();

struct Databases {
    relationships: Database<Str, U64<byteorder::BigEndian>>,
    relationships_rev: Database<Str, U64<byteorder::BigEndian>>,
    capabilities: Database<Str, U64<byteorder::BigEndian>>,
    inheritance: Database<Str, U64<byteorder::BigEndian>>,
    inheritance_by_source: Database<Str, U64<byteorder::BigEndian>>,
    inheritance_by_object: Database<Str, U64<byteorder::BigEndian>>,
    cap_labels: Database<Str, Str>,
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

fn with_read_txn<T, F>(f: F) -> Result<T>
where
    F: FnOnce(&RoTxn, &Databases) -> Result<T>,
{
    let env = get_env()?;
    let dbs = get_dbs()?;
    let txn = env.read_txn().map_err(err)?;
    f(&txn, dbs)
}

fn with_write_txn<T, F>(f: F) -> Result<T>
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

fn current_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn escape(s: &str) -> Cow<'_, str> {
    if s.contains('/') || s.contains('\\') {
        Cow::Owned(s.replace('\\', "\\\\").replace('/', "\\/"))
    } else {
        Cow::Borrowed(s)
    }
}

fn unescape(s: &str) -> Cow<'_, str> {
    if s.contains('\\') {
        Cow::Owned(s.replace("\\/", "/").replace("\\\\", "\\"))
    } else {
        Cow::Borrowed(s)
    }
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
            .max_dbs(10)
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
    };

    wtxn.commit().map_err(err)?;
    let _ = ENV.set(env);
    let _ = DBS.set(dbs);
    Ok(())
}

pub fn close() {}

// ============================================================================
// Internal Operations (take transaction)
// ============================================================================

fn set_relationship_in(txn: &mut RwTxn, dbs: &Databases, subject: &str, rel_type: &str, object: &str) -> Result<u64> {
    let epoch = current_epoch();
    let (s, r, o) = (escape(subject), escape(rel_type), escape(object));

    dbs.relationships.put(txn, &format!("{}/{}/{}", s, r, o), &epoch).map_err(err)?;
    dbs.relationships_rev.put(txn, &format!("{}/{}/{}", o, r, s), &epoch).map_err(err)?;
    Ok(epoch)
}

fn delete_relationship_in(txn: &mut RwTxn, dbs: &Databases, subject: &str, rel_type: &str, object: &str) -> Result<bool> {
    let (s, r, o) = (escape(subject), escape(rel_type), escape(object));

    let deleted = dbs.relationships.delete(txn, &format!("{}/{}/{}", s, r, o)).map_err(err)?;
    dbs.relationships_rev.delete(txn, &format!("{}/{}/{}", o, r, s)).map_err(err)?;
    Ok(deleted)
}

fn set_capability_in(txn: &mut RwTxn, dbs: &Databases, entity: &str, rel_type: &str, cap_mask: u64) -> Result<u64> {
    let epoch = current_epoch();
    let key = format!("{}/{}", escape(entity), escape(rel_type));
    dbs.capabilities.put(txn, &key, &cap_mask).map_err(err)?;
    Ok(epoch)
}

fn set_inheritance_in(txn: &mut RwTxn, dbs: &Databases, subject: &str, object: &str, source: &str) -> Result<u64> {
    let epoch = current_epoch();
    let (subj, obj, src) = (escape(subject), escape(object), escape(source));

    dbs.inheritance.put(txn, &format!("{}/{}/{}", subj, obj, src), &epoch).map_err(err)?;
    dbs.inheritance_by_source.put(txn, &format!("{}/{}/{}", src, obj, subj), &epoch).map_err(err)?;
    dbs.inheritance_by_object.put(txn, &format!("{}/{}/{}", obj, src, subj), &epoch).map_err(err)?;
    Ok(epoch)
}

fn delete_inheritance_in(txn: &mut RwTxn, dbs: &Databases, subject: &str, object: &str, source: &str) -> Result<bool> {
    let (subj, obj, src) = (escape(subject), escape(object), escape(source));

    let deleted = dbs.inheritance.delete(txn, &format!("{}/{}/{}", subj, obj, src)).map_err(err)?;
    dbs.inheritance_by_source.delete(txn, &format!("{}/{}/{}", src, obj, subj)).map_err(err)?;
    dbs.inheritance_by_object.delete(txn, &format!("{}/{}/{}", obj, src, subj)).map_err(err)?;
    Ok(deleted)
}

fn set_cap_label_in(txn: &mut RwTxn, dbs: &Databases, entity: &str, cap_bit: u64, label: &str) -> Result<()> {
    let key = format!("{}/{:016x}", escape(entity), cap_bit);
    dbs.cap_labels.put(txn, &key, label).map_err(err)?;
    Ok(())
}

// ============================================================================
// Public API - Relationships
// ============================================================================

pub fn set_relationship(subject: &str, rel_type: &str, object: &str) -> Result<u64> {
    with_write_txn(|txn, dbs| set_relationship_in(txn, dbs, subject, rel_type, object))
}

pub fn get_relationships(subject: &str, object: &str) -> Result<Vec<String>> {
    with_read_txn(|txn, dbs| {
        let prefix = format!("{}/", escape(subject));
        let suffix = format!("/{}", escape(object));
        let mut results = Vec::new();

        for item in dbs.relationships.prefix_iter(txn, &prefix).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            if key.ends_with(&suffix) {
                let rel = &key[prefix.len()..key.len() - suffix.len()];
                results.push(unescape(rel).into_owned());
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
        let key = format!("{}/{}", escape(entity), escape(rel_type));
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
        let prefix = format!("{}/{}/", escape(subject), escape(object));
        let mut results = Vec::new();

        for item in dbs.inheritance.prefix_iter(txn, &prefix).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            results.push(unescape(&key[prefix.len()..]).into_owned());
        }
        Ok(results)
    })
}

pub fn delete_inheritance(subject: &str, object: &str, source: &str) -> Result<bool> {
    with_write_txn(|txn, dbs| delete_inheritance_in(txn, dbs, subject, object, source))
}

pub fn get_inheritors_from_source(source: &str, object: &str) -> Result<Vec<String>> {
    with_read_txn(|txn, dbs| {
        let prefix = format!("{}/{}/", escape(source), escape(object));
        let mut results = Vec::new();

        for item in dbs.inheritance_by_source.prefix_iter(txn, &prefix).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            results.push(unescape(&key[prefix.len()..]).into_owned());
        }
        Ok(results)
    })
}

pub fn get_inheritance_for_object(object: &str) -> Result<Vec<(String, String)>> {
    with_read_txn(|txn, dbs| {
        let prefix = format!("{}/", escape(object));
        let mut results = Vec::new();

        for item in dbs.inheritance_by_object.prefix_iter(txn, &prefix).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            let rest = &key[prefix.len()..];
            if let Some(pos) = rest.find('/') {
                let source = unescape(&rest[..pos]).into_owned();
                let subject = unescape(&rest[pos + 1..]).into_owned();
                results.push((source, subject));
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
        let key = format!("{}/{:016x}", escape(entity), cap_bit);
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
        let obj_esc = escape(object);

        while let Some((current, depth)) = stack.pop() {
            let visit_key = format!("{}:{}", current, object);
            if !visited.insert(visit_key) {
                continue;
            }

            let subj_esc = escape(&current);
            let prefix = format!("{}/", subj_esc);
            let suffix = format!("/{}", obj_esc);

            // Get direct relationships
            for item in dbs.relationships.prefix_iter(txn, &prefix).map_err(err)? {
                let (key, _) = item.map_err(err)?;
                if key.ends_with(&suffix) {
                    let rel_esc = &key[prefix.len()..key.len() - suffix.len()];
                    let cap_key = format!("{}/{}", obj_esc, rel_esc);
                    if let Some(cap) = dbs.capabilities.get(txn, &cap_key).map_err(err)? {
                        effective_cap |= cap;
                    }
                }
            }

            // Get inheritance sources
            if depth < depth_limit {
                let inherit_prefix = format!("{}/{}/", subj_esc, obj_esc);
                for item in dbs.inheritance.prefix_iter(txn, &inherit_prefix).map_err(err)? {
                    let (key, _) = item.map_err(err)?;
                    let source = unescape(&key[inherit_prefix.len()..]).into_owned();
                    stack.push((source, depth + 1));
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

pub fn list_accessible(subject: &str) -> Result<Vec<(String, String)>> {
    with_read_txn(|txn, dbs| {
        let prefix = format!("{}/", escape(subject));
        let mut results = Vec::new();

        for item in dbs.relationships.prefix_iter(txn, &prefix).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            let rest = &key[prefix.len()..];
            if let Some(pos) = rest.rfind('/') {
                let rel_type = unescape(&rest[..pos]).into_owned();
                let object = unescape(&rest[pos + 1..]).into_owned();
                results.push((object, rel_type));
            }
        }
        Ok(results)
    })
}

pub fn list_subjects(object: &str) -> Result<Vec<(String, String)>> {
    with_read_txn(|txn, dbs| {
        let prefix = format!("{}/", escape(object));
        let mut results = Vec::new();

        for item in dbs.relationships_rev.prefix_iter(txn, &prefix).map_err(err)? {
            let (key, _) = item.map_err(err)?;
            let rest = &key[prefix.len()..];
            if let Some(pos) = rest.rfind('/') {
                let rel_type = unescape(&rest[..pos]).into_owned();
                let subject = unescape(&rest[pos + 1..]).into_owned();
                results.push((subject, rel_type));
            }
        }
        Ok(results)
    })
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
