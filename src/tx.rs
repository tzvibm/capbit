//! Transaction wrapper for batched writes

use heed::RwTxn;

use crate::constants::MAX_INHERITANCE_DEPTH;
use crate::db::{dbs, env, key, Dbs};
use crate::error::{err, CapbitError, Result};

/// Transaction wrapper for batched writes
pub struct Tx {
    txn: Option<RwTxn<'static>>,
    dbs: &'static Dbs,
}

impl Tx {
    #[inline]
    pub(crate) fn new() -> Result<Self> {
        Ok(Tx {
            txn: Some(env()?.write_txn().map_err(err)?),
            dbs: dbs()?,
        })
    }

    #[inline]
    pub(crate) fn tx(&mut self) -> &mut RwTxn<'static> {
        self.txn.as_mut().unwrap()
    }

    #[inline]
    pub(crate) fn dbs(&self) -> &'static Dbs {
        self.dbs
    }

    #[inline]
    pub(crate) fn commit(mut self) -> Result<()> {
        self.txn.take().unwrap().commit().map_err(err)
    }

    /// Grant permissions (OR with existing mask)
    #[inline]
    pub fn grant(&mut self, subject: u64, object: u64, mask: u64) -> Result<()> {
        self.dbs.caps.put_or(self.tx(), subject, object, mask)
    }

    /// Set permissions exactly (replace existing mask)
    #[inline]
    pub fn grant_set(&mut self, subject: u64, object: u64, mask: u64) -> Result<()> {
        self.dbs.caps.put(self.tx(), subject, object, mask)
    }

    /// Revoke all permissions
    #[inline]
    pub fn revoke(&mut self, subject: u64, object: u64) -> Result<bool> {
        let r = self.dbs.caps.del(self.tx(), subject, object)?;
        self.dbs.inh.delete(self.tx(), &key(object, subject)).map_err(err)?;
        Ok(r)
    }

    /// Set a role's permission mask
    #[inline]
    pub fn set_role(&mut self, object: u64, role: u64, mask: u64) -> Result<()> {
        self.dbs.roles.put(self.tx(), &key(object, role), &mask).map_err(err)
    }

    /// Set inheritance relationship
    #[inline]
    pub fn set_inherit(&mut self, object: u64, child: u64, parent: u64) -> Result<()> {
        self.no_cycle(object, child, parent)?;
        self.dbs.inh.put(self.tx(), &key(object, child), &parent).map_err(err)
    }

    /// Remove inheritance relationship
    #[inline]
    pub fn remove_inherit(&mut self, object: u64, child: u64) -> Result<bool> {
        self.dbs.inh.delete(self.tx(), &key(object, child)).map_err(err)
    }

    /// Create a new entity with a name
    pub fn create_entity(&mut self, name: &str) -> Result<u64> {
        let id = self.next_id()?;
        self.dbs.labels.put(self.tx(), &id.to_be_bytes(), name).map_err(err)?;
        self.dbs.names.put(self.tx(), name, &id).map_err(err)?;
        self.set_next_id(id + 1)?;
        Ok(id)
    }

    /// Rename an entity
    pub fn rename_entity(&mut self, id: u64, new_name: &str) -> Result<()> {
        let old = self.dbs.labels.get(self.tx(), &id.to_be_bytes()).map_err(err)?.map(|s| s.to_string());
        if let Some(old) = old {
            self.dbs.names.delete(self.tx(), &old).map_err(err)?;
        }
        self.dbs.labels.put(self.tx(), &id.to_be_bytes(), new_name).map_err(err)?;
        self.dbs.names.put(self.tx(), new_name, &id).map_err(err)
    }

    /// Delete an entity
    pub fn delete_entity(&mut self, id: u64) -> Result<bool> {
        let name = self.dbs.labels.get(self.tx(), &id.to_be_bytes()).map_err(err)?.map(|s| s.to_string());
        if let Some(name) = name {
            self.dbs.names.delete(self.tx(), &name).map_err(err)?;
        }
        self.dbs.labels.delete(self.tx(), &id.to_be_bytes()).map_err(err)
    }

    /// Set or update an entity's label
    pub fn set_label(&mut self, id: u64, name: &str) -> Result<()> {
        let old_id = self.dbs.names.get(self.tx(), name).map_err(err)?;
        if let Some(old_id) = old_id {
            if old_id != id {
                self.dbs.labels.delete(self.tx(), &old_id.to_be_bytes()).map_err(err)?;
            }
        }
        self.dbs.labels.put(self.tx(), &id.to_be_bytes(), name).map_err(err)?;
        self.dbs.names.put(self.tx(), name, &id).map_err(err)
    }

    pub(crate) fn next_id(&mut self) -> Result<u64> {
        Ok(self.dbs.meta
            .get(self.tx(), "next_id")
            .map_err(err)?
            .and_then(|s| s.parse().ok())
            .unwrap_or(1u64))
    }

    pub(crate) fn set_next_id(&mut self, id: u64) -> Result<()> {
        self.dbs.meta.put(self.tx(), "next_id", &id.to_string()).map_err(err)
    }

    fn no_cycle(&mut self, obj: u64, from: u64, to: u64) -> Result<()> {
        if from == to {
            return Err(CapbitError("Cannot reference self".into()));
        }
        let mut cur = to;
        for _ in 0..MAX_INHERITANCE_DEPTH {
            match self.dbs.inh.get(self.tx(), &key(obj, cur)).map_err(err)? {
                Some(p) if p == from => return Err(CapbitError("Circular reference".into())),
                Some(p) => cur = p,
                None => break,
            }
        }
        Ok(())
    }
}

/// Run multiple operations in a single transaction
#[inline]
pub fn transact<T, F: FnOnce(&mut Tx) -> Result<T>>(f: F) -> Result<T> {
    let mut tx = Tx::new()?;
    let r = f(&mut tx)?;
    tx.commit()?;
    Ok(r)
}
