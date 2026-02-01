//! Integration tests for Capbit public API

use capbit::*;
use std::sync::Once;

static INIT: Once = Once::new();

fn test_db_path() -> String {
    std::env::var("CAPBIT_TEST_DB").unwrap_or_else(|_| {
        let tmp = std::env::temp_dir();
        tmp.join("capbit_test.mdb").to_string_lossy().to_string()
    })
}

fn setup() -> std::sync::MutexGuard<'static, ()> {
    let lock = test_lock();
    INIT.call_once(|| {
        init(&test_db_path()).unwrap();
    });
    clear_all().unwrap();
    lock
}

// === Core Operations ===

#[test]
fn test_grant_and_check() {
    let _lock = setup();
    grant(1, 100, READ | WRITE).unwrap();

    assert!(check(1, 100, READ).unwrap());
    assert!(check(1, 100, WRITE).unwrap());
    assert!(check(1, 100, READ | WRITE).unwrap());
    assert!(!check(1, 100, DELETE).unwrap());
}

#[test]
fn test_grant_accumulates() {
    let _lock = setup();
    grant(1, 100, READ).unwrap();
    grant(1, 100, WRITE).unwrap();

    assert_eq!(get_mask(1, 100).unwrap(), READ | WRITE);
}

#[test]
fn test_grant_set_replaces() {
    let _lock = setup();
    grant(1, 100, READ | WRITE | DELETE).unwrap();
    grant_set(1, 100, READ).unwrap();

    assert_eq!(get_mask(1, 100).unwrap(), READ);
}

#[test]
fn test_revoke() {
    let _lock = setup();
    grant(1, 100, READ | WRITE).unwrap();
    assert!(revoke(1, 100).unwrap());

    assert!(!check(1, 100, READ).unwrap());
    assert!(!revoke(1, 100).unwrap()); // already revoked
}

// === Roles ===

#[test]
fn test_roles() {
    let _lock = setup();
    const EDITOR: u64 = 1000;

    set_role(100, EDITOR, READ | WRITE).unwrap();
    grant(1, 100, EDITOR).unwrap();

    assert!(check(1, 100, READ).unwrap());
    assert!(check(1, 100, WRITE).unwrap());
    assert!(!check(1, 100, DELETE).unwrap());
}

#[test]
fn test_role_update_affects_checks() {
    let _lock = setup();
    const EDITOR: u64 = 1000;

    set_role(100, EDITOR, READ).unwrap();
    grant(1, 100, EDITOR).unwrap();
    assert!(!check(1, 100, WRITE).unwrap());

    // Update role definition
    set_role(100, EDITOR, READ | WRITE).unwrap();
    assert!(check(1, 100, WRITE).unwrap());
}

// === Inheritance ===

#[test]
fn test_inheritance() {
    let _lock = setup();
    // alice (1) inherits from managers (10)
    set_inherit(100, 1, 10).unwrap();
    grant(10, 100, READ | WRITE).unwrap();

    assert!(check(1, 100, READ).unwrap());
    assert!(check(1, 100, WRITE).unwrap());
}

#[test]
fn test_inheritance_chain() {
    let _lock = setup();
    // alice -> managers -> admins
    set_inherit(100, 1, 10).unwrap();
    set_inherit(100, 10, 20).unwrap();
    grant(20, 100, ADMIN).unwrap();

    assert!(check(1, 100, ADMIN).unwrap());
}

#[test]
fn test_inheritance_cycle_prevention() {
    let _lock = setup();
    set_inherit(100, 1, 2).unwrap();
    set_inherit(100, 2, 3).unwrap();

    // Should fail: 3 -> 1 creates cycle
    assert!(set_inherit(100, 3, 1).is_err());
}

#[test]
fn test_self_inherit_prevention() {
    let _lock = setup();
    assert!(set_inherit(100, 1, 1).is_err());
}

#[test]
fn test_remove_inherit() {
    let _lock = setup();
    set_inherit(100, 1, 10).unwrap();
    grant(10, 100, READ).unwrap();
    assert!(check(1, 100, READ).unwrap());

    remove_inherit(100, 1).unwrap();
    assert!(!check(1, 100, READ).unwrap());
}

// === Batch Operations ===

