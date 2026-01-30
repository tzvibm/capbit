//! Authorized operations tests for Capbit v2
//!
//! These tests verify that properly authorized clients CAN perform all
//! operations they are permitted to do. This is the "happy path" complement
//! to attack vector tests.

use capbit::{
    init, bootstrap, protected, check_access, has_capability,
    list_accessible, list_subjects, get_relationships, get_inheritance,
    entity_exists, type_exists, SystemCap, clear_all, test_lock,
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
// Entity CRUD Operations
// ============================================================================

/// Verify authorized user can create entities of permitted type
#[test]
fn authorized_user_can_create_entity() {
    let _lock = setup_bootstrapped();

    // Give alice admin on _type:user
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::set_grant("user:root", "user:alice", "admin", "_type:user").unwrap();

    // Alice can create users
    let result = protected::create_entity("user:alice", "user", "bob");
    assert!(result.is_ok());
    assert!(entity_exists("user:bob").unwrap());

    // Alice can create multiple users
    protected::create_entity("user:alice", "user", "charlie").unwrap();
    protected::create_entity("user:alice", "user", "david").unwrap();
    assert!(entity_exists("user:charlie").unwrap());
    assert!(entity_exists("user:david").unwrap());
}

/// Verify authorized user can delete entities of permitted type
#[test]
fn authorized_user_can_delete_entity() {
    let _lock = setup_bootstrapped();

    // Setup: alice is user admin
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::set_grant("user:root", "user:alice", "admin", "_type:user").unwrap();

    // Create a user to delete
    protected::create_entity("user:alice", "user", "temp").unwrap();
    assert!(entity_exists("user:temp").unwrap());

    // Alice can delete
    let result = protected::delete_entity("user:alice", "user:temp");
    assert!(result.is_ok());
    assert!(!entity_exists("user:temp").unwrap());
}

// ============================================================================
// Grant Operations
// ============================================================================

/// Verify authorized user can set grants on permitted scope
#[test]
fn authorized_user_can_set_grant() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "team", "sales").unwrap();

    // Give alice GRANT_ADMIN on team:sales
    protected::set_capability("user:root", "team:sales", "admin", SystemCap::GRANT_ADMIN).unwrap();
    protected::set_grant("user:root", "user:alice", "admin", "team:sales").unwrap();

    // Define what "member" means
    protected::set_capability("user:root", "team:sales", "member", 0x01).unwrap();

    // Alice can grant bob membership
    let result = protected::set_grant("user:alice", "user:bob", "member", "team:sales");
    assert!(result.is_ok());

    // Verify bob has the grant
    assert!(has_capability("user:bob", "team:sales", 0x01).unwrap());
}

/// Verify authorized user can delete grants on permitted scope
#[test]
fn authorized_user_can_delete_grant() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "team", "sales").unwrap();

    // Setup: alice has GRANT_ADMIN, bob has membership
    protected::set_capability("user:root", "team:sales", "admin", SystemCap::GRANT_ADMIN).unwrap();
    protected::set_capability("user:root", "team:sales", "member", 0x01).unwrap();
    protected::set_grant("user:root", "user:alice", "admin", "team:sales").unwrap();
    protected::set_grant("user:root", "user:bob", "member", "team:sales").unwrap();

    assert!(has_capability("user:bob", "team:sales", 0x01).unwrap());

    // Alice can revoke bob's membership
    let result = protected::delete_grant("user:alice", "user:bob", "member", "team:sales");
    assert!(result.is_ok());
    assert!(!has_capability("user:bob", "team:sales", 0x01).unwrap());
}

// ============================================================================
// Capability Operations
// ============================================================================

