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

// Database types
type Db = Database<Bytes, U64<byteorder::BigEndian>>;
type DbStr = Database<Bytes, Str>;
type DbU64 = Database<Str, U64<byteorder::BigEndian>>;

#[inline] pub fn key(a: u64, b: u64) -> [u8; 16] {
    let mut k = [0u8; 16]; k[..8].copy_from_slice(&a.to_be_bytes()); k[8..].copy_from_slice(&b.to_be_bytes()); k
}

// Bidirectional index: fwd[a,b] and rev[b,a] stay in sync
struct BiPair { fwd: Db, rev: Db }
impl BiPair {
    fn get(&self, tx: &RoTxn, a: u64, b: u64) -> Result<u64> { Ok(self.fwd.get(tx, &key(a, b)).map_err(err)?.unwrap_or(0)) }
    fn put(&self, tx: &mut RwTxn, a: u64, b: u64, v: u64) -> Result<()> { self.fwd.put(tx, &key(a, b), &v).map_err(err)?; self.rev.put(tx, &key(b, a), &v).map_err(err) }
    fn del(&self, tx: &mut RwTxn, a: u64, b: u64) -> Result<bool> { let r = self.fwd.delete(tx, &key(a, b)).map_err(err)?; self.rev.delete(tx, &key(b, a)).map_err(err)?; Ok(r) }
    fn put_or(&self, tx: &mut RwTxn, a: u64, b: u64, mask: u64) -> Result<()> { self.put(tx, a, b, self.get(tx, a, b)? | mask) }
    fn list_fwd(&self, tx: &RoTxn, a: u64) -> Result<Vec<(u64, u64)>> { Self::list_pfx(tx, &self.fwd, a) }
    fn list_rev(&self, tx: &RoTxn, b: u64) -> Result<Vec<(u64, u64)>> { Self::list_pfx(tx, &self.rev, b) }
    fn count_fwd(&self, tx: &RoTxn, a: u64) -> Result<usize> { Self::count_pfx(tx, &self.fwd, a) }
    fn count_rev(&self, tx: &RoTxn, b: u64) -> Result<usize> { Self::count_pfx(tx, &self.rev, b) }
    fn list_pfx(tx: &RoTxn, db: &Db, pfx: u64) -> Result<Vec<(u64, u64)>> {
        let mut r = Vec::new();
        for item in db.prefix_iter(tx, &pfx.to_be_bytes()).map_err(err)? {
            let (k, v) = item.map_err(err)?;
            if k.len() == 16 { r.push((u64::from_be_bytes(k[8..16].try_into().unwrap()), v)); }
        }
        Ok(r)
    }
    fn count_pfx(tx: &RoTxn, db: &Db, pfx: u64) -> Result<usize> {
        Ok(db.prefix_iter(tx, &pfx.to_be_bytes()).map_err(err)?.count())
    }
}

struct Dbs { caps: BiPair, roles: Db, inh: Db, meta: Database<Str, Str>, labels: DbStr, names: DbU64 }

static ENV: OnceLock<Env> = OnceLock::new();
static DBS: OnceLock<Dbs> = OnceLock::new();
static TEST_LOCK: Mutex<()> = Mutex::new(());
static INIT_PATH: OnceLock<String> = OnceLock::new();

fn dbs() -> Result<&'static Dbs> { DBS.get().ok_or_else(|| CapbitError("Not initialized".into())) }
fn env() -> Result<&'static Env> { ENV.get().ok_or_else(|| CapbitError("Not initialized".into())) }

fn read<T, F: FnOnce(&Dbs, &RoTxn) -> Result<T>>(f: F) -> Result<T> { f(dbs()?, &env()?.read_txn().map_err(err)?) }

// Transaction wrapper for batched writes
pub struct Tx(Option<RwTxn<'static>>, &'static Dbs);

