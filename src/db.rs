//! Database types and global state

use std::path::Path;
use std::sync::{Mutex, OnceLock};
use heed::{Database, Env, EnvOpenOptions, RoTxn, RwTxn};
use heed::types::{Bytes, Str, U64};

use crate::error::{err, CapbitError, Result};
use crate::planner::Planner;

// Database type aliases
pub type Db = Database<Bytes, U64<byteorder::BigEndian>>;
pub type DbStr = Database<Bytes, Str>;
pub type DbU64 = Database<Str, U64<byteorder::BigEndian>>;

/// Create a 16-byte key from two u64 values
#[inline]
pub fn key(a: u64, b: u64) -> [u8; 16] {
    let a = a.to_be_bytes();
    let b = b.to_be_bytes();
    [a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7],
     b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]
}

/// Bidirectional index: fwd[a,b] and rev[b,a] stay in sync
pub struct BiPair {
    pub fwd: Db,
    pub rev: Db,
}

impl BiPair {
    #[inline]
    pub fn get(&self, tx: &RoTxn, a: u64, b: u64) -> Result<u64> {
        Ok(self.fwd.get(tx, &key(a, b)).map_err(err)?.unwrap_or(0))
    }

    #[inline]
    pub fn put(&self, tx: &mut RwTxn, a: u64, b: u64, v: u64) -> Result<()> {
        self.fwd.put(tx, &key(a, b), &v).map_err(err)?;
        self.rev.put(tx, &key(b, a), &v).map_err(err)
    }

    #[inline]
    pub fn del(&self, tx: &mut RwTxn, a: u64, b: u64) -> Result<bool> {
        let r = self.fwd.delete(tx, &key(a, b)).map_err(err)?;
        self.rev.delete(tx, &key(b, a)).map_err(err)?;
        Ok(r)
    }

    #[inline]
    pub fn put_or(&self, tx: &mut RwTxn, a: u64, b: u64, mask: u64) -> Result<()> {
        self.put(tx, a, b, self.get(tx, a, b)? | mask)
    }

    pub fn list_fwd(&self, tx: &RoTxn, a: u64) -> Result<Vec<(u64, u64)>> {
        Self::list_pfx(tx, &self.fwd, a)
    }

    pub fn list_rev(&self, tx: &RoTxn, b: u64) -> Result<Vec<(u64, u64)>> {
        Self::list_pfx(tx, &self.rev, b)
    }

    pub fn count_fwd(&self, tx: &RoTxn, a: u64) -> Result<usize> {
        Self::count_pfx(tx, &self.fwd, a)
    }

    pub fn count_rev(&self, tx: &RoTxn, b: u64) -> Result<usize> {
        Self::count_pfx(tx, &self.rev, b)
    }

    fn list_pfx(tx: &RoTxn, db: &Db, pfx: u64) -> Result<Vec<(u64, u64)>> {
        let mut r = Vec::new();
        for item in db.prefix_iter(tx, &pfx.to_be_bytes()).map_err(err)? {
            let (k, v) = item.map_err(err)?;
            if k.len() == 16 {
                r.push((u64::from_be_bytes(k[8..16].try_into().unwrap()), v));
            }
        }
        Ok(r)
    }

    fn count_pfx(tx: &RoTxn, db: &Db, pfx: u64) -> Result<usize> {
        Ok(db.prefix_iter(tx, &pfx.to_be_bytes()).map_err(err)?.count())
    }
}

/// All database handles
pub struct Dbs {
    pub caps: BiPair,
    pub roles: Db,
    pub inh: Db,
    pub meta: Database<Str, Str>,
    pub labels: DbStr,
    pub names: DbU64,
}

// Global state
pub static ENV: OnceLock<Env> = OnceLock::new();
pub static DBS: OnceLock<Dbs> = OnceLock::new();
pub static TEST_LOCK: Mutex<()> = Mutex::new(());
pub static INIT_PATH: OnceLock<String> = OnceLock::new();
pub static PLANNER: OnceLock<Planner> = OnceLock::new();

/// Get the database handles, or error if not initialized
#[inline]
pub fn dbs() -> Result<&'static Dbs> {
    DBS.get().ok_or_else(|| CapbitError("Not initialized".into()))
}

/// Get the environment, or error if not initialized
#[inline]
pub fn env() -> Result<&'static Env> {
    ENV.get().ok_or_else(|| CapbitError("Not initialized".into()))
}

/// Get the planner, or error if not initialized
#[inline]
pub fn planner() -> Result<&'static Planner> {
    PLANNER.get().ok_or_else(|| CapbitError("Not initialized".into()))
}

/// Execute a read-only operation
#[inline]
pub fn read<T, F: FnOnce(&Dbs, &RoTxn) -> Result<T>>(f: F) -> Result<T> {
    f(dbs()?, &env()?.read_txn().map_err(err)?)
}

/// Initialize the database
pub fn init(path: &str) -> Result<()> {
    if let Some(p) = INIT_PATH.get() {
        return if p == path {
            Ok(())
        } else {
            Err(CapbitError(format!("Already init at {}", p)))
        };
    }
    std::fs::create_dir_all(path).map_err(err)?;
    // SAFETY: LMDB requires no other processes access this path concurrently during open.
    let e = unsafe {
        EnvOpenOptions::new()
            .map_size(1 << 30)
            .max_dbs(7)
            .open(Path::new(path))
            .map_err(err)?
    };
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
    let _ = PLANNER.set(Planner::new());
    Ok(())
}

/// Clear all databases (for testing)
pub fn clear_all() -> Result<()> {
    crate::tx::transact(|tx| {
        tx.dbs().caps.fwd.clear(tx.tx()).map_err(err)?;
        tx.dbs().caps.rev.clear(tx.tx()).map_err(err)?;
        tx.dbs().roles.clear(tx.tx()).map_err(err)?;
        tx.dbs().inh.clear(tx.tx()).map_err(err)?;
        tx.dbs().meta.clear(tx.tx()).map_err(err)?;
        tx.dbs().labels.clear(tx.tx()).map_err(err)?;
        tx.dbs().names.clear(tx.tx()).map_err(err)
    })
}

/// Get the test lock (for single-threaded tests)
pub fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner())
}