/// Verify authorized user can set capability definitions
#[test]
fn authorized_user_can_set_capability() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Give alice CAP_ADMIN on resource:doc
    protected::set_capability("user:root", "resource:doc", "admin", SystemCap::CAP_ADMIN).unwrap();
    protected::set_grant("user:root", "user:alice", "admin", "resource:doc").unwrap();

    // Alice can define new capabilities
    let result = protected::set_capability("user:alice", "resource:doc", "viewer", 0x01);
    assert!(result.is_ok());

    let result = protected::set_capability("user:alice", "resource:doc", "editor", 0x03);
    assert!(result.is_ok());

    let result = protected::set_capability("user:alice", "resource:doc", "owner", 0x0F);
    assert!(result.is_ok());

    // Verify capabilities are set by granting and checking
    protected::set_grant("user:root", "user:alice", "viewer", "resource:doc").unwrap();
    assert!(has_capability("user:alice", "resource:doc", 0x01).unwrap());
}

// ============================================================================
// Delegation Operations
// ============================================================================

/// Verify authorized user can create delegations
#[test]
fn authorized_user_can_delegate() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Give alice access and delegation rights
    protected::set_capability("user:root", "resource:doc", "editor", 0x0F).unwrap();
    protected::set_capability("user:root", "resource:doc", "delegator", SystemCap::DELEGATE_WRITE).unwrap();
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();
    protected::set_grant("user:root", "user:alice", "delegator", "resource:doc").unwrap();

    // Alice can delegate to bob
    let result = protected::set_delegation("user:alice", "user:bob", "resource:doc", "user:alice");
    assert!(result.is_ok());

    // Bob now has alice's capabilities
    assert!(has_capability("user:bob", "resource:doc", 0x0F).unwrap());
}

/// Verify authorized user can delete delegations
#[test]
fn authorized_user_can_delete_delegation() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Give alice full delegation control
    protected::set_capability("user:root", "resource:doc", "editor", 0x0F).unwrap();
    protected::set_capability("user:root", "resource:doc", "delegator", SystemCap::DELEGATE_ADMIN).unwrap();
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();
    protected::set_grant("user:root", "user:alice", "delegator", "resource:doc").unwrap();

    // Create and verify delegation
    protected::set_delegation("user:alice", "user:bob", "resource:doc", "user:alice").unwrap();
    assert!(has_capability("user:bob", "resource:doc", 0x0F).unwrap());

    // Alice can remove the delegation
    let result = protected::delete_delegation("user:alice", "user:bob", "resource:doc", "user:alice");
    assert!(result.is_ok());
    assert!(!has_capability("user:bob", "resource:doc", 0x0F).unwrap());
}

// ============================================================================
// Type-Level Administration
// ============================================================================

/// Verify type admin can manage all instances of that type
#[test]
fn type_admin_can_manage_all_instances() {
    let _lock = setup_bootstrapped();

    // Alice is team admin (via _type:team)
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::set_grant("user:root", "user:alice", "admin", "_type:team").unwrap();

    // Alice can create any team
    protected::create_entity("user:alice", "team", "sales").unwrap();
    protected::create_entity("user:alice", "team", "engineering").unwrap();
    protected::create_entity("user:alice", "team", "marketing").unwrap();

    assert!(entity_exists("team:sales").unwrap());
    assert!(entity_exists("team:engineering").unwrap());
    assert!(entity_exists("team:marketing").unwrap());

    // Alice can delete any team
    protected::delete_entity("user:alice", "team:marketing").unwrap();
    assert!(!entity_exists("team:marketing").unwrap());

    // Alice can set capabilities on any team
    protected::set_capability("user:alice", "team:sales", "member", 0x01).unwrap();
    protected::set_capability("user:alice", "team:engineering", "member", 0x01).unwrap();

    // Alice can grant on any team
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::set_grant("user:alice", "user:bob", "member", "team:sales").unwrap();
    protected::set_grant("user:alice", "user:bob", "member", "team:engineering").unwrap();

    assert!(has_capability("user:bob", "team:sales", 0x01).unwrap());
    assert!(has_capability("user:bob", "team:engineering", 0x01).unwrap());
}

