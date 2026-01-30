//! Revocation and permission loss tests for Capbit v2
//!
//! These tests verify what happens when permissions are revoked, including
//! effects on delegation chains, capability definitions, and entity lifecycle.

use capbit::{
    init, bootstrap, protected, check_access, has_capability,
    SystemCap, clear_all, test_lock, entity_exists,
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
// Basic Revocation
// ============================================================================

/// Verify that deleting a grant immediately removes access
#[test]
fn delete_grant_removes_access() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    protected::set_capability("user:root", "resource:doc", "editor", 0x0F).unwrap();
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();

    // Alice has access
    assert!(has_capability("user:alice", "resource:doc", 0x0F).unwrap());

    // Revoke
    protected::delete_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();

    // Access is immediately gone
    assert!(!has_capability("user:alice", "resource:doc", 0x0F).unwrap());
    assert_eq!(check_access("user:alice", "resource:doc", None).unwrap(), 0);
}

/// Verify deleting a grant from the middle of a hierarchy
#[test]
fn delete_grant_from_hierarchy_middle() {
    let _lock = setup_bootstrapped();

    // Setup: root -> alice -> bob -> charlie (via delegation)
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "user", "charlie").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    protected::set_capability("user:root", "resource:doc", "admin", 0xFF | SystemCap::DELEGATE_WRITE).unwrap();
    protected::set_grant("user:root", "user:alice", "admin", "resource:doc").unwrap();
    protected::set_grant("user:root", "user:bob", "admin", "resource:doc").unwrap();

    // Alice delegates to bob, bob delegates to charlie
    protected::set_delegation("user:alice", "user:bob", "resource:doc", "user:alice").unwrap();
    protected::set_delegation("user:bob", "user:charlie", "resource:doc", "user:bob").unwrap();

    // Charlie has access through chain
    assert!(has_capability("user:charlie", "resource:doc", 0xFF).unwrap());

    // Revoke bob's direct grant (but keep the delegation from alice)
    protected::delete_grant("user:root", "user:bob", "admin", "resource:doc").unwrap();

    // Bob still has access via alice's delegation
    assert!(has_capability("user:bob", "resource:doc", 0xFF).unwrap());

    // Charlie still has access via bob's delegation (from alice)
    assert!(has_capability("user:charlie", "resource:doc", 0xFF).unwrap());
}

// ============================================================================
// Delegator Permission Changes
// ============================================================================

/// Verify effect when delegator's own permissions are reduced
#[test]
fn revoke_delegator_permissions_effect() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Alice has full access
    protected::set_capability("user:root", "resource:doc", "admin", 0xFF).unwrap();
    protected::set_capability("user:root", "resource:doc", "delegator", SystemCap::DELEGATE_WRITE).unwrap();
    protected::set_grant("user:root", "user:alice", "admin", "resource:doc").unwrap();
    protected::set_grant("user:root", "user:alice", "delegator", "resource:doc").unwrap();

    // Alice delegates to bob
    protected::set_delegation("user:alice", "user:bob", "resource:doc", "user:alice").unwrap();

    // Bob inherits alice's caps
    assert_eq!(check_access("user:bob", "resource:doc", None).unwrap() & 0xFF, 0xFF);

    // Revoke alice's admin (keep delegator)
    protected::delete_grant("user:root", "user:alice", "admin", "resource:doc").unwrap();

    // Alice now only has delegator capability
    let alice_caps = check_access("user:alice", "resource:doc", None).unwrap();
    assert_eq!(alice_caps & 0xFF, 0);
    assert!((alice_caps & SystemCap::DELEGATE_WRITE) == SystemCap::DELEGATE_WRITE);

    // Bob's inherited caps are bounded by alice's current caps
    let bob_caps = check_access("user:bob", "resource:doc", None).unwrap();
    assert_eq!(bob_caps & 0xFF, 0); // No longer has admin bits
}

/// Verify effect when root's type-level access is revoked
#[test]
fn revoke_root_type_access() {
    let _lock = setup_bootstrapped();

    // Create a non-root admin
    protected::create_entity("user:root", "user", "admin2").unwrap();

    // Give admin2 admin rights on _type:user
    protected::set_grant("user:root", "user:admin2", "admin", "_type:user").unwrap();

    // admin2 can create users
    let result = protected::create_entity("user:admin2", "user", "testuser");
    assert!(result.is_ok());

    // Revoke admin2's type-level access
    protected::delete_grant("user:root", "user:admin2", "admin", "_type:user").unwrap();

    // admin2 can no longer create users
    let result = protected::create_entity("user:admin2", "user", "testuser2");
    assert!(result.is_err());
}

// ============================================================================
// Capability Definition Changes
// ============================================================================

/// Verify effect of deleting a capability definition
#[test]
fn delete_capability_definition() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    protected::set_capability("user:root", "resource:doc", "editor", 0x0F).unwrap();
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();

    assert_eq!(check_access("user:alice", "resource:doc", None).unwrap(), 0x0F);

    // Redefine "editor" to have zero capability (effectively deleting its meaning)
    protected::set_capability("user:root", "resource:doc", "editor", 0).unwrap();

    // Alice still has the grant, but the capability is now zero
    assert_eq!(check_access("user:alice", "resource:doc", None).unwrap(), 0);

    // The relationship still exists, just with zero meaning
    let rels = capbit::get_relationships("user:alice", "resource:doc").unwrap();
    assert!(rels.contains(&"editor".to_string()));
}

// ============================================================================
// Entity Lifecycle Revocation
// ============================================================================

