//! Capbit - Minimal capability-based access control

use std::path::Path;
use std::sync::{Mutex, OnceLock};
use heed::{Database, Env, EnvOpenOptions, RoTxn, RwTxn};
use heed::types::{Bytes, Str, U64};

// Types
#[derive(Debug, Clone)]
pub struct CapbitError(pub String);
impl std::fmt::Display for CapbitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
}
impl std::error::Error for CapbitError {}
pub type Result<T> = std::result::Result<T, CapbitError>;
fn err<E: std::error::Error>(e: E) -> CapbitError { CapbitError(e.to_string()) }

// Database
type Db = Database<Bytes, U64<byteorder::BigEndian>>;
struct Dbs { caps: Db, rev: Db, roles: Db, inh: Db, meta: Database<Str, Str> }

static ENV: OnceLock<Env> = OnceLock::new();
static DBS: OnceLock<Dbs> = OnceLock::new();
static TEST_LOCK: Mutex<()> = Mutex::new(());

#[inline] pub fn key(a: u64, b: u64) -> [u8; 16] {
    let mut k = [0u8; 16];
    k[..8].copy_from_slice(&a.to_be_bytes());
    k[8..].copy_from_slice(&b.to_be_bytes());
    k
}

fn dbs() -> Result<&'static Dbs> { DBS.get().ok_or_else(|| CapbitError("Not initialized".into())) }
fn env() -> Result<&'static Env> { ENV.get().ok_or_else(|| CapbitError("Not initialized".into())) }

fn read<T, F: FnOnce(&Dbs, &RoTxn) -> Result<T>>(f: F) -> Result<T> {
    f(dbs()?, &env()?.read_txn().map_err(err)?)
}
fn write<T, F: FnOnce(&Dbs, &mut RwTxn) -> Result<T>>(f: F) -> Result<T> {
    let mut txn = env()?.write_txn().map_err(err)?;
    let r = f(dbs()?, &mut txn)?;
    txn.commit().map_err(err)?;
    Ok(r)
}

// Generic table ops
fn tget(tx: &RoTxn, db: &Db, k: &[u8]) -> Result<u64> { Ok(db.get(tx, k).map_err(err)?.unwrap_or(0)) }
fn tput(tx: &mut RwTxn, db: &Db, k: &[u8], v: u64) -> Result<()> { db.put(tx, k, &v).map_err(err) }
fn tdel(tx: &mut RwTxn, db: &Db, k: &[u8]) -> Result<bool> { db.delete(tx, k).map_err(err) }
fn tlist(tx: &RoTxn, db: &Db, pfx: u64) -> Result<Vec<(u64, u64)>> {
    let mut r = Vec::new();
    for item in db.prefix_iter(tx, &pfx.to_be_bytes()).map_err(err)? {
        let (k, v) = item.map_err(err)?;
        if k.len() == 16 { r.push((u64::from_be_bytes(k[8..16].try_into().unwrap()), v)); }
    }
    Ok(r)
}

// Init
pub fn init(path: &str) -> Result<()> {
    if ENV.get().is_some() { return Ok(()); }
    std::fs::create_dir_all(path).map_err(err)?;
    let e = unsafe { EnvOpenOptions::new().map_size(1<<30).max_dbs(5).open(Path::new(path)).map_err(err)? };
    let mut tx = e.write_txn().map_err(err)?;
    let d = Dbs {
        caps: e.create_database(&mut tx, Some("caps")).map_err(err)?,
        rev: e.create_database(&mut tx, Some("rev")).map_err(err)?,
        roles: e.create_database(&mut tx, Some("roles")).map_err(err)?,
        inh: e.create_database(&mut tx, Some("inh")).map_err(err)?,
        meta: e.create_database(&mut tx, Some("meta")).map_err(err)?,
    };
    tx.commit().map_err(err)?;
    let _ = (ENV.set(e), DBS.set(d));
    Ok(())
}

pub fn clear_all() -> Result<()> {
    write(|d, tx| { d.caps.clear(tx).map_err(err)?; d.rev.clear(tx).map_err(err)?; d.roles.clear(tx).map_err(err)?; d.inh.clear(tx).map_err(err)?; d.meta.clear(tx).map_err(err) })
}
pub fn test_lock() -> std::sync::MutexGuard<'static, ()> { TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner()) }

// Core ops
pub fn grant(subject: u64, object: u64, role: u64) -> Result<()> {
    write(|d, tx| {
        let k = key(subject, object);
        let v = tget(tx, &d.caps, &k)? | role;
        tput(tx, &d.caps, &k, v)?;
        tput(tx, &d.rev, &key(object, subject), v)
    })
}

pub fn revoke(subject: u64, object: u64) -> Result<bool> {
    write(|d, tx| { let r = tdel(tx, &d.caps, &key(subject, object))?; tdel(tx, &d.rev, &key(object, subject))?; tdel(tx, &d.inh, &key(object, subject))?; Ok(r) })
}

