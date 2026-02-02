//! Entity management (no protection - entities are just IDs)

use crate::error::Result;
use crate::tx::transact;

/// Create a new entity with a name
pub fn create_entity(name: &str) -> Result<u64> {
    transact(|tx| tx.create_entity(name))
}

/// Rename an existing entity
pub fn rename_entity(id: u64, new_name: &str) -> Result<()> {
    transact(|tx| tx.rename_entity(id, new_name))
}

/// Delete an entity
pub fn delete_entity(id: u64) -> Result<bool> {
    transact(|tx| tx.delete_entity(id))
}

/// Set or update an entity's label
pub fn set_label(id: u64, name: &str) -> Result<()> {
    transact(|tx| tx.set_label(id, name))
}