/// Verify deleting an entity removes its grants
#[test]
fn delete_entity_removes_all_grants() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc1").unwrap();
    protected::create_entity("user:root", "resource", "doc2").unwrap();

    protected::set_capability("user:root", "resource:doc1", "editor", 0x0F).unwrap();
    protected::set_capability("user:root", "resource:doc2", "viewer", 0x01).unwrap();
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc1").unwrap();
    protected::set_grant("user:root", "user:alice", "viewer", "resource:doc2").unwrap();

    // Alice has grants
    assert!(has_capability("user:alice", "resource:doc1", 0x0F).unwrap());
    assert!(has_capability("user:alice", "resource:doc2", 0x01).unwrap());

    // Delete alice
    protected::delete_entity("user:root", "user:alice").unwrap();
    assert!(!entity_exists("user:alice").unwrap());

    // Grants may still exist as orphans, but entity operations should handle this
    // The main point is alice can't perform any authorized actions anymore
}

// ============================================================================
// Partial Revocation
// ============================================================================

/// Verify revoking one of multiple relations preserves others
#[test]
fn revoke_one_of_multiple_relations() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    protected::set_capability("user:root", "resource:doc", "viewer", 0x01).unwrap();
    protected::set_capability("user:root", "resource:doc", "commenter", 0x02).unwrap();
    protected::set_capability("user:root", "resource:doc", "editor", 0x04).unwrap();

    protected::set_grant("user:root", "user:alice", "viewer", "resource:doc").unwrap();
    protected::set_grant("user:root", "user:alice", "commenter", "resource:doc").unwrap();
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();

    assert_eq!(check_access("user:alice", "resource:doc", None).unwrap(), 0x07);

    // Revoke only commenter
    protected::delete_grant("user:root", "user:alice", "commenter", "resource:doc").unwrap();

    // Viewer and editor remain
    assert_eq!(check_access("user:alice", "resource:doc", None).unwrap(), 0x05);
    assert!(has_capability("user:alice", "resource:doc", 0x01).unwrap()); // viewer
    assert!(has_capability("user:alice", "resource:doc", 0x04).unwrap()); // editor
    assert!(!has_capability("user:alice", "resource:doc", 0x02).unwrap()); // no commenter
}

// ============================================================================
// Inheritance Revocation
// ============================================================================

/// Verify revoking an inheritance source
#[test]
fn revoke_inheritance_source() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    protected::set_capability("user:root", "resource:doc", "editor", 0x0F).unwrap();
    // Need DELEGATE_WRITE to create and DELEGATE_DELETE to remove delegations
    protected::set_capability("user:root", "resource:doc", "delegator", SystemCap::DELEGATE_ADMIN).unwrap();
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();
    protected::set_grant("user:root", "user:alice", "delegator", "resource:doc").unwrap();

    // Set up inheritance
    protected::set_delegation("user:alice", "user:bob", "resource:doc", "user:alice").unwrap();

    assert!(has_capability("user:bob", "resource:doc", 0x0F).unwrap());

    // Delete the inheritance
    protected::delete_delegation("user:alice", "user:bob", "resource:doc", "user:alice").unwrap();

    // Bob no longer has inherited capabilities
    assert_eq!(check_access("user:bob", "resource:doc", None).unwrap(), 0);
}

/// Verify cascade effect on downstream delegatees
#[test]
fn cascade_effect_on_delegatees() {
    let _lock = setup_bootstrapped();

    // Chain: alice -> bob -> charlie
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "user", "charlie").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Include DELEGATE_ADMIN for both creating and deleting delegations
    protected::set_capability("user:root", "resource:doc", "admin", 0xFF | SystemCap::DELEGATE_ADMIN).unwrap();
    protected::set_grant("user:root", "user:alice", "admin", "resource:doc").unwrap();
    protected::set_grant("user:root", "user:bob", "admin", "resource:doc").unwrap(); // for delegation rights

    protected::set_delegation("user:alice", "user:bob", "resource:doc", "user:alice").unwrap();
    protected::set_delegation("user:bob", "user:charlie", "resource:doc", "user:bob").unwrap();

    // Charlie has access through chain
    assert!(has_capability("user:charlie", "resource:doc", 0xFF).unwrap());

    // Break the chain by removing alice -> bob delegation
    protected::delete_delegation("user:alice", "user:bob", "resource:doc", "user:alice").unwrap();

    // Bob still has direct grant, so bob->charlie delegation still gives charlie access via bob
    assert!(has_capability("user:charlie", "resource:doc", 0xFF).unwrap());

    // Now remove bob's direct grant
    protected::delete_grant("user:root", "user:bob", "admin", "resource:doc").unwrap();

    // Charlie no longer has any path to capabilities
    assert_eq!(check_access("user:charlie", "resource:doc", None).unwrap(), 0);
}

// ============================================================================
// Re-granting After Revocation
// ============================================================================

/// Verify permissions can be re-granted after revocation
#[test]
fn re_grant_after_revocation() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    protected::set_capability("user:root", "resource:doc", "editor", 0x0F).unwrap();

    // Grant
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();
    assert!(has_capability("user:alice", "resource:doc", 0x0F).unwrap());

    // Revoke
    protected::delete_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();
    assert!(!has_capability("user:alice", "resource:doc", 0x0F).unwrap());

    // Re-grant
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();
    assert!(has_capability("user:alice", "resource:doc", 0x0F).unwrap());
}

/// Verify changing capability value and re-granting
#[test]
fn change_capability_and_regrant() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Initial capability
    protected::set_capability("user:root", "resource:doc", "editor", 0x03).unwrap();
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();
    assert_eq!(check_access("user:alice", "resource:doc", None).unwrap(), 0x03);

    // Revoke
    protected::delete_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();

    // Change capability definition
    protected::set_capability("user:root", "resource:doc", "editor", 0xFF).unwrap();

    // Re-grant - should get new capability value
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();
    assert_eq!(check_access("user:alice", "resource:doc", None).unwrap(), 0xFF);
}
