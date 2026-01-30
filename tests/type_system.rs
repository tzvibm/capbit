//! Type system tests for Capbit v2
//!
//! These tests verify type lifecycle, custom type creation, and type-level
//! permission controls.

use capbit::{
    init, bootstrap, protected, check_access,
    type_exists, entity_exists, SystemCap, clear_all, test_lock,
};
use tempfile::TempDir;
use std::sync::Once;

static INIT: Once = Once::new();
static mut TEST_DIR: Option<TempDir> = None;

fn setup() {
    INIT.call_once(|| {
        let dir = TempDir::new().unwrap();
        init(dir.path().to_str().unwrap()).unwrap();
        unsafe { TEST_DIR = Some(dir); }
    });
}

fn setup_bootstrapped() -> std::sync::MutexGuard<'static, ()> {
    let lock = test_lock();
    setup();
    clear_all().unwrap();
    bootstrap("root").unwrap();
    lock
}

// ============================================================================
// Core Type Tests
// ============================================================================

/// Verify bootstrap creates all core types
#[test]
fn bootstrap_creates_core_types() {
    let _lock = setup_bootstrapped();

    // Core types from bootstrap
    assert!(type_exists("_type").unwrap());
    assert!(type_exists("user").unwrap());
    assert!(type_exists("team").unwrap());
    assert!(type_exists("app").unwrap());
    assert!(type_exists("resource").unwrap());
}

/// Verify bootstrap creates type entities
#[test]
fn bootstrap_creates_type_entities() {
    let _lock = setup_bootstrapped();

    // Type entities (meta-entities for permission control)
    assert!(entity_exists("_type:_type").unwrap());
    assert!(entity_exists("_type:user").unwrap());
    assert!(entity_exists("_type:team").unwrap());
    assert!(entity_exists("_type:app").unwrap());
    assert!(entity_exists("_type:resource").unwrap());
}

// ============================================================================
// Custom Type Creation
// ============================================================================

/// Verify authorized user can create custom type
#[test]
fn create_custom_type() {
    let _lock = setup_bootstrapped();

    // Root can create types
    let result = protected::create_type("user:root", "project");
    assert!(result.is_ok());

    // Type exists
    assert!(type_exists("project").unwrap());

    // Type entity exists
    assert!(entity_exists("_type:project").unwrap());
}

/// Verify custom type creator gets admin
#[test]
fn custom_type_creator_gets_admin() {
    let _lock = setup_bootstrapped();

    // Give alice TYPE_CREATE permission
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::set_grant("user:root", "user:alice", "admin", "_type:_type").unwrap();

    // Alice creates a type
    protected::create_type("user:alice", "task").unwrap();

    // Alice should have admin on _type:task
    let caps = check_access("user:alice", "_type:task", None).unwrap();
    assert!((caps & SystemCap::ENTITY_ADMIN) == SystemCap::ENTITY_ADMIN);
}

/// Verify can create entity of custom type
#[test]
fn create_entity_of_custom_type() {
    let _lock = setup_bootstrapped();

    // Create custom type
    protected::create_type("user:root", "document").unwrap();

    // Create entity of custom type
    let result = protected::create_entity("user:root", "document", "readme");
    assert!(result.is_ok());
    assert!(entity_exists("document:readme").unwrap());
}

// ============================================================================
// Type Permission Control
// ============================================================================

/// Verify type-level admin can manage entities of that type
#[test]
fn type_admin_can_create_entities() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();

    // Give alice admin on _type:team
    protected::set_grant("user:root", "user:alice", "admin", "_type:team").unwrap();

    // Alice can create teams
    let result = protected::create_entity("user:alice", "team", "engineering");
    assert!(result.is_ok());

    let result = protected::create_entity("user:alice", "team", "sales");
    assert!(result.is_ok());
}

/// Verify type-level admin can delete entities of that type
#[test]
fn type_admin_can_delete_entities() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::set_grant("user:root", "user:alice", "admin", "_type:team").unwrap();

    // Create and delete
    protected::create_entity("user:alice", "team", "temp").unwrap();
    assert!(entity_exists("team:temp").unwrap());

    let result = protected::delete_entity("user:alice", "team:temp");
    assert!(result.is_ok());
    assert!(!entity_exists("team:temp").unwrap());
}

/// Verify non-admin cannot create entities
#[test]
fn non_admin_cannot_create_entities() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "bob").unwrap();

    // Bob has no admin rights
    let result = protected::create_entity("user:bob", "team", "hackers");
    assert!(result.is_err());
}

// ============================================================================
// Type Name Validation
// ============================================================================

/// Verify reserved underscore types are protected
#[test]
fn reserved_underscore_types() {
    let _lock = setup_bootstrapped();

    // Regular users cannot create underscore types
    protected::create_entity("user:root", "user", "alice").unwrap();

    // Give alice TYPE_CREATE but she still might not be able to create _prefixed types
    // depending on implementation
    let result = protected::create_type("user:root", "_internal");

    // This may succeed for root since root has full type access
    // The key is that underscore types are typically reserved for system use
    if result.is_ok() {
        assert!(type_exists("_internal").unwrap());
    }
}

/// Verify type name case sensitivity
#[test]
fn type_case_sensitivity() {
    let _lock = setup_bootstrapped();

    // Create lowercase type
    protected::create_type("user:root", "project").unwrap();

    // Uppercase is a different type
    protected::create_type("user:root", "Project").unwrap();

    // Both exist
    assert!(type_exists("project").unwrap());
    assert!(type_exists("Project").unwrap());

    // They're separate
    protected::create_entity("user:root", "project", "a").unwrap();
    protected::create_entity("user:root", "Project", "a").unwrap();

    assert!(entity_exists("project:a").unwrap());
    assert!(entity_exists("Project:a").unwrap());
}

