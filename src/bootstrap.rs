//! Bootstrap/genesis logic for Capbit v2
//!
//! Creates the initial root user and system types.

use crate::caps::SystemCap;
use crate::core::{
    self, CapbitError, Result,
    create_type_in, create_entity_in, set_meta_in,
    _set_capability_in, _set_relationship_in,
    with_write_txn_pub,
};

/// Core entity types created at bootstrap
const CORE_TYPES: &[&str] = &["user", "team", "app", "resource"];

/// Check if the system has been bootstrapped
pub fn is_bootstrapped() -> Result<bool> {
    core::get_meta("bootstrapped").map(|v| v.as_deref() == Some("true"))
}

/// Bootstrap the system with a root user.
///
/// This is the ONLY operation that runs without permission checks.
/// After bootstrap, all mutations require authorization.
///
/// # Arguments
/// * `root_id` - The identifier for the root user (e.g., "root" becomes "user:root")
///
/// # Example
/// ```rust,no_run
/// capbit::bootstrap("root").unwrap();
/// // Now user:root has full system access
/// ```
pub fn bootstrap(root_id: &str) -> Result<u64> {
    // Check not already bootstrapped (before acquiring write lock)
    if is_bootstrapped()? {
        return Err(CapbitError { message: "System already bootstrapped".into() });
    }

    with_write_txn_pub(|txn, dbs| {

        // 1. Create the meta-type (type of types)
        create_type_in(txn, dbs, "_type")?;

        // 2. Create core entity types
        for t in CORE_TYPES {
            create_type_in(txn, dbs, t)?;
        }

        // 3. Create type entities (for permission control)
        let meta_type = "_type:_type";
        create_entity_in(txn, dbs, meta_type)?;

        for t in CORE_TYPES {
            let type_entity = format!("_type:{}", t);
            create_entity_in(txn, dbs, &type_entity)?;
        }

        // 4. Define capabilities on type entities
        // Admin on _type:_type can create/delete types
        _set_capability_in(txn, dbs, meta_type, "admin", SystemCap::TYPE_ADMIN)?;

        // Admin on _type:{type} can create/delete entities of that type
        // _type:user admin also gets PASSWORD_ADMIN for credential management
        for t in CORE_TYPES {
            let type_entity = format!("_type:{}", t);
            let caps = if *t == "user" {
                SystemCap::ENTITY_ADMIN | SystemCap::PASSWORD_ADMIN
            } else {
                SystemCap::ENTITY_ADMIN
            };
            _set_capability_in(txn, dbs, &type_entity, "admin", caps)?;
        }

        // 5. Create root user entity
        let root_entity = format!("user:{}", root_id);
        create_entity_in(txn, dbs, &root_entity)?;

        // 6. Grant root admin on all type entities
        _set_relationship_in(txn, dbs, &root_entity, "admin", meta_type)?;
        for t in CORE_TYPES {
            let type_entity = format!("_type:{}", t);
            _set_relationship_in(txn, dbs, &root_entity, "admin", &type_entity)?;
        }

        // 7. Define SystemCap bit labels on _type:_type
        let syscap_labels = [
            (0, "type-create"),
            (1, "type-delete"),
            (2, "entity-create"),
            (3, "entity-delete"),
            (4, "grant-read"),
            (5, "grant-write"),
            (6, "grant-delete"),
            (7, "cap-read"),
            (8, "cap-write"),
            (9, "cap-delete"),
            (10, "delegate-read"),
            (11, "delegate-write"),
            (12, "delegate-delete"),
            (13, "system-read"),
            (14, "password-admin"),
        ];
        for (bit, label) in syscap_labels {
            core::set_cap_label_in(txn, dbs, "_type:_type", bit, label)?;
        }

        // 8. Mark as bootstrapped
        let epoch = core::current_epoch_pub();
        set_meta_in(txn, dbs, "bootstrapped", "true")?;
        set_meta_in(txn, dbs, "bootstrap_epoch", &epoch.to_string())?;
        set_meta_in(txn, dbs, "root_entity", &root_entity)?;

        Ok(epoch)
    })
}

/// Get the root entity ID
pub fn get_root_entity() -> Result<Option<String>> {
    core::get_meta("root_entity")
}