impl Tx {
    fn new() -> Result<Self> { Ok(Tx(Some(env()?.write_txn().map_err(err)?), dbs()?)) }
    fn tx(&mut self) -> &mut RwTxn<'static> { self.0.as_mut().unwrap() }
    fn commit(mut self) -> Result<()> { self.0.take().unwrap().commit().map_err(err) }

    pub fn grant(&mut self, subject: u64, object: u64, mask: u64) -> Result<()> {
        self.1.caps.put_or(self.tx(), subject, object, mask)
    }
    pub fn grant_set(&mut self, subject: u64, object: u64, mask: u64) -> Result<()> {
        self.1.caps.put(self.tx(), subject, object, mask)
    }
    pub fn revoke(&mut self, subject: u64, object: u64) -> Result<bool> {
        let r = self.1.caps.del(self.tx(), subject, object)?;
        self.1.inh.delete(self.tx(), &key(object, subject)).map_err(err)?;
        Ok(r)
    }
    pub fn set_role(&mut self, object: u64, role: u64, mask: u64) -> Result<()> {
        self.1.roles.put(self.tx(), &key(object, role), &mask).map_err(err)
    }
    pub fn set_inherit(&mut self, object: u64, child: u64, parent: u64) -> Result<()> {
        self.no_cycle(object, child, parent)?;
        self.1.inh.put(self.tx(), &key(object, child), &parent).map_err(err)
    }
    pub fn remove_inherit(&mut self, object: u64, child: u64) -> Result<bool> {
        self.1.inh.delete(self.tx(), &key(object, child)).map_err(err)
    }
    pub fn create_entity(&mut self, name: &str) -> Result<u64> {
        let id = self.next_id()?;
        self.1.labels.put(self.tx(), &id.to_be_bytes(), name).map_err(err)?;
        self.1.names.put(self.tx(), name, &id).map_err(err)?;
        self.set_next_id(id + 1)?;
        Ok(id)
    }
    pub fn rename_entity(&mut self, id: u64, new_name: &str) -> Result<()> {
        let old = self.1.labels.get(self.tx(), &id.to_be_bytes()).map_err(err)?.map(|s| s.to_string());
        if let Some(old) = old { self.1.names.delete(self.tx(), &old).map_err(err)?; }
        self.1.labels.put(self.tx(), &id.to_be_bytes(), new_name).map_err(err)?;
        self.1.names.put(self.tx(), new_name, &id).map_err(err)
    }
    pub fn delete_entity(&mut self, id: u64) -> Result<bool> {
        let name = self.1.labels.get(self.tx(), &id.to_be_bytes()).map_err(err)?.map(|s| s.to_string());
        if let Some(name) = name { self.1.names.delete(self.tx(), &name).map_err(err)?; }
        self.1.labels.delete(self.tx(), &id.to_be_bytes()).map_err(err)
    }
    pub fn set_label(&mut self, id: u64, name: &str) -> Result<()> {
        let old_id = self.1.names.get(self.tx(), name).map_err(err)?;
        if let Some(old_id) = old_id { if old_id != id { self.1.labels.delete(self.tx(), &old_id.to_be_bytes()).map_err(err)?; } }
        self.1.labels.put(self.tx(), &id.to_be_bytes(), name).map_err(err)?;
        self.1.names.put(self.tx(), name, &id).map_err(err)
    }
    fn next_id(&mut self) -> Result<u64> {
        Ok(self.1.meta.get(self.tx(), "next_id").map_err(err)?.and_then(|s| s.parse().ok()).unwrap_or(1u64))
    }
    fn set_next_id(&mut self, id: u64) -> Result<()> {
        self.1.meta.put(self.tx(), "next_id", &id.to_string()).map_err(err)
    }
    fn no_cycle(&mut self, obj: u64, from: u64, to: u64) -> Result<()> {
        if from == to { return Err(CapbitError("Cannot reference self".into())); }
        let mut cur = to;
        for _ in 0..10 {
            match self.1.inh.get(self.tx(), &key(obj, cur)).map_err(err)? {
                Some(p) if p == from => return Err(CapbitError("Circular reference".into())),
                Some(p) => cur = p,
                None => break
            }
        }
        Ok(())
    }
}