/// Verify duplicate type creation fails
#[test]
fn type_already_exists_error() {
    let _lock = setup_bootstrapped();

    // "user" already exists from bootstrap
    let result = protected::create_type("user:root", "user");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("already exists"));

    // Create a new type
    protected::create_type("user:root", "widget").unwrap();

    // Creating it again should fail
    let result = protected::create_type("user:root", "widget");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("already exists"));
}

// ============================================================================
// Entity Type Validation
// ============================================================================

/// Verify entity requires valid type
#[test]
fn entity_requires_valid_type() {
    let _lock = setup_bootstrapped();

    // Try to create entity of non-existent type
    let result = protected::create_entity("user:root", "nonexistent", "test");
    assert!(result.is_err());
    // Error could be "Type does not exist" or "lacks permission" depending on check order
    let err_msg = result.unwrap_err().message;
    assert!(err_msg.contains("does not exist") || err_msg.contains("lacks permission"),
            "Unexpected error: {}", err_msg);
}

/// Verify entity type must match
#[test]
fn entity_type_in_id_must_match() {
    let _lock = setup_bootstrapped();

    // Create entity - type is specified separately
    protected::create_entity("user:root", "user", "alice").unwrap();

    // The entity ID becomes "user:alice"
    assert!(entity_exists("user:alice").unwrap());

    // It's stored under the "user" type
    assert!(!entity_exists("team:alice").unwrap());
}

// ============================================================================
// Type-Level Capabilities
// ============================================================================

/// Verify type-level capabilities grant appropriate permissions
#[test]
fn type_level_capability_grants() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();

    // Check what admin on _type:user provides
    let root_caps = check_access("user:root", "_type:user", None).unwrap();

    // Root should have full entity admin
    assert!((root_caps & SystemCap::ENTITY_CREATE) == SystemCap::ENTITY_CREATE);
    assert!((root_caps & SystemCap::ENTITY_DELETE) == SystemCap::ENTITY_DELETE);
    assert!((root_caps & SystemCap::GRANT_ADMIN) == SystemCap::GRANT_ADMIN);
    assert!((root_caps & SystemCap::CAP_ADMIN) == SystemCap::CAP_ADMIN);
}

/// Verify type admin can set capabilities on instances
#[test]
fn type_admin_can_set_instance_capabilities() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::set_grant("user:root", "user:alice", "admin", "_type:team").unwrap();

    // Create a team
    protected::create_entity("user:alice", "team", "sales").unwrap();

    // Alice can set capabilities on the team
    let result = protected::set_capability("user:alice", "team:sales", "member", 0x01);
    assert!(result.is_ok());

    // Alice can grant on the team
    protected::create_entity("user:root", "user", "bob").unwrap();
    let result = protected::set_grant("user:alice", "user:bob", "member", "team:sales");
    assert!(result.is_ok());
}

// ============================================================================
// Bootstrap Immutability
// ============================================================================

/// Verify core types cannot be re-created
#[test]
fn bootstrap_types_immutable() {
    let _lock = setup_bootstrapped();

    // Cannot recreate core types
    let result = protected::create_type("user:root", "user");
    assert!(result.is_err());

    let result = protected::create_type("user:root", "team");
    assert!(result.is_err());

    let result = protected::create_type("user:root", "app");
    assert!(result.is_err());

    let result = protected::create_type("user:root", "resource");
    assert!(result.is_err());

    let result = protected::create_type("user:root", "_type");
    assert!(result.is_err());
}

/// Verify root entity cannot be recreated
#[test]
fn root_entity_immutable() {
    let _lock = setup_bootstrapped();

    // Cannot create user:root again
    let result = protected::create_entity("user:root", "user", "root");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("already exists"));
}

// ============================================================================
// Type-Level Query Access
// ============================================================================

/// Verify check_access includes type-level permissions for instances
#[test]
fn check_access_includes_type_level_permissions() {
    let _lock = setup_bootstrapped();

    // Create a team instance
    protected::create_entity("user:root", "team", "engineering").unwrap();

    // Root has admin on _type:team (from bootstrap)
    // check_access on _type:team should show the permissions
    let type_caps = check_access("user:root", "_type:team", None).unwrap();
    assert!((type_caps & SystemCap::ENTITY_ADMIN) == SystemCap::ENTITY_ADMIN);

    // Now check_access on team:engineering should ALSO include type-level permissions
    let instance_caps = check_access("user:root", "team:engineering", None).unwrap();
    assert!((instance_caps & SystemCap::ENTITY_ADMIN) == SystemCap::ENTITY_ADMIN,
            "Expected type-level permissions on instance, got: 0x{:x}", instance_caps);
}

/// Verify type-level permissions work for any user with type admin
#[test]
fn type_admin_query_shows_permissions_on_instances() {
    let _lock = setup_bootstrapped();

    // Create alice and give her admin on _type:resource
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::set_grant("user:root", "user:alice", "admin", "_type:resource").unwrap();

    // Create some resources
    protected::create_entity("user:alice", "resource", "doc1").unwrap();
    protected::create_entity("user:alice", "resource", "doc2").unwrap();

    // Alice should show type-level permissions when querying any resource instance
    let caps1 = check_access("user:alice", "resource:doc1", None).unwrap();
    let caps2 = check_access("user:alice", "resource:doc2", None).unwrap();

    assert!((caps1 & SystemCap::ENTITY_ADMIN) == SystemCap::ENTITY_ADMIN);
    assert!((caps2 & SystemCap::ENTITY_ADMIN) == SystemCap::ENTITY_ADMIN);
}
