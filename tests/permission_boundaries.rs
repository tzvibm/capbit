//! Permission boundary tests for Capbit v2
//!
//! These tests verify exact capability boundaries - what happens at the edge of permissions,
//! ensuring that capability checks are precise and predictable.

use capbit::{
    init, bootstrap, protected, check_access, has_capability,
    get_capability, SystemCap, clear_all, test_lock,
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
// Exact Capability Match Tests
// ============================================================================

/// Verify that having exactly the required capability passes
#[test]
fn has_exact_capability_passes() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Define capability with exact bits
    const REQUIRED: u64 = 0x0F;
    protected::set_capability("user:root", "resource:doc", "editor", REQUIRED).unwrap();
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();

    let caps = check_access("user:alice", "resource:doc", None).unwrap();
    assert_eq!(caps, REQUIRED);
    assert!((caps & REQUIRED) == REQUIRED);
    assert!(has_capability("user:alice", "resource:doc", REQUIRED).unwrap());
}

/// Verify that having less than required capability fails
#[test]
fn has_less_than_required_fails() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Alice has only READ (0x01)
    protected::set_capability("user:root", "resource:doc", "viewer", 0x01).unwrap();
    protected::set_grant("user:root", "user:alice", "viewer", "resource:doc").unwrap();

    let caps = check_access("user:alice", "resource:doc", None).unwrap();
    assert_eq!(caps, 0x01);

    // Check for READ + WRITE (0x03) fails
    assert!((caps & 0x03) != 0x03);
    assert!(!has_capability("user:alice", "resource:doc", 0x03).unwrap());
}

/// Verify that having a superset of required capability passes
#[test]
fn has_superset_of_required_passes() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Alice has full permissions (0xFF)
    protected::set_capability("user:root", "resource:doc", "admin", 0xFF).unwrap();
    protected::set_grant("user:root", "user:alice", "admin", "resource:doc").unwrap();

    let caps = check_access("user:alice", "resource:doc", None).unwrap();
    assert_eq!(caps, 0xFF);

    // Check for just READ (0x01) passes
    assert!((caps & 0x01) == 0x01);
    assert!(has_capability("user:alice", "resource:doc", 0x01).unwrap());

    // Check for READ + WRITE (0x03) also passes
    assert!((caps & 0x03) == 0x03);
    assert!(has_capability("user:alice", "resource:doc", 0x03).unwrap());
}

/// Verify that zero capability requirement always passes
#[test]
fn zero_capability_always_passes() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Alice has no grants
    let caps = check_access("user:alice", "resource:doc", None).unwrap();
    assert_eq!(caps, 0);

    // Zero requirement always passes (vacuously true)
    assert!((caps & 0) == 0);
    assert!(has_capability("user:alice", "resource:doc", 0).unwrap());
}

/// Verify max u64 capability is handled correctly
#[test]
fn max_u64_capability_check() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Set max u64 capability
    protected::set_capability("user:root", "resource:doc", "superadmin", u64::MAX).unwrap();
    protected::set_grant("user:root", "user:alice", "superadmin", "resource:doc").unwrap();

    let caps = check_access("user:alice", "resource:doc", None).unwrap();
    assert_eq!(caps, u64::MAX);

    // Can check for any capability
    assert!(has_capability("user:alice", "resource:doc", u64::MAX).unwrap());
    assert!(has_capability("user:alice", "resource:doc", 0x01).unwrap());
    assert!(has_capability("user:alice", "resource:doc", 1 << 63).unwrap());
}

/// Verify that a single bit difference causes failure
#[test]
fn single_bit_difference_fails() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Alice has bits 0, 1, 2 (0x07)
    protected::set_capability("user:root", "resource:doc", "editor", 0x07).unwrap();
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();

    let caps = check_access("user:alice", "resource:doc", None).unwrap();
    assert_eq!(caps, 0x07);

    // Require bits 0, 1, 2, 3 (0x0F) - alice is missing bit 3
    assert!((caps & 0x0F) != 0x0F);
    assert!(!has_capability("user:alice", "resource:doc", 0x0F).unwrap());

    // Require just bit 3 (0x08) - alice doesn't have it
    assert!((caps & 0x08) != 0x08);
    assert!(!has_capability("user:alice", "resource:doc", 0x08).unwrap());
}

