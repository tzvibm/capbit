//! Attack vector tests for Capbit v2
//!
//! These tests verify that the protected API correctly denies unauthorized operations.

use capbit::{
    init, bootstrap, is_bootstrapped, protected, check_access,
    set_inheritance, SystemCap, clear_all, test_lock,
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

/// ATTACK: Entity spoofing - attacker creates entity before legitimate user
#[test]
fn attack_entity_spoofing() {
    let _lock = setup_bootstrapped();

    // Alice has admin on _type:user (root grants this)
    protected::set_grant("user:root", "user:alice", "admin", "_type:user").unwrap();

    // Bob (no permissions) tries to create user:alice (impersonation)
    let result = protected::create_entity("user:bob", "user", "alice");

    // Expected: DENIED - bob lacks ENTITY_CREATE on _type:user
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("lacks permission"));
}

/// ATTACK: Privilege escalation via self-grant
#[test]
fn attack_self_grant_escalation() {
    let _lock = setup_bootstrapped();

    // Create alice and grant her limited permissions
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "team", "sales").unwrap();

    // Give alice GRANT_WRITE on team:sales only
    protected::set_capability("user:root", "team:sales", "admin", SystemCap::GRANT_WRITE).unwrap();
    protected::set_grant("user:root", "user:alice", "admin", "team:sales").unwrap();

    // Attack: alice tries to grant herself admin on _type:user (escalation)
    let result = protected::set_grant("user:alice", "user:alice", "admin", "_type:user");

    // Expected: DENIED - alice lacks GRANT_WRITE on _type:user
    assert!(result.is_err());
}

/// ATTACK: Scope confusion - grant on wrong scope
#[test]
fn attack_scope_confusion() {
    let _lock = setup_bootstrapped();

    // Create alice and two teams
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "team", "sales").unwrap();
    protected::create_entity("user:root", "team", "engineering").unwrap();

    // Alice is admin on team:sales (can grant on sales)
    protected::set_capability("user:root", "team:sales", "admin", SystemCap::GRANT_ADMIN).unwrap();
    protected::set_grant("user:root", "user:alice", "admin", "team:sales").unwrap();

    // Attack: alice grants bob admin on team:engineering (wrong scope)
    let result = protected::set_grant("user:alice", "user:bob", "admin", "team:engineering");

    // Expected: DENIED - alice lacks GRANT_WRITE on team:engineering
    assert!(result.is_err());
}

/// ATTACK: Delegation abuse - inherit more than delegator has
#[test]
fn attack_delegation_amplification() {
    let _lock = setup_bootstrapped();

    // Setup: alice has only READ capability on doc
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    const READ: u64 = 0x01;
    const WRITE: u64 = 0x02;

    protected::set_capability("user:root", "resource:doc", "viewer", READ).unwrap();
    protected::set_grant("user:root", "user:alice", "viewer", "resource:doc").unwrap();

    // Root sets up delegation (alice delegates to bob)
    protected::set_capability("user:root", "resource:doc", "owner", SystemCap::DELEGATE_WRITE).unwrap();
    protected::set_grant("user:root", "user:alice", "owner", "resource:doc").unwrap();
    protected::set_delegation("user:alice", "user:bob", "resource:doc", "user:alice").unwrap();

    // Bob's effective caps should be bounded by alice's (only READ, not WRITE)
    let bob_caps = check_access("user:bob", "resource:doc", None).unwrap();
    assert_eq!(bob_caps & READ, READ);  // bob has READ (inherited from alice)
    assert_eq!(bob_caps & WRITE, 0);     // bob does NOT have WRITE
}

/// ATTACK: Bootstrap replay - re-run bootstrap to become root
#[test]
fn attack_bootstrap_replay() {
    let _lock = setup_bootstrapped();

    // System is already bootstrapped
    assert!(is_bootstrapped().unwrap());

    // Attack: attacker tries to call bootstrap again
    let result = bootstrap("attacker");

    // Expected: ERROR - already bootstrapped
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("already bootstrapped"));
}

/// ATTACK: Circular delegation DoS
#[test]
fn attack_circular_delegation_dos() {
    let _lock = setup_bootstrapped();

    // Create entities
    protected::create_entity("user:root", "user", "a").unwrap();
    protected::create_entity("user:root", "user", "b").unwrap();
    protected::create_entity("user:root", "user", "c").unwrap();
    protected::create_entity("user:root", "resource", "target").unwrap();

    // Give A direct access
    protected::set_capability("user:root", "resource:target", "member", 0x0F).unwrap();
    protected::set_grant("user:root", "user:a", "member", "resource:target").unwrap();

    // Create cycle using v1 API (simulating potential attack):
    // A delegates to B, B delegates to C, C delegates to A
    set_inheritance("user:b", "resource:target", "user:a").unwrap();
    set_inheritance("user:c", "resource:target", "user:b").unwrap();
    set_inheritance("user:a", "resource:target", "user:c").unwrap();  // Creates cycle!

    // Query should not hang - bounded traversal
    let caps = check_access("user:c", "resource:target", None).unwrap();

    // Should still return the correct capability (from A's direct grant)
    assert_eq!(caps, 0x0F);
}