#[test]
fn test_batch_grant() {
    let _lock = setup();
    batch_grant(&[
        (1, 100, READ),
        (2, 100, WRITE),
        (3, 100, DELETE),
    ]).unwrap();

    assert!(check(1, 100, READ).unwrap());
    assert!(check(2, 100, WRITE).unwrap());
    assert!(check(3, 100, DELETE).unwrap());
}

#[test]
fn test_batch_revoke() {
    let _lock = setup();
    batch_grant(&[(1, 100, READ), (2, 100, WRITE)]).unwrap();

    let count = batch_revoke(&[(1, 100), (2, 100), (3, 100)]).unwrap();
    assert_eq!(count, 2); // only 2 existed
}

#[test]
fn test_transact() {
    let _lock = setup();
    transact(|tx| {
        tx.grant(1, 100, READ)?;
        tx.grant(2, 100, WRITE)?;
        tx.set_role(100, 1000, READ | WRITE | DELETE)?;
        Ok(())
    }).unwrap();

    assert!(check(1, 100, READ).unwrap());
    assert!(check(2, 100, WRITE).unwrap());
    assert_eq!(get_role(100, 1000).unwrap(), READ | WRITE | DELETE);
}

// === Entities ===

#[test]
fn test_create_entity() {
    let _lock = setup();
    let alice = create_entity("alice").unwrap();
    let bob = create_entity("bob").unwrap();

    assert!(alice > 0);
    assert_eq!(bob, alice + 1);
    assert_eq!(get_label(alice).unwrap(), Some("alice".to_string()));
    assert_eq!(get_id_by_label("alice").unwrap(), Some(alice));
}

#[test]
fn test_rename_entity() {
    let _lock = setup();
    let id = create_entity("alice").unwrap();
    rename_entity(id, "alicia").unwrap();

    assert_eq!(get_label(id).unwrap(), Some("alicia".to_string()));
    assert_eq!(get_id_by_label("alicia").unwrap(), Some(id));
    assert_eq!(get_id_by_label("alice").unwrap(), None);
}

#[test]
fn test_delete_entity() {
    let _lock = setup();
    let id = create_entity("alice").unwrap();
    assert!(delete_entity(id).unwrap());

    assert_eq!(get_label(id).unwrap(), None);
    assert_eq!(get_id_by_label("alice").unwrap(), None);
}

#[test]
fn test_set_label() {
    let _lock = setup();
    set_label(42, "answer").unwrap();

    assert_eq!(get_label(42).unwrap(), Some("answer".to_string()));
    assert_eq!(get_id_by_label("answer").unwrap(), Some(42));
}

// === Queries ===

#[test]
fn test_list_for_subject() {
    let _lock = setup();
    grant(1, 100, READ).unwrap();
    grant(1, 101, WRITE).unwrap();
    grant(1, 102, DELETE).unwrap();

    let list = list_for_subject(1).unwrap();
    assert_eq!(list.len(), 3);
}

#[test]
fn test_list_for_object() {
    let _lock = setup();
    grant(1, 100, READ).unwrap();
    grant(2, 100, WRITE).unwrap();
    grant(3, 100, DELETE).unwrap();

    let list = list_for_object(100).unwrap();
    assert_eq!(list.len(), 3);
}

#[test]
fn test_count_functions() {
    let _lock = setup();
    grant(1, 100, READ).unwrap();
    grant(1, 101, READ).unwrap();
    grant(2, 100, READ).unwrap();

    assert_eq!(count_for_subject(1).unwrap(), 2);
    assert_eq!(count_for_object(100).unwrap(), 2);
}

// === Protected Operations (check against _system object) ===

#[test]
fn test_protected_grant_with_system_grant() {
    let _lock = setup();
    let (system, root_user) = bootstrap().unwrap();

    // Root user (with GRANT on _system) can grant any permission
    protected_grant(root_user, 2, 100, READ | WRITE | DELETE).unwrap();
    assert!(check(2, 100, DELETE).unwrap());

    // Non-privileged user cannot grant
    let alice = create_entity("alice").unwrap();
    assert!(protected_grant(alice, 3, 100, READ).is_err());

    // Grant GRANT permission on _system to alice
    grant(alice, system, GRANT).unwrap();
    protected_grant(alice, 3, 100, READ).unwrap();
    assert!(check(3, 100, READ).unwrap());
}

