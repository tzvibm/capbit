//! Extended attack vector tests for Capbit v2
//!
//! These tests cover advanced security scenarios including confused deputy attacks,
//! privilege accumulation, type confusion, and resource exhaustion attempts.

use capbit::{
    init, bootstrap, is_bootstrapped, protected, check_access,
    set_inheritance, SystemCap, clear_all, test_lock, entity_exists,
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
// Confused Deputy Attacks
// ============================================================================

/// ATTACK: Confused deputy - use a privileged intermediary to perform unauthorized actions
/// Scenario: Alice has delegation rights, Bob tricks Alice into delegating to him
#[test]
fn attack_confused_deputy_via_delegation() {
    let _lock = setup_bootstrapped();

    // Setup: alice is a team admin, bob is nobody
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "resource", "secret_doc").unwrap();

    // Alice has full access to secret_doc including delegation
    protected::set_capability("user:root", "resource:secret_doc", "admin",
        SystemCap::GRANT_ADMIN | SystemCap::DELEGATE_WRITE | 0xFF).unwrap();
    protected::set_grant("user:root", "user:alice", "admin", "resource:secret_doc").unwrap();

    // Verify alice can access
    let alice_caps = check_access("user:alice", "resource:secret_doc", None).unwrap();
    assert!(alice_caps & 0xFF == 0xFF);

    // Bob cannot directly access
    let bob_caps = check_access("user:bob", "resource:secret_doc", None).unwrap();
    assert_eq!(bob_caps, 0);

    // Attack: If bob could somehow call set_delegation as alice, he'd get access
    // The protected API requires the ACTOR to have DELEGATE_WRITE, not just be named
    // Bob tries to delegate to himself claiming to be alice
    let result = protected::set_delegation("user:bob", "user:bob", "resource:secret_doc", "user:alice");

    // DENIED - bob doesn't have DELEGATE_WRITE on resource:secret_doc
    assert!(result.is_err());

    // Bob still has no access
    let bob_caps_after = check_access("user:bob", "resource:secret_doc", None).unwrap();
    assert_eq!(bob_caps_after, 0);
}

// ============================================================================
// Capability Bit Manipulation
// ============================================================================

/// ATTACK: Capability bit overflow - attempt to overflow u64 capability mask
#[test]
fn attack_capability_bit_overflow() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "resource", "target").unwrap();

    // Set max u64 capability
    protected::set_capability("user:root", "resource:target", "superadmin", u64::MAX).unwrap();
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::set_grant("user:root", "user:alice", "superadmin", "resource:target").unwrap();

    // Check that max capability is correctly stored and retrieved
    let caps = check_access("user:alice", "resource:target", None).unwrap();
    assert_eq!(caps, u64::MAX);

    // Verify that checking with max required works
    assert!((caps & u64::MAX) == u64::MAX);
}

/// ATTACK: Use zero capability to bypass checks
#[test]
fn attack_zero_capability_bypass() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Alice has no grants - she has zero capabilities
    let caps = check_access("user:alice", "resource:doc", None).unwrap();
    assert_eq!(caps, 0);

    // Zero capability check always passes (required=0 means "no requirements")
    assert!((caps & 0) == 0);

    // But any non-zero requirement fails
    assert!((caps & 0x01) != 0x01);
}

// ============================================================================
// Scope & Path Manipulation
// ============================================================================

/// ATTACK: Attempt path traversal in scope names
#[test]
fn attack_scope_path_traversal_attempt() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "public").unwrap();
    protected::create_entity("user:root", "resource", "secret").unwrap();

    // Give alice access to public only
    protected::set_capability("user:root", "resource:public", "viewer", 0x01).unwrap();
    protected::set_grant("user:root", "user:alice", "viewer", "resource:public").unwrap();

    // Alice has access to public
    assert_eq!(check_access("user:alice", "resource:public", None).unwrap(), 0x01);

    // Alice does NOT have access to secret
    assert_eq!(check_access("user:alice", "resource:secret", None).unwrap(), 0);

    // Path traversal attempts should not grant access
    // These are different entity IDs, not path-related
    // In the normalized schema, non-existent entities return an error
    assert!(check_access("user:alice", "resource:public/../secret", None).is_err());
    assert!(check_access("user:alice", "resource:./secret", None).is_err());
}

/// ATTACK: Grant on type entity vs instance entity confusion
#[test]
fn attack_grant_on_type_vs_instance_confusion() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "team", "sales").unwrap();

    // Give alice admin on _type:team (can manage all teams)
    protected::set_grant("user:root", "user:alice", "admin", "_type:team").unwrap();

    // Give bob admin on team:sales only (specific instance)
    protected::set_capability("user:root", "team:sales", "admin", SystemCap::GRANT_ADMIN).unwrap();
    protected::set_grant("user:root", "user:bob", "admin", "team:sales").unwrap();

    // Alice can create teams (via _type:team)
    let result = protected::create_entity("user:alice", "team", "engineering");
    assert!(result.is_ok());

    // Bob cannot create teams
    let result = protected::create_entity("user:bob", "team", "marketing");
    assert!(result.is_err());

    // Bob can only grant on team:sales
    protected::create_entity("user:root", "user", "charlie").unwrap();
    let result = protected::set_grant("user:bob", "user:charlie", "member", "team:sales");
    assert!(result.is_ok());

    // Bob cannot grant on team:engineering (alice created it, bob has no rights)
    let result = protected::set_grant("user:bob", "user:charlie", "member", "team:engineering");
    assert!(result.is_err());
}