// ============================================================================
// Delegation Chain Access
// ============================================================================

/// Verify delegatee can access resources through delegation chain
#[test]
fn delegatee_can_access_delegated_scope() {
    let _lock = setup_bootstrapped();

    // Create a 3-level delegation chain: alice -> bob -> charlie
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "user", "charlie").unwrap();
    protected::create_entity("user:root", "resource", "secret").unwrap();

    // Alice has full access
    protected::set_capability("user:root", "resource:secret", "admin", 0xFF | SystemCap::DELEGATE_WRITE).unwrap();
    protected::set_grant("user:root", "user:alice", "admin", "resource:secret").unwrap();

    // Bob gets delegation from alice, give bob delegation rights too
    protected::set_grant("user:root", "user:bob", "admin", "resource:secret").unwrap();
    protected::set_delegation("user:alice", "user:bob", "resource:secret", "user:alice").unwrap();

    // Charlie gets delegation from bob
    protected::set_delegation("user:bob", "user:charlie", "resource:secret", "user:bob").unwrap();

    // All three can access the resource
    assert!(has_capability("user:alice", "resource:secret", 0xFF).unwrap());
    assert!(has_capability("user:bob", "resource:secret", 0xFF).unwrap());
    assert!(has_capability("user:charlie", "resource:secret", 0xFF).unwrap());
}

// ============================================================================
// Multi-Role Access
// ============================================================================

/// Verify user with multiple roles has combined access
#[test]
fn multi_role_user_has_combined_access() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Define multiple non-overlapping roles
    protected::set_capability("user:root", "resource:doc", "viewer", 0x01).unwrap();     // bit 0
    protected::set_capability("user:root", "resource:doc", "commenter", 0x02).unwrap();  // bit 1
    protected::set_capability("user:root", "resource:doc", "editor", 0x04).unwrap();     // bit 2
    protected::set_capability("user:root", "resource:doc", "admin", 0x08).unwrap();      // bit 3

    // Grant alice multiple roles
    protected::set_grant("user:root", "user:alice", "viewer", "resource:doc").unwrap();
    protected::set_grant("user:root", "user:alice", "commenter", "resource:doc").unwrap();
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();

    // Alice has combined capabilities (0x07 = viewer + commenter + editor)
    let caps = check_access("user:alice", "resource:doc", None).unwrap();
    assert_eq!(caps, 0x07);

    // Alice can use any individual capability
    assert!(has_capability("user:alice", "resource:doc", 0x01).unwrap()); // viewer
    assert!(has_capability("user:alice", "resource:doc", 0x02).unwrap()); // commenter
    assert!(has_capability("user:alice", "resource:doc", 0x04).unwrap()); // editor

    // Alice cannot use admin (not granted)
    assert!(!has_capability("user:alice", "resource:doc", 0x08).unwrap());

    // Alice can use any combination of her roles
    assert!(has_capability("user:alice", "resource:doc", 0x03).unwrap()); // viewer + commenter
    assert!(has_capability("user:alice", "resource:doc", 0x07).unwrap()); // all three
}

// ============================================================================
// Query Operations
// ============================================================================

/// Verify user can query all resources they have access to
#[test]
fn user_can_query_accessible_resources() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc1").unwrap();
    protected::create_entity("user:root", "resource", "doc2").unwrap();
    protected::create_entity("user:root", "resource", "doc3").unwrap();
    protected::create_entity("user:root", "team", "sales").unwrap();

    // Grant alice access to various resources
    protected::set_capability("user:root", "resource:doc1", "viewer", 0x01).unwrap();
    protected::set_capability("user:root", "resource:doc2", "editor", 0x03).unwrap();
    protected::set_capability("user:root", "team:sales", "member", 0x01).unwrap();

    protected::set_grant("user:root", "user:alice", "viewer", "resource:doc1").unwrap();
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc2").unwrap();
    protected::set_grant("user:root", "user:alice", "member", "team:sales").unwrap();

    // Query all accessible resources
    let accessible = list_accessible("user:alice").unwrap();

    // Should have access to doc1, doc2, and team:sales (not doc3)
    let objects: Vec<&str> = accessible.iter().map(|(obj, _)| obj.as_str()).collect();
    assert!(objects.contains(&"resource:doc1"));
    assert!(objects.contains(&"resource:doc2"));
    assert!(objects.contains(&"team:sales"));
    assert!(!objects.contains(&"resource:doc3"));
}