#[test]
fn test_protected_revoke_requires_system_grant() {
    let _lock = setup();
    let (system, root_user) = bootstrap().unwrap();
    grant(2, 100, READ).unwrap();

    // Non-privileged user cannot revoke
    let alice = create_entity("alice").unwrap();
    assert!(protected_revoke(alice, 2, 100).is_err());

    // Root user can revoke
    assert!(protected_revoke(root_user, 2, 100).unwrap());

    // Grant GRANT on _system to alice, then she can revoke
    grant(3, 100, READ).unwrap();
    grant(alice, system, GRANT).unwrap();
    assert!(protected_revoke(alice, 3, 100).unwrap());
}

#[test]
fn test_protected_set_role_requires_system_admin() {
    let _lock = setup();
    let (system, root_user) = bootstrap().unwrap();

    // Non-privileged user cannot set roles
    let alice = create_entity("alice").unwrap();
    assert!(protected_set_role(alice, 100, 1, READ | WRITE).is_err());

    // Root user can
    protected_set_role(root_user, 100, 1, READ | WRITE).unwrap();
    assert_eq!(get_role(100, 1).unwrap(), READ | WRITE);

    // Grant ADMIN on _system to alice
    grant(alice, system, ADMIN).unwrap();
    protected_set_role(alice, 100, 2, DELETE).unwrap();
    assert_eq!(get_role(100, 2).unwrap(), DELETE);
}

#[test]
fn test_protected_inherit_requires_system_admin() {
    let _lock = setup();
    let (system, root_user) = bootstrap().unwrap();

    let alice = create_entity("alice").unwrap();

    // Non-privileged user cannot set inheritance
    assert!(protected_set_inherit(alice, 100, 1, 2).is_err());
    assert!(protected_remove_inherit(alice, 100, 1).is_err());

    // Root user can
    protected_set_inherit(root_user, 100, 1, 2).unwrap();
    assert_eq!(get_inherit(100, 1).unwrap(), Some(2));
    protected_remove_inherit(root_user, 100, 1).unwrap();
    assert_eq!(get_inherit(100, 1).unwrap(), None);

    // Grant ADMIN on _system to alice
    grant(alice, system, ADMIN).unwrap();
    protected_set_inherit(alice, 100, 1, 2).unwrap();
    assert_eq!(get_inherit(100, 1).unwrap(), Some(2));
}

#[test]
fn test_protected_list_requires_system_view() {
    let _lock = setup();
    let (system, root_user) = bootstrap().unwrap();
    grant(1, 100, READ).unwrap();

    let alice = create_entity("alice").unwrap();

    // Non-privileged user cannot list
    assert!(protected_list_for_object(alice, 100).is_err());

    // Root user can
    assert_eq!(protected_list_for_object(root_user, 100).unwrap().len(), 1);

    // Grant VIEW on _system to alice
    grant(alice, system, VIEW).unwrap();
    assert_eq!(protected_list_for_object(alice, 100).unwrap().len(), 1);
}

#[test]
fn test_user_freedom_any_bits() {
    let _lock = setup();
    bootstrap().unwrap();

    // Users can use ANY bit on their own objects - system doesn't care
    const MY_ADMIN: u64 = 1 << 63;  // Same bit as ADMIN
    const MY_CUSTOM: u64 = 1 << 50;

    let alice = create_entity("alice").unwrap();
    let my_doc = create_entity("my_doc").unwrap();

    // Alice grants herself custom bits on her doc (no system permission needed)
    grant(alice, my_doc, MY_ADMIN | MY_CUSTOM | READ | WRITE).unwrap();

    assert!(check(alice, my_doc, MY_ADMIN).unwrap());
    assert!(check(alice, my_doc, MY_CUSTOM).unwrap());
    assert!(check(alice, my_doc, READ | WRITE).unwrap());

    // She can grant to others too (unprotected grant)
    let bob = create_entity("bob").unwrap();
    grant(bob, my_doc, READ).unwrap();
    assert!(check(bob, my_doc, READ).unwrap());
}

// === Happy Path: Root delegates to users ===

