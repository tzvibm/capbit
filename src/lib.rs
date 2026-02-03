//! Capbit - Minimal capability-based access control

use fjall::{Config, Keyspace, PartitionCreateOptions, PartitionHandle};
use std::{path::Path, sync::OnceLock};

#[derive(Debug, Clone)]
pub struct Error(pub String);
impl std::fmt::Display for Error { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) } }
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;
fn err(e: impl std::error::Error) -> Error { Error(e.to_string()) }

// Reserved IDs
pub const _SYSTEM: u64 = 1;
pub const _ROOT: u64 = 2;

// Reserved role IDs
pub const _OWNER: u64 = 1;
pub const _ADMIN: u64 = 2;
pub const _EDITOR: u64 = 3;
pub const _VIEWER: u64 = 4;

// Aggregate masks
pub const ALL_BITS: u64 = 0x3FFFFF;
pub const ADMIN_BITS: u64 = 0x3FCFCF;
pub const EDITOR_BITS: u64 = 0x000366;
pub const VIEWER_BITS: u64 = 0x000318;

// Granular bits (internal)
const _CREATE_ROLE: u64 = 1 << 0;
const _UPDATE_ROLE: u64 = 1 << 1;
const _DELETE_ROLE: u64 = 1 << 2;
const _GET_ROLE: u64 = 1 << 3;
const _CHECK_ROLE: u64 = 1 << 4;
const _CREATE_MASK: u64 = 1 << 5;
const _UPDATE_MASK: u64 = 1 << 6;
const _DELETE_MASK: u64 = 1 << 7;
const _GET_MASK: u64 = 1 << 8;
const _CHECK_MASK: u64 = 1 << 9;
const _CREATE_OBJECT: u64 = 1 << 10;
const _DELETE_OBJECT: u64 = 1 << 11;
const _GET_OBJECT: u64 = 1 << 12;
const _CHECK_OBJECT: u64 = 1 << 13;
const _GRANT: u64 = 1 << 14;
const _REVOKE: u64 = 1 << 15;
const _GET_GRANT: u64 = 1 << 16;
const _CHECK_GRANT: u64 = 1 << 17;
const _SET_INHERIT: u64 = 1 << 18;
const _REMOVE_INHERIT: u64 = 1 << 19;
const _GET_INHERIT: u64 = 1 << 20;
const _CHECK_INHERIT: u64 = 1 << 21;

static KS: OnceLock<Keyspace> = OnceLock::new();
static OBJECTS: OnceLock<PartitionHandle> = OnceLock::new();
static SUBJECTS: OnceLock<PartitionHandle> = OnceLock::new();
static SUBJECTS_REV: OnceLock<PartitionHandle> = OnceLock::new();
static INHERITS: OnceLock<PartitionHandle> = OnceLock::new();
static INHERITS_BY_OBJ: OnceLock<PartitionHandle> = OnceLock::new();
static INHERITS_BY_PARENT: OnceLock<PartitionHandle> = OnceLock::new();

// Key builders
#[inline] fn key(a: u64, b: u64) -> [u8; 16] { let mut x = [0u8; 16]; x[..8].copy_from_slice(&a.to_be_bytes()); x[8..].copy_from_slice(&b.to_be_bytes()); x }
#[inline] fn key3(a: u64, b: u64, c: u64) -> [u8; 24] { let mut x = [0u8; 24]; x[..8].copy_from_slice(&a.to_be_bytes()); x[8..16].copy_from_slice(&b.to_be_bytes()); x[16..].copy_from_slice(&c.to_be_bytes()); x }
#[inline] fn key4(a: u64, b: u64, c: u64, d: u64) -> [u8; 32] { let mut x = [0u8; 32]; x[..8].copy_from_slice(&a.to_be_bytes()); x[8..16].copy_from_slice(&b.to_be_bytes()); x[16..24].copy_from_slice(&c.to_be_bytes()); x[24..].copy_from_slice(&d.to_be_bytes()); x }

// Key/value helpers
#[inline] fn u64_at(k: &[u8], pos: usize) -> u64 { u64::from_be_bytes(k[pos*8..(pos+1)*8].try_into().unwrap()) }
#[inline] fn val(v: &[u8]) -> u64 { u64::from_be_bytes(v[..8].try_into().unwrap()) }