pub fn batch_grant(grants: &[(u64, u64, u64)]) -> Result<()> {
    write(|d, tx| { for &(s, o, r) in grants { let k = key(s, o); let v = tget(tx, &d.caps, &k)? | r; tput(tx, &d.caps, &k, v)?; tput(tx, &d.rev, &key(o, s), v)?; } Ok(()) })
}

pub fn batch_revoke(revokes: &[(u64, u64)]) -> Result<usize> {
    write(|d, tx| { let mut c = 0; for &(s, o) in revokes { if tdel(tx, &d.caps, &key(s, o))? { c += 1; } tdel(tx, &d.rev, &key(o, s))?; } Ok(c) })
}

fn resolve(d: &Dbs, tx: &RoTxn, s: u64, o: u64, depth: u8) -> Result<u64> {
    if depth > 10 { return Ok(0); }
    let role = tget(tx, &d.caps, &key(s, o))?;
    let mut mask = if role == 0 { 0 } else { d.roles.get(tx, &key(o, role)).map_err(err)?.unwrap_or(role) };
    if let Some(p) = d.inh.get(tx, &key(o, s)).map_err(err)? { mask |= resolve(d, tx, p, o, depth + 1)?; }
    Ok(mask)
}

pub fn get_mask(subject: u64, object: u64) -> Result<u64> { read(|d, tx| resolve(d, tx, subject, object, 0)) }
pub fn get_role_id(subject: u64, object: u64) -> Result<u64> { read(|d, tx| tget(tx, &d.caps, &key(subject, object))) }
pub fn check(subject: u64, object: u64, required: u64) -> Result<bool> { Ok((get_mask(subject, object)? & required) == required) }

// Validation - prevents self-reference and cycles
fn no_cycle(d: &Dbs, tx: &RoTxn, obj: u64, from: u64, to: u64) -> Result<()> {
    if from == to { return Err(CapbitError("Cannot reference self".into())); }
    let mut cur = to;
    for _ in 0..10 { match d.inh.get(tx, &key(obj, cur)).map_err(err)? { Some(p) if p == from => return Err(CapbitError("Circular reference".into())), Some(p) => cur = p, None => break } }
    Ok(())
}

// Roles & Inheritance
pub fn set_role(object: u64, role: u64, mask: u64) -> Result<()> { write(|d, tx| tput(tx, &d.roles, &key(object, role), mask)) }
pub fn get_role(object: u64, role: u64) -> Result<u64> { read(|d, tx| Ok(d.roles.get(tx, &key(object, role)).map_err(err)?.unwrap_or(role))) }
pub fn set_inherit(object: u64, child: u64, parent: u64) -> Result<()> {
    read(|d, tx| no_cycle(d, tx, object, child, parent))?;
    write(|d, tx| tput(tx, &d.inh, &key(object, child), parent))
}
pub fn remove_inherit(object: u64, child: u64) -> Result<bool> { write(|d, tx| tdel(tx, &d.inh, &key(object, child))) }
pub fn get_inherit(object: u64, child: u64) -> Result<Option<u64>> { read(|d, tx| Ok(d.inh.get(tx, &key(object, child)).map_err(err)?)) }

pub fn list_for_subject(subject: u64) -> Result<Vec<(u64, u64)>> { read(|d, tx| tlist(tx, &d.caps, subject)) }
pub fn list_for_object(object: u64) -> Result<Vec<(u64, u64)>> { read(|d, tx| tlist(tx, &d.rev, object)) }

// Capability bits
pub const READ: u64 = 1;
pub const WRITE: u64 = 1 << 1;
pub const DELETE: u64 = 1 << 2;
pub const CREATE: u64 = 1 << 3;
pub const GRANT: u64 = 1 << 4;
pub const EXECUTE: u64 = 1 << 5;
pub const VIEW: u64 = 1 << 62;
pub const ADMIN: u64 = 1 << 63;

const CAPS: &[(&str, u64)] = &[("read",READ),("write",WRITE),("delete",DELETE),("create",CREATE),("grant",GRANT),("execute",EXECUTE),("view",VIEW),("admin",ADMIN)];
pub fn caps_to_names(mask: u64) -> Vec<&'static str> { CAPS.iter().filter(|(_, b)| mask & b == *b).map(|(n, _)| *n).collect() }
pub fn names_to_caps(names: &[&str]) -> u64 { names.iter().filter_map(|n| CAPS.iter().find(|(k, _)| k == n).map(|(_, v)| v)).fold(0, |a, b| a | b) }

// Protected ops
fn require(actor: u64, object: u64, req: u64) -> Result<()> {
    if get_root()?.map(|r| r == actor).unwrap_or(false) { return Ok(()); }
    if (get_mask(actor, object)? & req) != req { return Err(CapbitError(format!("{} lacks {:x} on {}", actor, req, object))); }
    Ok(())
}