#[test]
fn test_root_creates_admin_user() {
    let _lock = setup();
    let (system, root_user) = bootstrap().unwrap();

    // Root creates an admin and grants them full system access
    let admin = create_entity("admin").unwrap();
    protected_grant(root_user, admin, system, u64::MAX).unwrap();

    // Admin now has all powers
    assert!(check(admin, system, GRANT | ADMIN | VIEW).unwrap());

    // Admin can do protected operations
    let user = create_entity("user").unwrap();
    protected_grant(admin, user, system, VIEW).unwrap();
    assert!(check(user, system, VIEW).unwrap());
}

#[test]
fn test_root_creates_limited_operator() {
    let _lock = setup();
    let (system, root_user) = bootstrap().unwrap();

    // Root creates operator with only GRANT (can assign permissions but not change roles)
    let operator = create_entity("operator").unwrap();
    protected_grant(root_user, operator, system, GRANT).unwrap();

    // Operator can grant permissions to users
    let user = create_entity("user").unwrap();
    protected_grant(operator, user, system, VIEW).unwrap();
    assert!(check(user, system, VIEW).unwrap());

    // But operator cannot set roles (needs ADMIN)
    assert!(protected_set_role(operator, 100, 1, READ).is_err());
}

#[test]
fn test_delegation_chain() {
    let _lock = setup();
    let (system, root_user) = bootstrap().unwrap();

    // Root -> Admin -> Operator -> User
    let admin = create_entity("admin").unwrap();
    let operator = create_entity("operator").unwrap();
    let user = create_entity("user").unwrap();

    // Root grants admin full access
    protected_grant(root_user, admin, system, GRANT | ADMIN | VIEW).unwrap();

    // Admin grants operator limited access
    protected_grant(admin, operator, system, GRANT | VIEW).unwrap();

    // Operator grants user view-only
    protected_grant(operator, user, system, VIEW).unwrap();

    // Verify permissions
    assert!(check(admin, system, ADMIN).unwrap());
    assert!(!check(operator, system, ADMIN).unwrap());
    assert!(check(operator, system, GRANT).unwrap());
    assert!(!check(user, system, GRANT).unwrap());
    assert!(check(user, system, VIEW).unwrap());
}

// === Adversarial: Privilege escalation attempts ===

#[test]
fn test_adversarial_unprivileged_user() {
    let _lock = setup();
    let (system, _root_user) = bootstrap().unwrap();

    let attacker = create_entity("attacker").unwrap();
    let victim = create_entity("victim").unwrap();

    // Attacker has no system permissions - all protected ops fail
    assert!(protected_grant(attacker, attacker, system, ADMIN).is_err());
    assert!(protected_grant(attacker, victim, 100, READ).is_err());
    assert!(protected_revoke(attacker, victim, 100).is_err());
    assert!(protected_set_role(attacker, 100, 1, READ).is_err());
    assert!(protected_set_inherit(attacker, 100, 1, 2).is_err());
    assert!(protected_list_for_object(attacker, 100).is_err());
}

#[test]
fn test_adversarial_self_escalation() {
    let _lock = setup();
    let (system, root_user) = bootstrap().unwrap();

    // User with VIEW tries to escalate to GRANT
    let user = create_entity("user").unwrap();
    protected_grant(root_user, user, system, VIEW).unwrap();

    // Cannot grant themselves more permissions
    assert!(protected_grant(user, user, system, GRANT).is_err());
    assert!(protected_grant(user, user, system, ADMIN).is_err());

    // Still only has VIEW
    assert!(check(user, system, VIEW).unwrap());
    assert!(!check(user, system, GRANT).unwrap());
}

#[test]
fn test_adversarial_grant_without_permission() {
    let _lock = setup();
    let (system, root_user) = bootstrap().unwrap();

    // User with GRANT but not ADMIN tries to grant ADMIN
    let user = create_entity("user").unwrap();
    let target = create_entity("target").unwrap();
    protected_grant(root_user, user, system, GRANT).unwrap();

    // Can grant GRANT (has it)
    protected_grant(user, target, system, GRANT).unwrap();
    assert!(check(target, system, GRANT).unwrap());

    // The system doesn't restrict what bits you grant - it only checks if you have GRANT permission
    // This is by design: GRANT on _system = can call protected_grant
    protected_grant(user, target, system, ADMIN).unwrap();
    assert!(check(target, system, ADMIN).unwrap());
}