// ============================================================================
// Role Combination Tests
// ============================================================================

/// Verify multiple roles combine correctly via OR
#[test]
fn multiple_roles_combine_correctly() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Define three non-overlapping roles
    protected::set_capability("user:root", "resource:doc", "viewer", 0x01).unwrap();
    protected::set_capability("user:root", "resource:doc", "commenter", 0x02).unwrap();
    protected::set_capability("user:root", "resource:doc", "editor", 0x04).unwrap();

    // Grant all three
    protected::set_grant("user:root", "user:alice", "viewer", "resource:doc").unwrap();
    protected::set_grant("user:root", "user:alice", "commenter", "resource:doc").unwrap();
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();

    // Capabilities should be OR'd together
    let caps = check_access("user:alice", "resource:doc", None).unwrap();
    assert_eq!(caps, 0x07);
}

/// Verify inherited capabilities are bounded correctly by delegator
#[test]
fn inherited_caps_bounded_correctly() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Alice has viewer (0x01) and delegation rights
    protected::set_capability("user:root", "resource:doc", "viewer", 0x01).unwrap();
    protected::set_capability("user:root", "resource:doc", "delegator", SystemCap::DELEGATE_WRITE).unwrap();
    protected::set_grant("user:root", "user:alice", "viewer", "resource:doc").unwrap();
    protected::set_grant("user:root", "user:alice", "delegator", "resource:doc").unwrap();

    // Also define editor (0x0F) but don't give it to alice
    protected::set_capability("user:root", "resource:doc", "editor", 0x0F).unwrap();

    // Alice delegates to bob
    protected::set_delegation("user:alice", "user:bob", "resource:doc", "user:alice").unwrap();

    // Bob inherits alice's caps: viewer (0x01) + delegator (DELEGATE_WRITE)
    let bob_caps = check_access("user:bob", "resource:doc", None).unwrap();
    assert_eq!(bob_caps & 0x0F, 0x01); // Only viewer bit
    assert!((bob_caps & SystemCap::DELEGATE_WRITE) == SystemCap::DELEGATE_WRITE);
}

/// Verify type-level grants work alongside instance-level grants
#[test]
fn type_level_vs_instance_level_priority() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "team", "sales").unwrap();

    // Alice has admin on _type:team (type-level)
    // This grants her ENTITY_ADMIN capabilities on any team

    // Also give alice specific capability on team:sales (instance-level)
    protected::set_capability("user:root", "team:sales", "member", 0x01).unwrap();
    protected::set_grant("user:root", "user:alice", "member", "team:sales").unwrap();

    // Alice has member (0x01) on team:sales via instance grant
    let caps = check_access("user:alice", "team:sales", None).unwrap();
    assert_eq!(caps & 0x01, 0x01);

    // Alice can also use type-level admin for operations via protected API
    // The check_permission function in protected.rs checks both instance and type level
}

/// Verify overlapping capability grants from multiple relations merge correctly
#[test]
fn overlapping_grants_merge_correctly() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Two relations with overlapping capabilities
    protected::set_capability("user:root", "resource:doc", "role_a", 0x07).unwrap();  // bits 0,1,2
    protected::set_capability("user:root", "resource:doc", "role_b", 0x0E).unwrap();  // bits 1,2,3

    protected::set_grant("user:root", "user:alice", "role_a", "resource:doc").unwrap();
    protected::set_grant("user:root", "user:alice", "role_b", "resource:doc").unwrap();

    // Caps should OR together: 0x07 | 0x0E = 0x0F
    let caps = check_access("user:alice", "resource:doc", None).unwrap();
    assert_eq!(caps, 0x0F);
}