/// ATTACK: Type mutation after bootstrap - delete system type
#[test]
fn attack_mutate_system_types() {
    let _lock = setup_bootstrapped();

    // Create bob with no special permissions
    protected::create_entity("user:root", "user", "bob").unwrap();

    // Attack: bob tries to create a new type (requires TYPE_CREATE on _type:_type)
    let result = protected::create_type("user:bob", "custom");

    // Expected: DENIED - bob lacks TYPE_CREATE on _type:_type
    assert!(result.is_err());
}

/// ATTACK: Unauthorized entity deletion
#[test]
fn attack_unauthorized_deletion() {
    let _lock = setup_bootstrapped();

    // Root creates alice and bob
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();

    // Bob tries to delete alice
    let result = protected::delete_entity("user:bob", "user:alice");

    // Expected: DENIED - bob lacks ENTITY_DELETE on _type:user
    assert!(result.is_err());
}

/// ATTACK: Grant to non-existent scope (scope validation)
#[test]
fn attack_grant_nonexistent_scope() {
    let _lock = setup_bootstrapped();

    // Note: root has GRANT_WRITE on all _type: scopes, but not on random scopes
    // Root tries to grant on a scope that doesn't exist
    let result = protected::set_grant("user:root", "user:alice", "member", "team:nonexistent");

    // This might succeed if we allow grants to non-existent scopes,
    // or fail if we validate scope existence. Current impl validates.
    // Adjust test based on desired behavior.
    assert!(result.is_err());
}

/// ATTACK: Fake SystemCap - user defines same bitmask value on their own entity
/// This proves that SystemCap values are only protected when on _type:* scopes
#[test]
fn attack_fake_systemcap_bitmask() {
    let _lock = setup_bootstrapped();

    // Create alice and give her a resource she controls
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "alice-doc").unwrap();

    // Give alice full control on her own resource
    protected::set_capability("user:root", "resource:alice-doc", "owner",
        SystemCap::ENTITY_ADMIN | SystemCap::GRANT_ADMIN | SystemCap::CAP_ADMIN).unwrap();
    protected::set_grant("user:root", "user:alice", "owner", "resource:alice-doc").unwrap();

    // Alice now has ALL SystemCap bits on resource:alice-doc
    let alice_caps_on_doc = check_access("user:alice", "resource:alice-doc", None).unwrap();
    assert!(alice_caps_on_doc > 0); // She has capabilities on HER resource

    // ATTACK: Alice tries to use these "powers" to create a user
    // She has ENTITY_CREATE (0x0004) on her resource, but NOT on _type:user
    let result = protected::create_entity("user:alice", "user", "hacked");

    // Expected: DENIED - having SystemCap bits on resource:alice-doc
    // does NOT grant system powers on _type:user
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("lacks permission"));

    // Verify alice has zero capabilities on _type:user
    let alice_caps_on_type = check_access("user:alice", "_type:user", None).unwrap();
    assert_eq!(alice_caps_on_type, 0, "Alice should have no caps on _type:user");
}

/// Verify root's grants on _type:* are properly protected
#[test]
fn verify_root_grants_protected() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();

    // Verify root has SystemCap on _type:user
    let root_caps = check_access("user:root", "_type:user", None).unwrap();
    assert!((root_caps & SystemCap::ENTITY_CREATE) == SystemCap::ENTITY_CREATE);
    assert!((root_caps & SystemCap::GRANT_WRITE) == SystemCap::GRANT_WRITE);
    assert!((root_caps & SystemCap::CAP_WRITE) == SystemCap::CAP_WRITE);

    // Alice has NOTHING on _type:user
    let alice_caps = check_access("user:alice", "_type:user", None).unwrap();
    assert_eq!(alice_caps, 0);

    // Alice cannot grant herself admin on _type:user
    let result = protected::set_grant("user:alice", "user:alice", "admin", "_type:user");
    assert!(result.is_err());

    // Alice cannot define capabilities on _type:user
    let result = protected::set_capability("user:alice", "_type:user", "fake", 0xFFFF);
    assert!(result.is_err());
}