// ============================================================================
// Privilege Accumulation
// ============================================================================

/// ATTACK: Accumulate privileges through multiple roles
#[test]
fn attack_privilege_accumulation() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Define multiple roles with different bits
    protected::set_capability("user:root", "resource:doc", "viewer", 0x01).unwrap();
    protected::set_capability("user:root", "resource:doc", "commenter", 0x02).unwrap();
    protected::set_capability("user:root", "resource:doc", "editor", 0x04).unwrap();

    // Alice gets viewer only
    protected::set_grant("user:root", "user:alice", "viewer", "resource:doc").unwrap();
    assert_eq!(check_access("user:alice", "resource:doc", None).unwrap(), 0x01);

    // Alice gets commenter too - caps accumulate via OR
    protected::set_grant("user:root", "user:alice", "commenter", "resource:doc").unwrap();
    assert_eq!(check_access("user:alice", "resource:doc", None).unwrap(), 0x03);

    // This is expected behavior - roles combine via OR
    // The "attack" would be if someone could grant themselves roles
    // Alice tries to self-grant editor
    let result = protected::set_grant("user:alice", "user:alice", "editor", "resource:doc");
    assert!(result.is_err()); // Alice lacks GRANT_WRITE on resource:doc
}

/// ATTACK: Indirect privilege escalation through delegation chain
#[test]
fn attack_indirect_escalation_via_chain() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "user", "charlie").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Alice has limited access
    protected::set_capability("user:root", "resource:doc", "viewer", 0x01).unwrap();
    protected::set_capability("user:root", "resource:doc", "delegator", SystemCap::DELEGATE_WRITE).unwrap();
    protected::set_grant("user:root", "user:alice", "viewer", "resource:doc").unwrap();
    protected::set_grant("user:root", "user:alice", "delegator", "resource:doc").unwrap();

    // Bob has more access
    protected::set_capability("user:root", "resource:doc", "editor", 0x0F).unwrap();
    protected::set_grant("user:root", "user:bob", "editor", "resource:doc").unwrap();
    protected::set_grant("user:root", "user:bob", "delegator", "resource:doc").unwrap();

    // Alice delegates to charlie
    protected::set_delegation("user:alice", "user:charlie", "resource:doc", "user:alice").unwrap();

    // Charlie's caps should be bounded by Alice's, not Bob's
    let charlie_caps = check_access("user:charlie", "resource:doc", None).unwrap();
    // Charlie gets viewer (0x01) + delegator (DELEGATE_WRITE) from alice
    assert_eq!(charlie_caps & 0x0F, 0x01); // Only viewer bit, not editor bits

    // Even if bob also delegates to charlie, caps are bounded per source
    protected::set_delegation("user:bob", "user:charlie", "resource:doc", "user:bob").unwrap();
    let charlie_caps_after = check_access("user:charlie", "resource:doc", None).unwrap();
    // Now charlie has alice's caps OR bob's caps
    assert_eq!(charlie_caps_after & 0x0F, 0x0F); // All editor bits via bob
}

// ============================================================================
// Identity Confusion
// ============================================================================

/// ATTACK: Impersonate by using same ID with different type
#[test]
fn attack_impersonate_via_same_id_different_type() {
    let _lock = setup_bootstrapped();

    // Create user:admin with high privileges
    protected::create_entity("user:root", "user", "admin").unwrap();
    protected::set_grant("user:root", "user:admin", "admin", "_type:user").unwrap();

    // Create app:admin - same ID, different type
    protected::create_entity("user:root", "app", "admin").unwrap();

    // Verify they are completely separate entities
    let user_admin_caps = check_access("user:admin", "_type:user", None).unwrap();
    let app_admin_caps = check_access("app:admin", "_type:user", None).unwrap();

    // user:admin has privileges
    assert!((user_admin_caps & SystemCap::ENTITY_ADMIN) == SystemCap::ENTITY_ADMIN);

    // app:admin does NOT have user:admin's privileges
    assert_eq!(app_admin_caps, 0);
}

// ============================================================================
// Resource Exhaustion
// ============================================================================

/// ATTACK: Mass delegation to exhaust traversal
#[test]
fn attack_mass_delegation_resource_exhaustion() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "resource", "target").unwrap();
    protected::set_capability("user:root", "resource:target", "member", 0x01).unwrap();

    // Create a chain of 50 users with delegations (using v1 API for speed)
    for i in 0..50 {
        protected::create_entity("user:root", "user", &format!("u{}", i)).unwrap();
    }

    // u0 has direct access
    protected::set_grant("user:root", "user:u0", "member", "resource:target").unwrap();

    // Create long delegation chain: u0 -> u1 -> u2 -> ... -> u49
    for i in 0..49 {
        set_inheritance(
            &format!("user:u{}", i + 1),
            "resource:target",
            &format!("user:u{}", i),
        ).unwrap();
    }

    // Query should complete (bounded by max_depth)
    let caps = check_access("user:u49", "resource:target", None).unwrap();
    assert_eq!(caps, 0x01); // Should still get the capability through the chain

    // With explicit low depth limit, might not traverse full chain
    let caps_limited = check_access("user:u49", "resource:target", Some(10)).unwrap();
    // May or may not have caps depending on traversal order
    // The key is that it doesn't hang
    let _ = caps_limited;
}

