//! Public write API - all operations require actor with permission

use crate::constants::{GRANT, SYSTEM_ID, VIEW};
use crate::db::planner;
use crate::error::{CapbitError, Result};
use crate::planner::Op;
use crate::read::{check, list_for_object_internal};
use crate::tx::transact;

/// Check if actor has required permission on object
#[inline]
fn require(actor: u64, object: u64, perm: u64) -> Result<()> {
    if check(actor, object, perm)? {
        Ok(())
    } else {
        Err(CapbitError(format!(
            "{} lacks {:x} on {}",
            actor, perm, object
        )))
    }
}

/// Check if actor has required permission on _system
#[inline]
fn require_system(actor: u64, perm: u64) -> Result<()> {
    // SYSTEM_ID is a known constant - no DB lookup needed
    if check(actor, SYSTEM_ID, perm)? {
        Ok(())
    } else {
        Err(CapbitError(format!(
            "{} lacks {:x} on _system",
            actor, perm
        )))
    }
}

/// Grant permissions to a subject on an object (requires GRANT on object)
pub fn grant(actor: u64, subject: u64, object: u64, mask: u64) -> Result<()> {
    require(actor, object, GRANT)?;
    planner()?.submit(Op::Grant { subject, object, mask })
}

/// Set permissions exactly (replace existing mask) - requires GRANT on object
pub fn grant_set(actor: u64, subject: u64, object: u64, mask: u64) -> Result<()> {
    require(actor, object, GRANT)?;
    transact(|tx| tx.grant_set(subject, object, mask))
}

/// Revoke all permissions from a subject on an object (requires GRANT on object)
pub fn revoke(actor: u64, subject: u64, object: u64) -> Result<()> {
    require(actor, object, GRANT)?;
    planner()?.submit(Op::Revoke { subject, object })
}

/// Set a role's permission mask (requires ADMIN on _system)
pub fn set_role(actor: u64, object: u64, role: u64, mask: u64) -> Result<()> {
    require_system(actor, crate::constants::ADMIN)?;
    planner()?.submit(Op::SetRole { object, role, mask })
}

/// Set inheritance relationship (requires ADMIN on _system)
pub fn set_inherit(actor: u64, object: u64, child: u64, parent: u64) -> Result<()> {
    require_system(actor, crate::constants::ADMIN)?;
    planner()?.submit(Op::SetInherit { object, child, parent })
}

/// Remove inheritance relationship (requires ADMIN on _system)
pub fn remove_inherit(actor: u64, object: u64, child: u64) -> Result<()> {
    require_system(actor, crate::constants::ADMIN)?;
    planner()?.submit(Op::RemoveInherit { object, child })
}

/// List subjects with permissions on an object (requires VIEW on object)
pub fn list_for_object(actor: u64, object: u64) -> Result<Vec<(u64, u64)>> {
    require(actor, object, VIEW)?;
    list_for_object_internal(object)
}