#[test]
fn test_adversarial_revoke_root() {
    let _lock = setup();
    let (system, root_user) = bootstrap().unwrap();

    // User with GRANT tries to revoke root's permissions
    let attacker = create_entity("attacker").unwrap();
    protected_grant(root_user, attacker, system, GRANT).unwrap();

    // Attacker can revoke root (has GRANT permission)
    // This is intentional - if you grant someone GRANT, they can revoke anyone
    assert!(protected_revoke(attacker, root_user, system).unwrap());

    // Root no longer has permissions on _system
    assert!(!check(root_user, system, GRANT).unwrap());

    // But attacker still does
    assert!(check(attacker, system, GRANT).unwrap());
}

#[test]
fn test_adversarial_direct_grant_bypass() {
    let _lock = setup();
    let (system, _root_user) = bootstrap().unwrap();

    // Attacker tries to use unprotected grant() to give themselves system permissions
    let attacker = create_entity("attacker").unwrap();

    // This works! grant() is unprotected
    grant(attacker, system, ADMIN).unwrap();
    assert!(check(attacker, system, ADMIN).unwrap());

    // This is why you must not expose grant() to untrusted code
    // Only expose protected_* functions to users
}

#[test]
fn test_view_only_cannot_modify() {
    let _lock = setup();
    let (system, root_user) = bootstrap().unwrap();

    let viewer = create_entity("viewer").unwrap();
    protected_grant(root_user, viewer, system, VIEW).unwrap();

    // Viewer can list
    grant(1, 100, READ).unwrap();
    assert!(protected_list_for_object(viewer, 100).is_ok());

    // But cannot modify anything
    assert!(protected_grant(viewer, 2, 100, READ).is_err());
    assert!(protected_revoke(viewer, 1, 100).is_err());
    assert!(protected_set_role(viewer, 100, 1, READ).is_err());
}

#[test]
fn test_separate_permission_bits() {
    let _lock = setup();
    let (system, root_user) = bootstrap().unwrap();

    // Create users with different individual permissions
    let granter = create_entity("granter").unwrap();
    let admin = create_entity("admin").unwrap();
    let viewer = create_entity("viewer").unwrap();

    protected_grant(root_user, granter, system, GRANT).unwrap();
    protected_grant(root_user, admin, system, ADMIN).unwrap();
    protected_grant(root_user, viewer, system, VIEW).unwrap();

    // Granter: can grant/revoke, cannot set roles
    protected_grant(granter, 1, 100, READ).unwrap();
    protected_revoke(granter, 1, 100).unwrap();
    assert!(protected_set_role(granter, 100, 1, READ).is_err());

    // Admin: can set roles/inherit, cannot grant (no GRANT bit)
    protected_set_role(admin, 100, 1, READ).unwrap();
    protected_set_inherit(admin, 100, 1, 2).unwrap();
    assert!(protected_grant(admin, 1, 100, READ).is_err());

    // Viewer: can only list
    grant(1, 100, READ).unwrap();
    protected_list_for_object(viewer, 100).unwrap();
    assert!(protected_grant(viewer, 2, 100, READ).is_err());
    assert!(protected_set_role(viewer, 100, 2, READ).is_err());
}

// === Bootstrap ===

#[test]
fn test_bootstrap() {
    let _lock = setup();
    assert!(!is_bootstrapped().unwrap());

    let (system, root_user) = bootstrap().unwrap();

    assert!(is_bootstrapped().unwrap());
    assert_eq!(get_root_user().unwrap(), Some(root_user));
    assert_eq!(get_system().unwrap(), system);

    // Root user has all permissions on _system
    assert!(check(root_user, system, u64::MAX).unwrap());

    // Entities have labels
    assert_eq!(get_label(system).unwrap(), Some("_system".to_string()));
    assert_eq!(get_label(root_user).unwrap(), Some("_root_user".to_string()));
}

#[test]
fn test_bootstrap_only_once() {
    let _lock = setup();
    bootstrap().unwrap();
    assert!(bootstrap().is_err());
}

// === Utility Functions ===

#[test]
fn test_caps_to_names() {
    let names = caps_to_names(READ | WRITE | ADMIN);
    assert!(names.contains(&"read"));
    assert!(names.contains(&"write"));
    assert!(names.contains(&"admin"));
    assert!(!names.contains(&"delete"));
}