// CRUD primitives
fn ks() -> &'static Keyspace { KS.get().unwrap() }
fn get(p: &PartitionHandle, k: &[u8]) -> Result<Option<u64>> { Ok(p.get(k).map_err(err)?.map(|v| val(&v))) }
fn set(p: &PartitionHandle, k: &[u8], v: u64) -> Result<()> { p.insert(k, &v.to_be_bytes()).map_err(err)?; ks().persist(fjall::PersistMode::Buffer).map_err(err) }
fn del(p: &PartitionHandle, k: &[u8]) -> Result<()> { p.remove(k).map_err(err)?; ks().persist(fjall::PersistMode::Buffer).map_err(err) }

// Transaction helper for atomic multi-partition writes
fn transact(f: impl FnOnce(&mut fjall::Batch)) -> Result<()> {
    let mut batch = ks().batch();
    f(&mut batch);
    batch.commit().map_err(err)?;
    ks().persist(fjall::PersistMode::Buffer).map_err(err)
}

// Generic scan with extractor
fn scan<T>(p: &PartitionHandle, prefix: &[u8], f: impl Fn(&[u8], &[u8]) -> T) -> Result<Vec<T>> {
    let mut out = Vec::new();
    for kv in p.prefix(prefix) {
        let (k, v) = kv.map_err(err)?;
        out.push(f(&k, &v));
    }
    Ok(out)
}

pub fn init(path: &str) -> Result<()> {
    if KS.get().is_some() { return Ok(()); }
    std::fs::create_dir_all(path).map_err(err)?;
    let ks = Config::new(Path::new(path)).open().map_err(err)?;
    let o = PartitionCreateOptions::default();
    let _ = (OBJECTS.set(ks.open_partition("objects", o.clone()).map_err(err)?),
             SUBJECTS.set(ks.open_partition("subjects", o.clone()).map_err(err)?),
             SUBJECTS_REV.set(ks.open_partition("subjects_rev", o.clone()).map_err(err)?),
             INHERITS.set(ks.open_partition("inherits", o.clone()).map_err(err)?),
             INHERITS_BY_OBJ.set(ks.open_partition("inherits_by_obj", o.clone()).map_err(err)?),
             INHERITS_BY_PARENT.set(ks.open_partition("inherits_by_parent", o).map_err(err)?),
             KS.set(ks));
    Ok(())
}

fn auth(actor: u64, object: u64, req: u64) -> Result<()> {
    if check(actor, object, req)? { Ok(()) } else { Err(Error("Denied".into())) }
}

// Resolution
pub fn get_mask(sub: u64, obj: u64) -> Result<u64> {
    let (sp, op, ip) = (SUBJECTS.get().unwrap(), OBJECTS.get().unwrap(), INHERITS.get().unwrap());
    let (mut mask, mut cur) = (0u64, sub);
    for _ in 0..10 {
        let mut found = false;
        for kv in sp.prefix(&key(cur, obj)) {
            let (k, _) = kv.map_err(err)?;
            let role = u64_at(&k, 2);
            mask |= get(op, &key(obj, role))?.unwrap_or(role);
            found = true;
            if let Some(p) = get(ip, &key3(cur, obj, role))? {
                cur = p;
                break;
            }
        }
        if !found { break; }
    }
    Ok(mask)
}

pub fn check(sub: u64, obj: u64, req: u64) -> Result<bool> { Ok(get_mask(sub, obj)? & req == req) }

// OBJECTS table
pub fn create(actor: u64, obj: u64, role: u64, mask: u64) -> Result<()> {
    auth(actor, obj, _CREATE_ROLE | _CREATE_MASK)?;
    if get(OBJECTS.get().unwrap(), &key(obj, role))?.is_some() { return Err(Error("Exists".into())); }
    set(OBJECTS.get().unwrap(), &key(obj, role), mask)
}

pub fn delete(actor: u64, obj: u64, role: u64) -> Result<()> {
    auth(actor, obj, _DELETE_ROLE | _DELETE_MASK)?;
    del(OBJECTS.get().unwrap(), &key(obj, role))
}