/// Verify user can query who has access to a resource they manage
#[test]
fn user_can_query_who_has_access() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "user", "charlie").unwrap();
    protected::create_entity("user:root", "team", "sales").unwrap();

    // Grant various users access to sales
    protected::set_capability("user:root", "team:sales", "member", 0x01).unwrap();
    protected::set_capability("user:root", "team:sales", "admin", 0x0F).unwrap();

    protected::set_grant("user:root", "user:alice", "admin", "team:sales").unwrap();
    protected::set_grant("user:root", "user:bob", "member", "team:sales").unwrap();
    protected::set_grant("user:root", "user:charlie", "member", "team:sales").unwrap();

    // Query who has access
    let subjects = list_subjects("team:sales").unwrap();

    let users: Vec<&str> = subjects.iter().map(|(subj, _)| subj.as_str()).collect();
    assert!(users.contains(&"user:alice"));
    assert!(users.contains(&"user:bob"));
    assert!(users.contains(&"user:charlie"));
}

// ============================================================================
// Inherited Permissions
// ============================================================================

/// Verify inherited permissions grant proper access
#[test]
fn inherited_permissions_grant_access() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Alice has direct access
    protected::set_capability("user:root", "resource:doc", "editor", 0x0F).unwrap();
    protected::set_capability("user:root", "resource:doc", "delegator", SystemCap::DELEGATE_WRITE).unwrap();
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();
    protected::set_grant("user:root", "user:alice", "delegator", "resource:doc").unwrap();

    // Bob inherits from alice
    protected::set_delegation("user:alice", "user:bob", "resource:doc", "user:alice").unwrap();

    // Bob can use inherited permissions
    let bob_caps = check_access("user:bob", "resource:doc", None).unwrap();
    assert!(bob_caps & 0x0F == 0x0F); // Has editor capabilities

    // Verify inheritance is recorded
    let inheritance = get_inheritance("user:bob", "resource:doc").unwrap();
    assert!(inheritance.contains(&"user:alice".to_string()));
}

// ============================================================================
// Partial Capabilities
// ============================================================================

/// Verify user with partial caps can use exactly what they have
#[test]
fn user_with_partial_caps_can_use_them() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Define a role with specific bits
    protected::set_capability("user:root", "resource:doc", "limited", 0x05).unwrap(); // bits 0 and 2

    protected::set_grant("user:root", "user:alice", "limited", "resource:doc").unwrap();

    // Alice has exactly bits 0 and 2
    let caps = check_access("user:alice", "resource:doc", None).unwrap();
    assert_eq!(caps, 0x05);

    // Can use bit 0
    assert!(has_capability("user:alice", "resource:doc", 0x01).unwrap());
    // Can use bit 2
    assert!(has_capability("user:alice", "resource:doc", 0x04).unwrap());
    // Can use both together
    assert!(has_capability("user:alice", "resource:doc", 0x05).unwrap());

    // Cannot use bit 1 (not granted)
    assert!(!has_capability("user:alice", "resource:doc", 0x02).unwrap());
    // Cannot use bit 1 + anything else
    assert!(!has_capability("user:alice", "resource:doc", 0x03).unwrap());
    assert!(!has_capability("user:alice", "resource:doc", 0x07).unwrap());
}

// ============================================================================
// Custom Type Management
// ============================================================================

