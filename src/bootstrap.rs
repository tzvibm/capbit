//! Bootstrap and system initialization

use crate::constants::{ADMIN, GRANT, ROLE_ADMIN, ROLE_FULL, ROLE_GRANTER, ROLE_VIEWER, ROOT_USER_ID, SYSTEM_ID, VIEW};
use crate::error::{CapbitError, Result};
use crate::read::get_mask;
use crate::tx::transact;

/// Check if bootstrapped by seeing if root_user has grant on system
pub fn is_bootstrapped() -> Result<bool> {
    Ok(get_mask(ROOT_USER_ID, SYSTEM_ID)? != 0)
}

/// Get the root user ID (returns constant, no DB query)
/// Returns None only if you need to check bootstrap status first
pub fn get_root_user() -> u64 {
    ROOT_USER_ID
}

/// Get the system ID (error if not bootstrapped)
pub fn get_system() -> Result<u64> {
    if is_bootstrapped()? {
        Ok(SYSTEM_ID)
    } else {
        Err(CapbitError("Not bootstrapped".into()))
    }
}

/// Bootstrap the system. Returns (system_id, root_user_id).
pub fn bootstrap() -> Result<(u64, u64)> {
    if is_bootstrapped()? {
        return Err(CapbitError("Already bootstrapped".into()));
    }
    transact(|tx| {
        tx.set_label(SYSTEM_ID, "_system")?;
        tx.set_label(ROOT_USER_ID, "_root_user")?;
        // Define roles on _system for granular inheritance
        tx.set_role(SYSTEM_ID, ROLE_GRANTER, GRANT)?;
        tx.set_role(SYSTEM_ID, ROLE_ADMIN, ADMIN)?;
        tx.set_role(SYSTEM_ID, ROLE_VIEWER, VIEW)?;
        tx.set_role(SYSTEM_ID, ROLE_FULL, GRANT | ADMIN | VIEW)?;
        // Root user gets full access
        tx.grant(ROOT_USER_ID, SYSTEM_ID, ROLE_FULL)?;
        tx.set_next_id(3)?;
        Ok((SYSTEM_ID, ROOT_USER_ID))
    })
}
