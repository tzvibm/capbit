//! Capbit - Minimal capability-based access control

mod bootstrap;
mod constants;
mod db;
mod entity;
mod error;
mod planner;
mod read;
mod tx;
mod write;

// Re-export error types
pub use error::{CapbitError, Result};

// Re-export constants
pub use constants::{
    caps_to_names, names_to_caps,
    READ, WRITE, DELETE, CREATE, GRANT, EXECUTE, VIEW, ADMIN,
    SYSTEM_ID, ROOT_USER_ID,
    ROLE_GRANTER, ROLE_ADMIN, ROLE_VIEWER, ROLE_FULL,
    MAX_INHERITANCE_DEPTH,
};

// Re-export init
pub use db::{init, clear_all, test_lock};

// Re-export transaction API
pub use tx::{transact, Tx};

// Re-export bootstrap
pub use bootstrap::{bootstrap, is_bootstrapped, get_system, get_root_user};

// Re-export entity management
pub use entity::{create_entity, rename_entity, delete_entity, set_label};

// Re-export read operations
pub use read::{
    check, get_mask, get_role_id, get_role, get_inherit,
    list_for_subject, count_for_subject, count_for_object,
    get_label, get_id_by_label, list_labels,
};

// Re-export write operations (with permission checks)
pub use write::{
    grant, grant_set, revoke, set_role, set_inherit, remove_inherit,
    list_for_object,
};
