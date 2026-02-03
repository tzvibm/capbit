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
static INHERITS: OnceLock<PartitionHandle> = OnceLock::new();

#[inline] fn key(a: u64, b: u64) -> [u8; 16] { let mut x = [0u8; 16]; x[..8].copy_from_slice(&a.to_be_bytes()); x[8..].copy_from_slice(&b.to_be_bytes()); x }
#[inline] fn key3(a: u64, b: u64, c: u64) -> [u8; 24] { let mut x = [0u8; 24]; x[..8].copy_from_slice(&a.to_be_bytes()); x[8..16].copy_from_slice(&b.to_be_bytes()); x[16..].copy_from_slice(&c.to_be_bytes()); x }
#[inline] fn val(v: &[u8]) -> u64 { u64::from_be_bytes(v[..8].try_into().unwrap()) }
fn get(p: &PartitionHandle, k: &[u8]) -> Result<Option<u64>> { Ok(p.get(k).map_err(err)?.map(|v| val(&v))) }
fn set(p: &PartitionHandle, k: &[u8], v: u64) -> Result<()> { p.insert(k, &v.to_be_bytes()).map_err(err)?; KS.get().unwrap().persist(fjall::PersistMode::Buffer).map_err(err) }
fn del(p: &PartitionHandle, k: &[u8]) -> Result<()> { p.remove(k).map_err(err)?; KS.get().unwrap().persist(fjall::PersistMode::Buffer).map_err(err) }

pub fn init(path: &str) -> Result<()> {
    if KS.get().is_some() { return Ok(()); }
    std::fs::create_dir_all(path).map_err(err)?;
    let ks = Config::new(Path::new(path)).open().map_err(err)?;
    let o = PartitionCreateOptions::default();
    let _ = (OBJECTS.set(ks.open_partition("objects", o.clone()).map_err(err)?),
             SUBJECTS.set(ks.open_partition("subjects", o.clone()).map_err(err)?),
             INHERITS.set(ks.open_partition("inherits", o).map_err(err)?), KS.set(ks));
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
        if let Some(role) = get(sp, &key(cur, obj))? {
            mask |= get(op, &key(obj, role))?.unwrap_or(role);
            match get(ip, &key3(cur, obj, role))? {
                Some(p) => cur = p,
                None => break
            }
        } else {
            break
        }
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

// SUBJECTS table
pub fn grant(actor: u64, sub: u64, obj: u64, role: u64) -> Result<()> {
    auth(actor, obj, _GRANT)?;
    set(SUBJECTS.get().unwrap(), &key(sub, obj), role)
}

pub fn revoke(actor: u64, sub: u64, obj: u64) -> Result<()> {
    auth(actor, obj, _REVOKE)?;
    del(SUBJECTS.get().unwrap(), &key(sub, obj))
}

pub fn get_subject(actor: u64, sub: u64, obj: u64) -> Result<Option<u64>> {
    auth(actor, obj, _GET_GRANT)?;
    get(SUBJECTS.get().unwrap(), &key(sub, obj))
}

pub fn check_subject(sub: u64, obj: u64, role: u64) -> Result<bool> {
    Ok(get(SUBJECTS.get().unwrap(), &key(sub, obj))? == Some(role))
}

// INHERITS table
pub fn inherit(actor: u64, sub: u64, obj: u64, role: u64, parent: u64) -> Result<()> {
    auth(actor, obj, _SET_INHERIT)?;
    if sub == parent { return Err(Error("Self".into())); }
    set(INHERITS.get().unwrap(), &key3(sub, obj, role), parent)
}

pub fn remove_inherit(actor: u64, sub: u64, obj: u64, role: u64) -> Result<()> {
    auth(actor, obj, _REMOVE_INHERIT)?;
    del(INHERITS.get().unwrap(), &key3(sub, obj, role))
}

pub fn get_inherit(actor: u64, sub: u64, obj: u64, role: u64) -> Result<Option<u64>> {
    auth(actor, obj, _GET_INHERIT)?;
    get(INHERITS.get().unwrap(), &key3(sub, obj, role))
}

pub fn check_inherit(actor: u64, sub: u64, obj: u64, role: u64) -> Result<bool> {
    auth(actor, obj, _CHECK_INHERIT)?;
    Ok(get(INHERITS.get().unwrap(), &key3(sub, obj, role))?.is_some())
}

// Bootstrap
pub fn bootstrap() -> Result<(u64, u64)> {
    let obj = OBJECTS.get().unwrap();
    if get(obj, &key(_SYSTEM, _OWNER))?.is_some() {
        return Err(Error("Already bootstrapped".into()));
    }
    let sub = SUBJECTS.get().unwrap();
    set(obj, &key(_SYSTEM, _OWNER), ALL_BITS)?;
    set(obj, &key(_SYSTEM, _ADMIN), ADMIN_BITS)?;
    set(obj, &key(_SYSTEM, _EDITOR), EDITOR_BITS)?;
    set(obj, &key(_SYSTEM, _VIEWER), VIEWER_BITS)?;
    set(sub, &key(_ROOT, _SYSTEM), _OWNER)?;
    Ok((_SYSTEM, _ROOT))
}

pub fn clear() -> Result<()> {
    for p in [OBJECTS.get().unwrap(), SUBJECTS.get().unwrap(), INHERITS.get().unwrap()] {
        for kv in p.prefix(&[]) { p.remove(&*kv.map_err(err)?.0).map_err(err)?; }
    }
    KS.get().unwrap().persist(fjall::PersistMode::Buffer).map_err(err)
}