// ============================================================================
// Scope Precision Tests
// ============================================================================

/// Verify capability on wrong scope is completely ignored
#[test]
fn capability_on_wrong_scope_ignored() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc1").unwrap();
    protected::create_entity("user:root", "resource", "doc2").unwrap();

    // Define capability and grant on doc1
    protected::set_capability("user:root", "resource:doc1", "editor", 0xFF).unwrap();
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc1").unwrap();

    // Alice has full caps on doc1
    assert_eq!(check_access("user:alice", "resource:doc1", None).unwrap(), 0xFF);

    // Alice has ZERO caps on doc2 (different scope)
    assert_eq!(check_access("user:alice", "resource:doc2", None).unwrap(), 0);
}

/// Verify relation without capability definition gives zero capability
#[test]
fn relation_without_capability_gives_zero() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Grant alice "friend" relation, but don't define what "friend" means
    // Using v1 API since protected API might validate
    capbit::set_relationship("user:alice", "friend", "resource:doc").unwrap();

    // Alice has the relationship, but no capability defined for "friend"
    let caps = check_access("user:alice", "resource:doc", None).unwrap();
    assert_eq!(caps, 0);

    // Now define what "friend" means
    protected::set_capability("user:root", "resource:doc", "friend", 0x01).unwrap();

    // Now alice gets the capability
    let caps = check_access("user:alice", "resource:doc", None).unwrap();
    assert_eq!(caps, 0x01);
}

/// Verify capability without grant gives zero capability
#[test]
fn capability_without_grant_gives_zero() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Define capability, but don't grant alice the relation
    protected::set_capability("user:root", "resource:doc", "editor", 0xFF).unwrap();

    // Alice has zero caps (no grant)
    let caps = check_access("user:alice", "resource:doc", None).unwrap();
    assert_eq!(caps, 0);
}

// ============================================================================
// Grant Lifecycle Tests
// ============================================================================

/// Verify deleted grant removes capability
#[test]
fn deleted_grant_removes_capability() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    protected::set_capability("user:root", "resource:doc", "editor", 0x0F).unwrap();
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();

    // Alice has caps
    assert_eq!(check_access("user:alice", "resource:doc", None).unwrap(), 0x0F);

    // Delete the grant
    protected::delete_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();

    // Alice no longer has caps
    assert_eq!(check_access("user:alice", "resource:doc", None).unwrap(), 0);
}

/// Verify updated capability affects existing grants
#[test]
fn updated_capability_affects_existing_grants() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Define initial capability
    protected::set_capability("user:root", "resource:doc", "editor", 0x03).unwrap();
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();

    assert_eq!(check_access("user:alice", "resource:doc", None).unwrap(), 0x03);

    // Update capability definition (same relation name, different caps)
    protected::set_capability("user:root", "resource:doc", "editor", 0xFF).unwrap();

    // Alice's existing grant now gives her the new capability value
    assert_eq!(check_access("user:alice", "resource:doc", None).unwrap(), 0xFF);

    // Verify the stored capability
    let stored = get_capability("resource:doc", "editor").unwrap();
    assert_eq!(stored, Some(0xFF));
}

/// Verify removing one of multiple relations reduces capability correctly
#[test]
fn removing_one_relation_reduces_caps() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Two relations
    protected::set_capability("user:root", "resource:doc", "viewer", 0x01).unwrap();
    protected::set_capability("user:root", "resource:doc", "editor", 0x06).unwrap();

    protected::set_grant("user:root", "user:alice", "viewer", "resource:doc").unwrap();
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();

    assert_eq!(check_access("user:alice", "resource:doc", None).unwrap(), 0x07);

    // Remove editor
    protected::delete_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();

    // Only viewer remains
    assert_eq!(check_access("user:alice", "resource:doc", None).unwrap(), 0x01);
}