#[test]
fn test_names_to_caps() {
    let mask = names_to_caps(&["read", "write", "admin"]);
    assert_eq!(mask, READ | WRITE | ADMIN);
}

// === Constants ===

#[test]
fn test_constants_are_distinct() {
    let all = [READ, WRITE, DELETE, CREATE, GRANT, EXECUTE, VIEW, ADMIN];
    for i in 0..all.len() {
        for j in (i+1)..all.len() {
            assert_eq!(all[i] & all[j], 0, "Constants should not overlap");
        }
    }
}

#[test]
fn test_constant_values() {
    assert_eq!(READ, 1);
    assert_eq!(WRITE, 2);
    assert_eq!(DELETE, 4);
    assert_eq!(CREATE, 8);
    assert_eq!(GRANT, 16);
    assert_eq!(EXECUTE, 32);
    assert_eq!(VIEW, 1 << 62);
    assert_eq!(ADMIN, 1 << 63);
}

// === Edge Cases ===

#[test]
fn test_edge_case_ids() {
    let _lock = setup();
    let ids: &[u64] = &[0, 1, 2, 100, 1000, u64::MAX / 2, u64::MAX - 1, u64::MAX];
    for &s in ids {
        for &o in ids {
            grant(s, o, READ).unwrap();
            assert!(check(s, o, READ).unwrap(), "s={s} o={o}");
        }
    }
}

#[test]
fn test_check_zero_required() {
    let _lock = setup();
    // 0 required = always true (no permissions needed)
    assert!(check(1, 100, 0).unwrap());
}

#[test]
fn test_get_mask_empty() {
    let _lock = setup();
    assert_eq!(get_mask(999, 999).unwrap(), 0);
}

#[test]
fn test_grant_idempotent() {
    let _lock = setup();
    grant(1, 100, READ | WRITE).unwrap();
    grant(1, 100, READ | WRITE).unwrap();
    assert_eq!(get_mask(1, 100).unwrap(), READ | WRITE);
}

#[test]
fn test_revoke_nonexistent() {
    let _lock = setup();
    assert!(!revoke(999, 999).unwrap());
}

// === Isolation ===

#[test]
fn test_isolation_subjects() {
    let _lock = setup();
    for s in 1..=10 {
        grant(s, 100, s).unwrap();
    }
    for s in 1..=10 {
        assert_eq!(get_mask(s, 100).unwrap(), s);
    }
}

#[test]
fn test_isolation_objects() {
    let _lock = setup();
    for o in 100..=110 {
        grant(1, o, o - 99).unwrap();
    }
    for o in 100..=110 {
        assert_eq!(get_mask(1, o).unwrap(), o - 99);
    }
}

#[test]
fn test_isolation_pairs() {
    let _lock = setup();
    grant(1, 100, READ).unwrap();
    grant(2, 101, WRITE).unwrap();
    assert!(!check(1, 101, READ).unwrap());
    assert!(!check(2, 100, WRITE).unwrap());
}

// === Table-Driven Cap Combos ===

#[test]
fn test_cap_bit_combinations() {
    let _lock = setup();
    let cases: &[(u64, u64, bool)] = &[
        (READ, READ, true),
        (READ, WRITE, false),
        (READ | WRITE, READ, true),
        (READ | WRITE, WRITE, true),
        (READ | WRITE, DELETE, false),
        (READ | WRITE | DELETE, READ | WRITE, true),
        (0xFF, 0x0F, true),
        (0x0F, 0xFF, false),
        (ADMIN, ADMIN, true),
        (ADMIN, READ, false),
        (u64::MAX, u64::MAX, true),
        (u64::MAX, 1, true),
        (1, u64::MAX, false),
        (0, 0, true),
        (0, 1, false),
    ];
    for (i, &(mask, req, exp)) in cases.iter().enumerate() {
        grant(1, 100 + i as u64, mask).unwrap();
        assert_eq!(
            check(1, 100 + i as u64, req).unwrap(),
            exp,
            "case {i}: mask={mask:x} req={req:x}"
        );
    }
}

// === More Role Tests ===

#[test]
fn test_role_fallback_to_mask() {
    let _lock = setup();
    // No role defined, uses mask directly
    grant(1, 100, READ | WRITE).unwrap();
    assert!(check(1, 100, READ | WRITE).unwrap());
}