/// Verify custom type creator can manage their type
#[test]
fn custom_type_creator_can_manage_type() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();

    // Give alice TYPE_CREATE permission
    protected::set_grant("user:root", "user:alice", "admin", "_type:_type").unwrap();

    // Alice creates a custom type
    let result = protected::create_type("user:alice", "project");
    assert!(result.is_ok());
    assert!(type_exists("project").unwrap());

    // Alice automatically gets admin on _type:project
    let alice_caps = check_access("user:alice", "_type:project", None).unwrap();
    assert!((alice_caps & SystemCap::ENTITY_ADMIN) == SystemCap::ENTITY_ADMIN);

    // Alice can create project entities
    let result = protected::create_entity("user:alice", "project", "alpha");
    assert!(result.is_ok());
    assert!(entity_exists("project:alpha").unwrap());

    // Alice can manage the project
    protected::set_capability("user:alice", "project:alpha", "member", 0x01).unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::set_grant("user:alice", "user:bob", "member", "project:alpha").unwrap();

    assert!(has_capability("user:bob", "project:alpha", 0x01).unwrap());
}

// ============================================================================
// Cross-Scope Operations
// ============================================================================

/// Verify user can manage multiple scopes they have access to
#[test]
fn user_can_manage_multiple_scopes() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "team", "sales").unwrap();
    protected::create_entity("user:root", "team", "marketing").unwrap();
    protected::create_entity("user:root", "resource", "doc1").unwrap();

    // Alice is admin on multiple scopes
    protected::set_capability("user:root", "team:sales", "admin", SystemCap::GRANT_ADMIN).unwrap();
    protected::set_capability("user:root", "team:marketing", "admin", SystemCap::GRANT_ADMIN).unwrap();
    protected::set_capability("user:root", "resource:doc1", "admin", SystemCap::GRANT_ADMIN).unwrap();

    protected::set_grant("user:root", "user:alice", "admin", "team:sales").unwrap();
    protected::set_grant("user:root", "user:alice", "admin", "team:marketing").unwrap();
    protected::set_grant("user:root", "user:alice", "admin", "resource:doc1").unwrap();

    // Define member roles
    protected::set_capability("user:root", "team:sales", "member", 0x01).unwrap();
    protected::set_capability("user:root", "team:marketing", "member", 0x02).unwrap();
    protected::set_capability("user:root", "resource:doc1", "viewer", 0x04).unwrap();

    // Alice can grant bob access to all her scopes
    protected::set_grant("user:alice", "user:bob", "member", "team:sales").unwrap();
    protected::set_grant("user:alice", "user:bob", "member", "team:marketing").unwrap();
    protected::set_grant("user:alice", "user:bob", "viewer", "resource:doc1").unwrap();

    // Bob has access to all three
    assert!(has_capability("user:bob", "team:sales", 0x01).unwrap());
    assert!(has_capability("user:bob", "team:marketing", 0x02).unwrap());
    assert!(has_capability("user:bob", "resource:doc1", 0x04).unwrap());
}

/// Verify relationships are correctly recorded and queryable
#[test]
fn relationships_are_queryable() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "team", "sales").unwrap();

    // Grant multiple relations
    protected::set_capability("user:root", "team:sales", "member", 0x01).unwrap();
    protected::set_capability("user:root", "team:sales", "lead", 0x03).unwrap();
    protected::set_capability("user:root", "team:sales", "admin", 0x0F).unwrap();

    protected::set_grant("user:root", "user:alice", "member", "team:sales").unwrap();
    protected::set_grant("user:root", "user:alice", "lead", "team:sales").unwrap();

    // Query alice's relationships to team:sales
    let relations = get_relationships("user:alice", "team:sales").unwrap();

    assert!(relations.contains(&"member".to_string()));
    assert!(relations.contains(&"lead".to_string()));
    assert!(!relations.contains(&"admin".to_string())); // not granted
}