/// Run multiple operations in a single transaction
pub fn transact<T, F: FnOnce(&mut Tx) -> Result<T>>(f: F) -> Result<T> {
    let mut tx = Tx::new()?;
    let r = f(&mut tx)?;
    tx.commit()?;
    Ok(r)
}

// Init
pub fn init(path: &str) -> Result<()> {
    if let Some(p) = INIT_PATH.get() {
        return if p == path { Ok(()) } else { Err(CapbitError(format!("Already init at {}", p))) };
    }
    std::fs::create_dir_all(path).map_err(err)?;
    // SAFETY: LMDB requires no other processes access this path concurrently during open.
    let e = unsafe { EnvOpenOptions::new().map_size(1<<30).max_dbs(7).open(Path::new(path)).map_err(err)? };
    let mut tx = e.write_txn().map_err(err)?;
    let d = Dbs {
        caps: BiPair {
            fwd: e.create_database(&mut tx, Some("caps")).map_err(err)?,
            rev: e.create_database(&mut tx, Some("rev")).map_err(err)?,
        },
        roles: e.create_database(&mut tx, Some("roles")).map_err(err)?,
        inh: e.create_database(&mut tx, Some("inh")).map_err(err)?,
        meta: e.create_database(&mut tx, Some("meta")).map_err(err)?,
        labels: e.create_database(&mut tx, Some("labels")).map_err(err)?,
        names: e.create_database(&mut tx, Some("names")).map_err(err)?,
    };
    tx.commit().map_err(err)?;
    let _ = (ENV.set(e), DBS.set(d), INIT_PATH.set(path.to_string()));
    Ok(())
}

pub fn clear_all() -> Result<()> {
    transact(|tx| {
        tx.1.caps.fwd.clear(tx.tx()).map_err(err)?;
        tx.1.caps.rev.clear(tx.tx()).map_err(err)?;
        tx.1.roles.clear(tx.tx()).map_err(err)?;
        tx.1.inh.clear(tx.tx()).map_err(err)?;
        tx.1.meta.clear(tx.tx()).map_err(err)?;
        tx.1.labels.clear(tx.tx()).map_err(err)?;
        tx.1.names.clear(tx.tx()).map_err(err)
    })
}
pub fn test_lock() -> std::sync::MutexGuard<'static, ()> { TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner()) }

// Single-op convenience functions
pub fn grant(subject: u64, object: u64, mask: u64) -> Result<()> { transact(|tx| tx.grant(subject, object, mask)) }
pub fn grant_set(subject: u64, object: u64, mask: u64) -> Result<()> { transact(|tx| tx.grant_set(subject, object, mask)) }
pub fn revoke(subject: u64, object: u64) -> Result<bool> { transact(|tx| tx.revoke(subject, object)) }
pub fn set_role(object: u64, role: u64, mask: u64) -> Result<()> { transact(|tx| tx.set_role(object, role, mask)) }
pub fn set_inherit(object: u64, child: u64, parent: u64) -> Result<()> { transact(|tx| tx.set_inherit(object, child, parent)) }
pub fn remove_inherit(object: u64, child: u64) -> Result<bool> { transact(|tx| tx.remove_inherit(object, child)) }
pub fn create_entity(name: &str) -> Result<u64> { transact(|tx| tx.create_entity(name)) }
pub fn rename_entity(id: u64, new_name: &str) -> Result<()> { transact(|tx| tx.rename_entity(id, new_name)) }
pub fn delete_entity(id: u64) -> Result<bool> { transact(|tx| tx.delete_entity(id)) }
pub fn set_label(id: u64, name: &str) -> Result<()> { transact(|tx| tx.set_label(id, name)) }

// Batch convenience functions
pub fn batch_grant(grants: &[(u64, u64, u64)]) -> Result<()> {
    transact(|tx| { for &(s, o, m) in grants { tx.grant(s, o, m)?; } Ok(()) })
}
pub fn batch_revoke(revokes: &[(u64, u64)]) -> Result<usize> {
    transact(|tx| { let mut c = 0; for &(s, o) in revokes { if tx.revoke(s, o)? { c += 1; } } Ok(c) })
}