pub fn update(actor: u64, obj: u64, role: u64, mask: u64) -> Result<()> {
    auth(actor, obj, _UPDATE_ROLE | _UPDATE_MASK)?;
    set(OBJECTS.get().unwrap(), &key(obj, role), mask)
}

pub fn get_object(actor: u64, obj: u64, role: u64) -> Result<Option<u64>> {
    auth(actor, obj, _GET_ROLE | _GET_MASK)?;
    get(OBJECTS.get().unwrap(), &key(obj, role))
}

pub fn check_object(actor: u64, obj: u64, role: u64) -> Result<bool> {
    auth(actor, obj, _CHECK_ROLE | _CHECK_MASK)?;
    Ok(get(OBJECTS.get().unwrap(), &key(obj, role))?.is_some())
}

pub fn list_roles(actor: u64, obj: u64) -> Result<Vec<(u64, u64)>> {
    auth(actor, obj, _GET_ROLE | _GET_MASK)?;
    scan(OBJECTS.get().unwrap(), &obj.to_be_bytes(), |k, v| (u64_at(k, 1), val(v)))
}

// SUBJECTS table - (subject, object, role) with reverse index (object, subject, role)
pub fn grant(actor: u64, sub: u64, obj: u64, role: u64) -> Result<()> {
    auth(actor, obj, _GRANT)?;
    transact(|b| {
        b.insert(SUBJECTS.get().unwrap(), &key3(sub, obj, role), &1u64.to_be_bytes());
        b.insert(SUBJECTS_REV.get().unwrap(), &key3(obj, sub, role), &1u64.to_be_bytes());
    })
}

pub fn revoke(actor: u64, sub: u64, obj: u64, role: u64) -> Result<()> {
    auth(actor, obj, _REVOKE)?;
    transact(|b| {
        b.remove(SUBJECTS.get().unwrap(), &key3(sub, obj, role));
        b.remove(SUBJECTS_REV.get().unwrap(), &key3(obj, sub, role));
    })
}

pub fn check_subject(sub: u64, obj: u64, role: u64) -> Result<bool> {
    Ok(get(SUBJECTS.get().unwrap(), &key3(sub, obj, role))?.is_some())
}

pub fn list_roles_for(actor: u64, sub: u64, obj: u64) -> Result<Vec<u64>> {
    auth(actor, obj, _GET_GRANT)?;
    scan(SUBJECTS.get().unwrap(), &key(sub, obj), |k, _| u64_at(k, 2))
}

pub fn list_grants(actor: u64, sub: u64) -> Result<Vec<(u64, u64)>> {
    auth(actor, _SYSTEM, _GET_GRANT)?;
    scan(SUBJECTS.get().unwrap(), &sub.to_be_bytes(), |k, _| (u64_at(k, 1), u64_at(k, 2)))
}

pub fn list_subjects(actor: u64, obj: u64) -> Result<Vec<(u64, u64)>> {
    auth(actor, obj, _GET_GRANT)?;
    scan(SUBJECTS_REV.get().unwrap(), &obj.to_be_bytes(), |k, _| (u64_at(k, 1), u64_at(k, 2)))
}

// INHERITS table - (subject, object, role) → parent with reverse indexes
pub fn inherit(actor: u64, sub: u64, obj: u64, role: u64, parent: u64) -> Result<()> {
    auth(actor, obj, _SET_INHERIT)?;
    if sub == parent { return Err(Error("Self".into())); }
    transact(|b| {
        b.insert(INHERITS.get().unwrap(), &key3(sub, obj, role), &parent.to_be_bytes());
        b.insert(INHERITS_BY_OBJ.get().unwrap(), &key4(obj, role, parent, sub), &1u64.to_be_bytes());
        b.insert(INHERITS_BY_PARENT.get().unwrap(), &key4(parent, obj, role, sub), &1u64.to_be_bytes());
    })
}

pub fn remove_inherit(actor: u64, sub: u64, obj: u64, role: u64) -> Result<()> {
    auth(actor, obj, _REMOVE_INHERIT)?;
    if let Some(parent) = get(INHERITS.get().unwrap(), &key3(sub, obj, role))? {
        transact(|b| {
            b.remove(INHERITS.get().unwrap(), &key3(sub, obj, role));
            b.remove(INHERITS_BY_OBJ.get().unwrap(), &key4(obj, role, parent, sub));
            b.remove(INHERITS_BY_PARENT.get().unwrap(), &key4(parent, obj, role, sub));
        })
    } else {
        Ok(())
    }
}