#[test]
fn test_role_per_object_scoped() {
    let _lock = setup();
    set_role(100, 1, READ | WRITE | DELETE).unwrap();
    set_role(101, 1, READ).unwrap();
    grant(1, 100, 1).unwrap();
    grant(1, 101, 1).unwrap();
    assert!(check(1, 100, DELETE).unwrap());
    assert!(!check(1, 101, DELETE).unwrap());
}

#[test]
fn test_get_role_undefined() {
    let _lock = setup();
    // Undefined role returns role ID itself
    assert_eq!(get_role(100, 99).unwrap(), 99);
}

// === More Inheritance Tests ===

#[test]
fn test_inherit_combines_with_direct() {
    let _lock = setup();
    grant(1, 100, READ).unwrap();
    grant(1000, 100, WRITE).unwrap();
    set_inherit(100, 1, 1000).unwrap();
    assert_eq!(get_mask(1, 100).unwrap(), READ | WRITE);
}

#[test]
fn test_inherit_dynamic_updates() {
    let _lock = setup();
    grant(1000, 100, READ | WRITE).unwrap();
    set_inherit(100, 1, 1000).unwrap();
    assert!(check(1, 100, READ | WRITE).unwrap());

    // Revoke parent's permission
    revoke(1000, 100).unwrap();
    assert!(!check(1, 100, READ).unwrap());
}

#[test]
fn test_inherit_per_object_scoped() {
    let _lock = setup();
    grant(1000, 100, READ).unwrap();
    grant(1000, 101, WRITE).unwrap();
    set_inherit(100, 1, 1000).unwrap();
    // Inheritance only on object 100, not 101
    assert!(check(1, 100, READ).unwrap());
    assert!(!check(1, 101, WRITE).unwrap());
}

#[test]
fn test_cycle_allowed_different_objects() {
    let _lock = setup();
    set_inherit(100, 1, 2).unwrap();
    // Different object = ok
    set_inherit(101, 2, 1).unwrap();
}

#[test]
fn test_cycle_allowed_after_remove() {
    let _lock = setup();
    set_inherit(100, 1, 2).unwrap();
    set_inherit(100, 2, 3).unwrap();
    remove_inherit(100, 1).unwrap();
    // Now 3 -> 1 is ok
    set_inherit(100, 3, 1).unwrap();
}

// === Labels ===

#[test]
fn test_label_unicode() {
    let _lock = setup();
    set_label(1, "日本語").unwrap();
    set_label(2, "🎉").unwrap();
    assert_eq!(get_label(1).unwrap(), Some("日本語".to_string()));
    assert_eq!(get_label(2).unwrap(), Some("🎉".to_string()));
}

#[test]
fn test_label_update() {
    let _lock = setup();
    set_label(1, "alice").unwrap();
    set_label(1, "alicia").unwrap();
    assert_eq!(get_label(1).unwrap(), Some("alicia".to_string()));
}

#[test]
fn test_list_labels() {
    let _lock = setup();
    set_label(1, "alice").unwrap();
    set_label(2, "bob").unwrap();
    assert_eq!(list_labels().unwrap().len(), 2);
}

// === Batch Edge Cases ===

#[test]
fn test_batch_grant_empty() {
    let _lock = setup();
    batch_grant(&[]).unwrap();
}

#[test]
fn test_batch_revoke_empty() {
    let _lock = setup();
    assert_eq!(batch_revoke(&[]).unwrap(), 0);
}

#[test]
fn test_batch_grant_accumulates() {
    let _lock = setup();
    batch_grant(&[(1, 100, READ), (1, 100, WRITE), (1, 100, DELETE)]).unwrap();
    assert_eq!(get_mask(1, 100).unwrap(), READ | WRITE | DELETE);
}

// === Scale Tests ===

#[test]
fn test_scale_100_users() {
    let _lock = setup();
    for u in 0..100 {
        grant(u, 1000, u + 1).unwrap();
    }
    for u in 0..100 {
        assert_eq!(get_mask(u, 1000).unwrap(), u + 1);
    }
}

#[test]
fn test_scale_100_objects() {
    let _lock = setup();
    for o in 0..100 {
        grant(1, o + 1000, o + 1).unwrap();
    }
    for o in 0..100 {
        assert_eq!(get_mask(1, o + 1000).unwrap(), o + 1);
    }
}
