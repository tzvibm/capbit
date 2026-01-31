//! Protected API layer for Capbit v2
//!
//! All mutations require a requester with appropriate permissions.

use crate::caps::SystemCap;
use crate::core::{
    self, CapbitError, Result,
    parse_entity, create_entity_in, delete_entity_in, entity_exists_in_rw,
    _set_relationship_in, _delete_relationship_in,
    _set_capability_in, _set_inheritance_in, _delete_inheritance_in,
    with_write_txn_pub,
};
use crate::entity_id::EntityId;

// ============================================================================
// Permission Checks
// ============================================================================

/// Check if requester has required permissions on scope.
/// Also checks _type:{type} if scope is a typed entity.
fn check_permission(requester: &str, scope: &str, required: u64) -> Result<()> {
    // First check direct permissions on the scope
    let caps = core::check_access(requester, scope, None)?;
    if (caps & required) == required {
        return Ok(());
    }

    // If scope is a typed entity (e.g., "team:sales"), also check _type:team
    // Use EntityId for O(1) type extraction
    if !scope.starts_with("_type:") {
        if let Ok(eid) = parse_entity(scope) {
            let type_scope = eid.meta_type();
            let type_caps = core::check_access(requester, &type_scope.to_string(), None)?;
            if (type_caps & required) == required {
                return Ok(());
            }
        }
    }

    Err(CapbitError {
        message: format!(
            "{} lacks permission 0x{:04x} on {}",
            requester, required, scope
        ),
    })
}

// ============================================================================
// Entity Lifecycle
// ============================================================================

/// Create a new entity. Requires ENTITY_CREATE on _type:{type}.
pub fn create_entity(requester: &str, entity_type: &str, id: &str) -> Result<u64> {
    // Build EntityId upfront - validates format and enables O(1) access
    let eid = EntityId::new(entity_type, id)
        .map_err(|e| CapbitError { message: e.message })?;
    let type_scope = eid.meta_type();
    check_permission(requester, &type_scope.to_string(), SystemCap::ENTITY_CREATE)?;

    with_write_txn_pub(|txn, dbs| {
        create_entity_in(txn, dbs, &eid.to_string())
    })
}

/// Delete an entity. Requires ENTITY_DELETE on _type:{type}.
pub fn delete_entity(requester: &str, entity_id: &str) -> Result<bool> {
    // Parse into EntityId for O(1) type extraction
    let eid = parse_entity(entity_id)?;
    let type_scope = eid.meta_type();
    check_permission(requester, &type_scope.to_string(), SystemCap::ENTITY_DELETE)?;

    with_write_txn_pub(|txn, dbs| {
        delete_entity_in(txn, dbs, entity_id)
    })
}

// ============================================================================
// Grants (seeker/relation/scope)
// ============================================================================

/// Set a grant. Requires GRANT_WRITE on scope.
pub fn set_grant(requester: &str, seeker: &str, relation: &str, scope: &str) -> Result<u64> {
    check_permission(requester, scope, SystemCap::GRANT_WRITE)?;

    with_write_txn_pub(|txn, dbs| {
        // Validate scope exists (seeker can be external identity)
        if !scope.starts_with("_type:") && !entity_exists_in_rw(txn, dbs, scope)? {
            return Err(CapbitError {
                message: format!("Scope '{}' does not exist", scope),
            });
        }
        _set_relationship_in(txn, dbs, seeker, relation, scope)
    })
}

/// Delete a grant. Requires GRANT_DELETE on scope.
pub fn delete_grant(requester: &str, seeker: &str, relation: &str, scope: &str) -> Result<bool> {
    check_permission(requester, scope, SystemCap::GRANT_DELETE)?;

    with_write_txn_pub(|txn, dbs| {
        _delete_relationship_in(txn, dbs, seeker, relation, scope)
    })
}

// ============================================================================
// Capabilities
// ============================================================================