pub fn get_inherit(actor: u64, sub: u64, obj: u64, role: u64) -> Result<Option<u64>> {
    auth(actor, obj, _GET_INHERIT)?;
    get(INHERITS.get().unwrap(), &key3(sub, obj, role))
}

pub fn check_inherit(actor: u64, sub: u64, obj: u64, role: u64) -> Result<bool> {
    auth(actor, obj, _CHECK_INHERIT)?;
    Ok(get(INHERITS.get().unwrap(), &key3(sub, obj, role))?.is_some())
}

pub fn list_inherits(actor: u64, sub: u64, obj: u64) -> Result<Vec<(u64, u64)>> {
    auth(actor, obj, _GET_INHERIT)?;
    scan(INHERITS.get().unwrap(), &key(sub, obj), |k, v| (u64_at(k, 2), val(v)))
}

pub fn list_inherits_on_obj(actor: u64, obj: u64) -> Result<Vec<(u64, u64, u64)>> {
    auth(actor, obj, _GET_INHERIT)?;
    scan(INHERITS_BY_OBJ.get().unwrap(), &obj.to_be_bytes(), |k, _| (u64_at(k, 1), u64_at(k, 2), u64_at(k, 3)))
}

pub fn list_inherits_on_obj_role(actor: u64, obj: u64, role: u64) -> Result<Vec<(u64, u64)>> {
    auth(actor, obj, _GET_INHERIT)?;
    scan(INHERITS_BY_OBJ.get().unwrap(), &key(obj, role), |k, _| (u64_at(k, 2), u64_at(k, 3)))
}

pub fn list_inherits_from_parent(actor: u64, parent: u64) -> Result<Vec<(u64, u64, u64)>> {
    auth(actor, _SYSTEM, _GET_INHERIT)?;
    scan(INHERITS_BY_PARENT.get().unwrap(), &parent.to_be_bytes(), |k, _| (u64_at(k, 1), u64_at(k, 2), u64_at(k, 3)))
}

pub fn list_inherits_from_parent_on_obj(actor: u64, parent: u64, obj: u64) -> Result<Vec<(u64, u64)>> {
    auth(actor, obj, _GET_INHERIT)?;
    scan(INHERITS_BY_PARENT.get().unwrap(), &key(parent, obj), |k, _| (u64_at(k, 2), u64_at(k, 3)))
}

// Bootstrap
pub fn bootstrap() -> Result<(u64, u64)> {
    let obj = OBJECTS.get().unwrap();
    if get(obj, &key(_SYSTEM, _OWNER))?.is_some() {
        return Err(Error("Already bootstrapped".into()));
    }
    transact(|b| {
        b.insert(obj, &key(_SYSTEM, _OWNER), &ALL_BITS.to_be_bytes());
        b.insert(obj, &key(_SYSTEM, _ADMIN), &ADMIN_BITS.to_be_bytes());
        b.insert(obj, &key(_SYSTEM, _EDITOR), &EDITOR_BITS.to_be_bytes());
        b.insert(obj, &key(_SYSTEM, _VIEWER), &VIEWER_BITS.to_be_bytes());
        b.insert(SUBJECTS.get().unwrap(), &key3(_ROOT, _SYSTEM, _OWNER), &1u64.to_be_bytes());
        b.insert(SUBJECTS_REV.get().unwrap(), &key3(_SYSTEM, _ROOT, _OWNER), &1u64.to_be_bytes());
    })?;
    Ok((_SYSTEM, _ROOT))
}

pub fn clear() -> Result<()> {
    for p in [OBJECTS.get().unwrap(), SUBJECTS.get().unwrap(), SUBJECTS_REV.get().unwrap(),
              INHERITS.get().unwrap(), INHERITS_BY_OBJ.get().unwrap(), INHERITS_BY_PARENT.get().unwrap()] {
        for kv in p.prefix(&[]) { p.remove(&*kv.map_err(err)?.0).map_err(err)?; }
    }
    KS.get().unwrap().persist(fjall::PersistMode::Buffer).map_err(err)
}