// Read operations (iterative resolve - no recursion)
fn resolve(d: &Dbs, tx: &RoTxn, mut s: u64, o: u64) -> Result<u64> {
    let mut mask = 0u64;
    for _ in 0..=10 {
        let role = d.caps.get(tx, s, o)?;
        mask |= if role == 0 { 0 } else { d.roles.get(tx, &key(o, role)).map_err(err)?.unwrap_or(role) };
        match d.inh.get(tx, &key(o, s)).map_err(err)? { Some(p) => s = p, None => break }
    }
    Ok(mask)
}

pub fn get_mask(subject: u64, object: u64) -> Result<u64> { read(|d, tx| resolve(d, tx, subject, object)) }
pub fn get_role_id(subject: u64, object: u64) -> Result<u64> { read(|d, tx| d.caps.get(tx, subject, object)) }
pub fn check(subject: u64, object: u64, required: u64) -> Result<bool> { Ok((get_mask(subject, object)? & required) == required) }
pub fn get_role(object: u64, role: u64) -> Result<u64> { read(|d, tx| Ok(d.roles.get(tx, &key(object, role)).map_err(err)?.unwrap_or(role))) }
pub fn get_inherit(object: u64, child: u64) -> Result<Option<u64>> { read(|d, tx| Ok(d.inh.get(tx, &key(object, child)).map_err(err)?)) }
pub fn list_for_subject(subject: u64) -> Result<Vec<(u64, u64)>> { read(|d, tx| d.caps.list_fwd(tx, subject)) }
pub fn list_for_object(object: u64) -> Result<Vec<(u64, u64)>> { read(|d, tx| d.caps.list_rev(tx, object)) }
pub fn count_for_subject(subject: u64) -> Result<usize> { read(|d, tx| d.caps.count_fwd(tx, subject)) }
pub fn count_for_object(object: u64) -> Result<usize> { read(|d, tx| d.caps.count_rev(tx, object)) }
pub fn get_label(id: u64) -> Result<Option<String>> { read(|d, tx| Ok(d.labels.get(tx, &id.to_be_bytes()).map_err(err)?.map(|s| s.to_string()))) }
pub fn get_id_by_label(name: &str) -> Result<Option<u64>> { read(|d, tx| Ok(d.names.get(tx, name).map_err(err)?)) }
pub fn list_labels() -> Result<Vec<(u64, String)>> {
    read(|d, tx| {
        let mut r = Vec::new();
        for item in d.labels.iter(tx).map_err(err)? {
            let (k, v) = item.map_err(err)?;
            if k.len() == 8 { r.push((u64::from_be_bytes(k.try_into().unwrap()), v.to_string())); }
        }
        Ok(r)
    })
}

// Capability constants
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
fn is_root(actor: u64) -> Result<bool> { Ok(get_root()? == Some(actor)) }
fn require(actor: u64, object: u64, req: u64) -> Result<()> {
    if is_root(actor)? || (get_mask(actor, object)? & req) == req { Ok(()) }
    else { Err(CapbitError(format!("{} lacks {:x} on {}", actor, req, object))) }
}

pub fn protected_grant(actor: u64, subject: u64, object: u64, mask: u64) -> Result<()> {
    if is_root(actor)? { return grant(subject, object, mask); }
    let m = get_mask(actor, object)?;
    if (m & ADMIN) == 0 && (mask & !m) != 0 { return Err(CapbitError(format!("{} cannot grant {:x}", actor, mask))); }
    grant(subject, object, mask)
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
    transact(|tx| {
        tx.grant(root, root, ADMIN)?;
        tx.1.meta.put(tx.tx(), "boot", "1").map_err(err)?;
        tx.1.meta.put(tx.tx(), "root", &root.to_string()).map_err(err)
    })
}