/// Set capability definition. Requires CAP_WRITE on scope.
pub fn set_capability(requester: &str, scope: &str, relation: &str, cap_mask: u64) -> Result<u64> {
    check_permission(requester, scope, SystemCap::CAP_WRITE)?;

    with_write_txn_pub(|txn, dbs| {
        _set_capability_in(txn, dbs, scope, relation, cap_mask)
    })
}

// ============================================================================
// Delegations
// ============================================================================

/// Set delegation. Requires DELEGATE_WRITE on scope.
pub fn set_delegation(requester: &str, seeker: &str, scope: &str, delegate: &str) -> Result<u64> {
    check_permission(requester, scope, SystemCap::DELEGATE_WRITE)?;

    with_write_txn_pub(|txn, dbs| {
        _set_inheritance_in(txn, dbs, seeker, scope, delegate)
    })
}

/// Delete delegation. Requires DELEGATE_DELETE on scope.
pub fn delete_delegation(requester: &str, seeker: &str, scope: &str, delegate: &str) -> Result<bool> {
    check_permission(requester, scope, SystemCap::DELEGATE_DELETE)?;

    with_write_txn_pub(|txn, dbs| {
        _delete_inheritance_in(txn, dbs, seeker, scope, delegate)
    })
}

// ============================================================================
// Type Management
// ============================================================================

/// Create a new entity type. Requires TYPE_CREATE on _type:_type.
pub fn create_type(requester: &str, type_name: &str) -> Result<u64> {
    check_permission(requester, "_type:_type", SystemCap::TYPE_CREATE)?;

    with_write_txn_pub(|txn, dbs| {
        // Create the type
        crate::core::create_type_in(txn, dbs, type_name)?;

        // Create the type entity for permission control
        let type_entity = format!("_type:{}", type_name);
        create_entity_in(txn, dbs, &type_entity)?;

        // Define default admin capability on the type
        _set_capability_in(txn, dbs, &type_entity, "admin", SystemCap::ENTITY_ADMIN)?;

        // Grant requester admin on the new type
        _set_relationship_in(txn, dbs, requester, "admin", &type_entity)?;

        Ok(crate::core::current_epoch_pub())
    })
}

// ============================================================================
// Protected Reads
// ============================================================================

/// List entities visible to requester.
/// - SYSTEM_READ on _type:_type → see all entities
/// - Otherwise, see entities you have any access to (including _type:* if you have grants there)
pub fn list_entities(requester: &str) -> Result<Vec<String>> {
    let all = core::list_all_entities()?;
    let has_system = (core::check_access(requester, "_type:_type", None)? & SystemCap::SYSTEM_READ) != 0;
    if has_system { return Ok(all); }
    // Check access to each entity - includes _type:* entities user has direct grants on
    Ok(all.into_iter().filter(|e| core::check_access(requester, e, None).unwrap_or(0) > 0).collect())
}

/// List grants visible to requester. Requires GRANT_READ on scope.
pub fn list_grants(requester: &str) -> Result<Vec<(String, String, String)>> {
    let all = core::list_all_grants()?;
    Ok(all.into_iter().filter(|(_, _, scope)| {
        (core::check_access(requester, scope, None).unwrap_or(0) & SystemCap::GRANT_READ) != 0
    }).collect())
}

/// List capabilities visible to requester. Requires CAP_READ on scope.
pub fn list_capabilities(requester: &str) -> Result<Vec<(String, String, u64)>> {
    let all = core::list_all_capabilities()?;
    Ok(all.into_iter().filter(|(scope, _, _)| {
        (core::check_access(requester, scope, None).unwrap_or(0) & SystemCap::CAP_READ) != 0
    }).collect())
}

/// List cap labels visible to requester. Requires CAP_READ on scope.
pub fn list_cap_labels(requester: &str) -> Result<Vec<(String, u64, String)>> {
    let all = core::list_all_cap_labels()?;
    Ok(all.into_iter().filter(|(scope, _, _)| {
        (core::check_access(requester, scope, None).unwrap_or(0) & SystemCap::CAP_READ) != 0
    }).collect())
}