// ============================================================================
// Type System Attacks
// ============================================================================

/// ATTACK: Type confusion - create entities with type-like names
#[test]
fn attack_type_confusion() {
    let _lock = setup_bootstrapped();

    // Attempt to create entity with underscore prefix (reserved for types)
    // The type "_user" doesn't exist, so this should fail
    let result = protected::create_entity("user:root", "_user", "sneaky");

    // This fails because _user type doesn't exist (and can't be created easily)
    assert!(result.is_err());
}

/// ATTACK: Hijack capability definition
#[test]
fn attack_capability_definition_hijack() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "team", "sales").unwrap();

    // Root defines "member" with low capability
    protected::set_capability("user:root", "team:sales", "member", 0x01).unwrap();
    protected::set_grant("user:root", "user:alice", "member", "team:sales").unwrap();

    // Verify alice has low caps
    assert_eq!(check_access("user:alice", "team:sales", None).unwrap(), 0x01);

    // Bob (no CAP_WRITE) tries to redefine "member" with high capability
    let result = protected::set_capability("user:bob", "team:sales", "member", 0xFF);
    assert!(result.is_err()); // Bob lacks CAP_WRITE

    // Alice still has original capability
    assert_eq!(check_access("user:alice", "team:sales", None).unwrap(), 0x01);
}

// ============================================================================
// Zombie Permissions
// ============================================================================

/// ATTACK: Use permissions after entity deletion
#[test]
fn attack_zombie_permission_after_delete() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Give alice access
    protected::set_capability("user:root", "resource:doc", "editor", 0x0F).unwrap();
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();

    assert_eq!(check_access("user:alice", "resource:doc", None).unwrap(), 0x0F);

    // Delete alice
    protected::delete_entity("user:root", "user:alice").unwrap();
    assert!(!entity_exists("user:alice").unwrap());

    // Alice's grants may still exist in relationships db (depending on cleanup)
    // But the entity doesn't exist - operations should handle this gracefully
    // This is a data consistency concern, not necessarily a security one
    // if we check entity existence before operations

    // Check access for deleted entity - should return error (entity not found)
    // In normalized schema, deleted entities can't be looked up
    assert!(check_access("user:alice", "resource:doc", None).is_err());
}

// ============================================================================
// Cross-Type Permission Leak
// ============================================================================

/// ATTACK: Permissions on one type should not affect another type
#[test]
fn attack_cross_type_permission_leak() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "team", "sales").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Alice is admin on team:sales
    protected::set_capability("user:root", "team:sales", "admin", SystemCap::GRANT_ADMIN).unwrap();
    protected::set_grant("user:root", "user:alice", "admin", "team:sales").unwrap();

    // Alice should NOT have permissions on resource:doc
    let result = protected::set_grant("user:alice", "user:alice", "admin", "resource:doc");
    assert!(result.is_err());

    // Alice should NOT be able to create resources
    let result = protected::create_entity("user:alice", "resource", "alice_doc");
    assert!(result.is_err());

    // Verify alice has no caps on resource:doc
    assert_eq!(check_access("user:alice", "resource:doc", None).unwrap(), 0);
}

/// ATTACK: Double bootstrap attempt (simulating race condition)
#[test]
fn attack_bootstrap_race_condition() {
    let _lock = setup_bootstrapped();

    // System is already bootstrapped
    assert!(is_bootstrapped().unwrap());

    // Multiple bootstrap attempts should all fail
    for i in 0..5 {
        let attacker = format!("attacker{}", i);
        let result = bootstrap(&attacker);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("already bootstrapped"));
    }

    // Original root is still the only root
    let root = capbit::get_root_entity().unwrap();
    assert_eq!(root, Some("user:root".to_string()));
}

/// ATTACK: Attempt to inject into meta keys
#[test]
fn attack_meta_key_injection() {
    let _lock = setup_bootstrapped();

    // Meta keys are used internally for bootstrapped flag, root entity, etc.
    // Users shouldn't be able to manipulate these through normal API

    // Try to create entity with meta-like name
    // The entity system uses entities/ db, not meta/ db, so this is safe
    protected::create_entity("user:root", "user", "bootstrapped").unwrap();
    protected::create_entity("user:root", "user", "root_entity").unwrap();

    // System state should be unchanged
    assert!(is_bootstrapped().unwrap());
    assert_eq!(capbit::get_root_entity().unwrap(), Some("user:root".to_string()));

    // These are just regular user entities, not meta entries
    assert!(entity_exists("user:bootstrapped").unwrap());
    assert!(entity_exists("user:root_entity").unwrap());
}
