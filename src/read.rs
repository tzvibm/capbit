//! Read operations (no permission checks, direct LMDB access)

use heed::RoTxn;

use crate::constants::MAX_INHERITANCE_DEPTH;
use crate::db::{key, read, Dbs};
use crate::error::{err, Result};

/// Resolve effective permission mask (iterative - no recursion)
#[inline]
pub(crate) fn resolve(d: &Dbs, tx: &RoTxn, mut s: u64, o: u64) -> Result<u64> {
    let mut mask = 0u64;
    for _ in 0..MAX_INHERITANCE_DEPTH {
        let role = d.caps.get(tx, s, o)?;
        mask |= if role == 0 {
            0
        } else {
            d.roles.get(tx, &key(o, role)).map_err(err)?.unwrap_or(role)
        };
        match d.inh.get(tx, &key(o, s)).map_err(err)? {
            Some(p) => s = p,
            None => break,
        }
    }
    Ok(mask)
}

/// Get the effective permission mask for a subject on an object
#[inline]
pub fn get_mask(subject: u64, object: u64) -> Result<u64> {
    read(|d, tx| resolve(d, tx, subject, object))
}

/// Get the role ID assigned to a subject on an object
#[inline]
pub fn get_role_id(subject: u64, object: u64) -> Result<u64> {
    read(|d, tx| d.caps.get(tx, subject, object))
}

/// Check if subject has required permissions on object
#[inline]
pub fn check(subject: u64, object: u64, required: u64) -> Result<bool> {
    Ok((get_mask(subject, object)? & required) == required)
}

/// Get the permission mask for a role on an object
pub fn get_role(object: u64, role: u64) -> Result<u64> {
    read(|d, tx| Ok(d.roles.get(tx, &key(object, role)).map_err(err)?.unwrap_or(role)))
}

/// Get the parent in an inheritance chain
pub fn get_inherit(object: u64, child: u64) -> Result<Option<u64>> {
    read(|d, tx| d.inh.get(tx, &key(object, child)).map_err(err))
}

/// List all objects a subject has permissions on
pub fn list_for_subject(subject: u64) -> Result<Vec<(u64, u64)>> {
    read(|d, tx| d.caps.list_fwd(tx, subject))
}

/// Count objects a subject has permissions on
pub fn count_for_subject(subject: u64) -> Result<usize> {
    read(|d, tx| d.caps.count_fwd(tx, subject))
}

/// Count subjects that have permissions on an object
pub fn count_for_object(object: u64) -> Result<usize> {
    read(|d, tx| d.caps.count_rev(tx, object))
}

/// Get the label for an entity ID
pub fn get_label(id: u64) -> Result<Option<String>> {
    read(|d, tx| {
        Ok(d.labels
            .get(tx, &id.to_be_bytes())
            .map_err(err)?
            .map(|s| s.to_string()))
    })
}

/// Get an entity ID by its label
pub fn get_id_by_label(name: &str) -> Result<Option<u64>> {
    read(|d, tx| d.names.get(tx, name).map_err(err))
}

/// List all entity labels
pub fn list_labels() -> Result<Vec<(u64, String)>> {
    read(|d, tx| {
        let mut r = Vec::new();
        for item in d.labels.iter(tx).map_err(err)? {
            let (k, v) = item.map_err(err)?;
            if k.len() == 8 {
                r.push((u64::from_be_bytes(k.try_into().unwrap()), v.to_string()));
            }
        }
        Ok(r)
    })
}

/// List subjects with permissions on an object (internal, no permission check)
pub(crate) fn list_for_object_internal(object: u64) -> Result<Vec<(u64, u64)>> {
    read(|d, tx| d.caps.list_rev(tx, object))
}