pub fn protected_grant(actor: u64, subject: u64, object: u64, role: u64) -> Result<()> {
    if get_root()?.map(|r| r == actor).unwrap_or(false) { return grant(subject, object, role); }
    let m = get_mask(actor, object)?;
    if (m & ADMIN) == 0 && (m & role) != role { return Err(CapbitError(format!("{} cannot grant {:x}", actor, role))); }
    grant(subject, object, role)
}
pub fn protected_revoke(actor: u64, subject: u64, object: u64) -> Result<bool> { require(actor, object, ADMIN)?; revoke(subject, object) }
pub fn protected_set_role(actor: u64, object: u64, role: u64, mask: u64) -> Result<()> { require(actor, object, ADMIN)?; set_role(object, role, mask) }
pub fn protected_set_inherit(actor: u64, object: u64, child: u64, parent: u64) -> Result<()> { require(actor, object, ADMIN)?; set_inherit(object, child, parent) }
pub fn protected_remove_inherit(actor: u64, object: u64, child: u64) -> Result<bool> { require(actor, object, ADMIN)?; remove_inherit(object, child) }
pub fn protected_list_for_object(actor: u64, object: u64) -> Result<Vec<(u64, u64)>> { require(actor, object, VIEW)?; list_for_object(object) }

// Bootstrap
pub fn is_bootstrapped() -> Result<bool> { read(|d, tx| Ok(d.meta.get(tx, "boot").map_err(err)?.is_some())) }
pub fn get_root() -> Result<Option<u64>> { read(|d, tx| Ok(d.meta.get(tx, "root").map_err(err)?.and_then(|s| s.parse().ok()))) }
pub fn bootstrap(root: u64) -> Result<()> {
    if is_bootstrapped()? { return Err(CapbitError("Already bootstrapped".into())); }
    write(|d, tx| { tput(tx, &d.caps, &key(root, root), ADMIN)?; tput(tx, &d.rev, &key(root, root), ADMIN)?; d.meta.put(tx, "boot", "1").map_err(err)?; d.meta.put(tx, "root", &root.to_string()).map_err(err) })
}

// Labels & Entities
pub fn create_entity(name: &str) -> Result<u64> {
    write(|d, tx| {
        let id = d.meta.get(tx, "next_id").map_err(err)?.and_then(|s| s.parse().ok()).unwrap_or(1u64);
        d.meta.put(tx, "next_id", &(id + 1).to_string()).map_err(err)?;
        d.meta.put(tx, &format!("l:{}", id), name).map_err(err)?;
        d.meta.put(tx, &format!("n:{}", name), &id.to_string()).map_err(err)?;
        Ok(id)
    })
}
pub fn rename_entity(id: u64, new_name: &str) -> Result<()> {
    write(|d, tx| {
        if let Some(old) = d.meta.get(tx, &format!("l:{}", id)).map_err(err)? {
            d.meta.delete(tx, &format!("n:{}", old)).map_err(err)?;
        }
        d.meta.put(tx, &format!("l:{}", id), new_name).map_err(err)?;
        d.meta.put(tx, &format!("n:{}", new_name), &id.to_string()).map_err(err)
    })
}
pub fn delete_entity(id: u64) -> Result<bool> {
    write(|d, tx| {
        if let Some(name) = d.meta.get(tx, &format!("l:{}", id)).map_err(err)? {
            d.meta.delete(tx, &format!("n:{}", name)).map_err(err)?;
        }
        d.meta.delete(tx, &format!("l:{}", id)).map_err(err)
    })
}
pub fn set_label(id: u64, name: &str) -> Result<()> {
    write(|d, tx| { d.meta.put(tx, &format!("l:{}", id), name).map_err(err)?; d.meta.put(tx, &format!("n:{}", name), &id.to_string()).map_err(err) })
}
pub fn get_label(id: u64) -> Result<Option<String>> { read(|d, tx| Ok(d.meta.get(tx, &format!("l:{}", id)).map_err(err)?.map(|s| s.to_string()))) }
pub fn get_id_by_label(name: &str) -> Result<Option<u64>> { read(|d, tx| Ok(d.meta.get(tx, &format!("n:{}", name)).map_err(err)?.and_then(|s| s.parse().ok()))) }
pub fn list_labels() -> Result<Vec<(u64, String)>> {
    read(|d, tx| {
        let mut r = Vec::new();
        for item in d.meta.prefix_iter(tx, "l:").map_err(err)? {
            let (k, v) = item.map_err(err)?;
            if let Some(id) = k.strip_prefix("l:").and_then(|s| s.parse().ok()) { r.push((id, v.to_string())); }
        }
        Ok(r)
    })
}
