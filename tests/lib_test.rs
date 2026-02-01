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

fn setup() -> (std::sync::MutexGuard<'static, ()>, u64, u64) {
    let lock = test_lock();
    INIT.call_once(|| {
        init(&test_db_path()).unwrap();
    });
    clear_all().unwrap();
    let (system, root) = bootstrap().unwrap();
    (lock, system, root)
}

// === Core Operations ===

#[test]
fn test_grant_and_check() {
    let (_lock, _, root) = setup();
    grant(root, 1, 100, READ | WRITE).unwrap();

    assert!(check(1, 100, READ).unwrap());
    assert!(check(1, 100, WRITE).unwrap());
    assert!(check(1, 100, READ | WRITE).unwrap());
    assert!(!check(1, 100, DELETE).unwrap());
}

#[test]
fn test_grant_accumulates() {
    let (_lock, _, root) = setup();
    grant(root, 1, 100, READ).unwrap();
    grant(root, 1, 100, WRITE).unwrap();

    assert_eq!(get_mask(1, 100).unwrap(), READ | WRITE);
}

#[test]
fn test_grant_set_replaces() {
    let (_lock, _, root) = setup();
    grant(root, 1, 100, READ | WRITE | DELETE).unwrap();
    grant_set(root, 1, 100, READ).unwrap();

    assert_eq!(get_mask(1, 100).unwrap(), READ);
}

#[test]
fn test_revoke() {
    let (_lock, _, root) = setup();
    grant(root, 1, 100, READ | WRITE).unwrap();
    assert!(revoke(root, 1, 100).unwrap());

    assert!(!check(1, 100, READ).unwrap());
    assert!(!revoke(root, 1, 100).unwrap()); // already revoked
}

// === Roles ===

#[test]
fn test_roles() {
    let (_lock, _, root) = setup();
    const EDITOR: u64 = 1000;

    set_role(root, 100, EDITOR, READ | WRITE).unwrap();
    grant(root, 1, 100, EDITOR).unwrap();

    assert!(check(1, 100, READ).unwrap());
    assert!(check(1, 100, WRITE).unwrap());
    assert!(!check(1, 100, DELETE).unwrap());
}

#[test]
fn test_role_update_affects_checks() {
    let (_lock, _, root) = setup();
    const EDITOR: u64 = 1000;

    set_role(root, 100, EDITOR, READ).unwrap();
    grant(root, 1, 100, EDITOR).unwrap();
    assert!(!check(1, 100, WRITE).unwrap());

    // Update role definition
    set_role(root, 100, EDITOR, READ | WRITE).unwrap();
    assert!(check(1, 100, WRITE).unwrap());
}

// === Inheritance ===

#[test]
fn test_inheritance() {
    let (_lock, _, root) = setup();
    // alice (1) inherits from managers (10)
    set_inherit(root, 100, 1, 10).unwrap();
    grant(root, 10, 100, READ | WRITE).unwrap();

    assert!(check(1, 100, READ).unwrap());
    assert!(check(1, 100, WRITE).unwrap());
}

#[test]
fn test_inheritance_chain() {
    let (_lock, _, root) = setup();
    // alice -> managers -> admins
    set_inherit(root, 100, 1, 10).unwrap();
    set_inherit(root, 100, 10, 20).unwrap();
    grant(root, 20, 100, ADMIN).unwrap();

    assert!(check(1, 100, ADMIN).unwrap());
}

#[test]
fn test_inheritance_cycle_prevention() {
    let (_lock, _, root) = setup();
    set_inherit(root, 100, 1, 2).unwrap();
    set_inherit(root, 100, 2, 3).unwrap();

    // Should fail: 3 -> 1 creates cycle
    assert!(set_inherit(root, 100, 3, 1).is_err());
}

#[test]
fn test_self_inherit_prevention() {
    let (_lock, _, root) = setup();
    assert!(set_inherit(root, 100, 1, 1).is_err());
}

#[test]
fn test_remove_inherit() {
    let (_lock, _, root) = setup();
    set_inherit(root, 100, 1, 10).unwrap();
    grant(root, 10, 100, READ).unwrap();
    assert!(check(1, 100, READ).unwrap());

    remove_inherit(root, 100, 1).unwrap();
    assert!(!check(1, 100, READ).unwrap());
}

// === Batch Operations ===

#[test]
fn test_batch_grant() {
    let (_lock, _, root) = setup();
    batch_grant(root, &[
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
    let (_lock, _, root) = setup();
    batch_grant(root, &[(1, 100, READ), (2, 100, WRITE)]).unwrap();

    let count = batch_revoke(root, &[(1, 100), (2, 100), (3, 100)]).unwrap();
    assert_eq!(count, 2); // only 2 existed
}

#[test]
fn test_transact() {
    let (_lock, _, root) = setup();
    // Use transact for internal batch operations
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
    let (_lock, _, _root) = setup();
    let alice = create_entity("alice").unwrap();
    let bob = create_entity("bob").unwrap();

    assert!(alice > 0);
    assert_eq!(bob, alice + 1);
    assert_eq!(get_label(alice).unwrap(), Some("alice".to_string()));
    assert_eq!(get_id_by_label("alice").unwrap(), Some(alice));
}

#[test]
fn test_rename_entity() {
    let (_lock, _, _root) = setup();
    let id = create_entity("alice").unwrap();
    rename_entity(id, "alicia").unwrap();

    assert_eq!(get_label(id).unwrap(), Some("alicia".to_string()));
    assert_eq!(get_id_by_label("alicia").unwrap(), Some(id));
    assert_eq!(get_id_by_label("alice").unwrap(), None);
}

#[test]
fn test_delete_entity() {
    let (_lock, _, _root) = setup();
    let id = create_entity("alice").unwrap();
    assert!(delete_entity(id).unwrap());

    assert_eq!(get_label(id).unwrap(), None);
    assert_eq!(get_id_by_label("alice").unwrap(), None);
}

#[test]
fn test_set_label() {
    let (_lock, _, _root) = setup();
    set_label(42, "answer").unwrap();

    assert_eq!(get_label(42).unwrap(), Some("answer".to_string()));
    assert_eq!(get_id_by_label("answer").unwrap(), Some(42));
}

// === Queries ===

#[test]
fn test_list_for_subject() {
    let (_lock, _, root) = setup();
    grant(root, 1, 100, READ).unwrap();
    grant(root, 1, 101, WRITE).unwrap();
    grant(root, 1, 102, DELETE).unwrap();

    let list = list_for_subject(1).unwrap();
    assert_eq!(list.len(), 3);
}

#[test]
fn test_list_for_object() {
    let (_lock, _, root) = setup();
    grant(root, 1, 100, READ).unwrap();
    grant(root, 2, 100, WRITE).unwrap();
    grant(root, 3, 100, DELETE).unwrap();

    let list = list_for_object(root, 100).unwrap();
    assert_eq!(list.len(), 3);
}

#[test]
fn test_count_functions() {
    let (_lock, _, root) = setup();
    grant(root, 1, 100, READ).unwrap();
    grant(root, 1, 101, READ).unwrap();
    grant(root, 2, 100, READ).unwrap();

    assert_eq!(count_for_subject(1).unwrap(), 2);
    assert_eq!(count_for_object(100).unwrap(), 2);
}

// === Protection Semantics ===

#[test]
fn test_grant_requires_system_permission() {
    let (_lock, system, root) = setup();

    // Unprivileged user cannot grant
    let alice = create_entity("alice").unwrap();
    assert!(grant(alice, 1, 100, READ).is_err());

    // Grant GRANT on _system to alice
    grant(root, alice, system, GRANT).unwrap();
    grant(alice, 1, 100, READ).unwrap();
    assert!(check(1, 100, READ).unwrap());
}

#[test]
fn test_revoke_requires_system_permission() {
    let (_lock, system, root) = setup();
    grant(root, 2, 100, READ).unwrap();

    // Unprivileged user cannot revoke
    let alice = create_entity("alice").unwrap();
    assert!(revoke(alice, 2, 100).is_err());

    // Root can revoke
    assert!(revoke(root, 2, 100).unwrap());

    // Grant GRANT on _system to alice
    grant(root, 3, 100, READ).unwrap();
    grant(root, alice, system, GRANT).unwrap();
    assert!(revoke(alice, 3, 100).unwrap());
}

#[test]
fn test_set_role_requires_system_admin() {
    let (_lock, system, root) = setup();

    // Unprivileged user cannot set roles
    let alice = create_entity("alice").unwrap();
    assert!(set_role(alice, 100, 1, READ | WRITE).is_err());

    // Root can
    set_role(root, 100, 1, READ | WRITE).unwrap();
    assert_eq!(get_role(100, 1).unwrap(), READ | WRITE);

    // Grant ADMIN on _system to alice
    grant(root, alice, system, ADMIN).unwrap();
    set_role(alice, 100, 2, DELETE).unwrap();
    assert_eq!(get_role(100, 2).unwrap(), DELETE);
}

#[test]
fn test_inherit_requires_system_admin() {
    let (_lock, system, root) = setup();

    let alice = create_entity("alice").unwrap();

    // Unprivileged user cannot set inheritance
    assert!(set_inherit(alice, 100, 1, 2).is_err());
    assert!(remove_inherit(alice, 100, 1).is_err());

    // Root can
    set_inherit(root, 100, 1, 2).unwrap();
    assert_eq!(get_inherit(100, 1).unwrap(), Some(2));
    remove_inherit(root, 100, 1).unwrap();
    assert_eq!(get_inherit(100, 1).unwrap(), None);

    // Grant ADMIN on _system to alice
    grant(root, alice, system, ADMIN).unwrap();
    set_inherit(alice, 100, 1, 2).unwrap();
    assert_eq!(get_inherit(100, 1).unwrap(), Some(2));
}

#[test]
fn test_list_requires_system_view() {
    let (_lock, system, root) = setup();
    grant(root, 1, 100, READ).unwrap();

    let alice = create_entity("alice").unwrap();

    // Unprivileged user cannot list
    assert!(list_for_object(alice, 100).is_err());

    // Root can
    assert_eq!(list_for_object(root, 100).unwrap().len(), 1);

    // Grant VIEW on _system to alice
    grant(root, alice, system, VIEW).unwrap();
    assert_eq!(list_for_object(alice, 100).unwrap().len(), 1);
}

// === Happy Path: Delegation ===

#[test]
fn test_root_creates_admin_user() {
    let (_lock, system, root) = setup();

    // Root creates an admin and grants them full system access
    let admin = create_entity("admin").unwrap();
    grant(root, admin, system, u64::MAX).unwrap();

    // Admin now has all powers
    assert!(check(admin, system, GRANT | ADMIN | VIEW).unwrap());

    // Admin can do operations
    let user = create_entity("user").unwrap();
    grant(admin, user, system, VIEW).unwrap();
    assert!(check(user, system, VIEW).unwrap());
}

#[test]
fn test_root_creates_limited_operator() {
    let (_lock, system, root) = setup();

    // Root creates operator with only GRANT (can assign permissions but not change roles)
    let operator = create_entity("operator").unwrap();
    grant(root, operator, system, GRANT).unwrap();

    // Operator can grant permissions to users
    let user = create_entity("user").unwrap();
    grant(operator, user, system, VIEW).unwrap();
    assert!(check(user, system, VIEW).unwrap());

    // But operator cannot set roles (needs ADMIN)
    assert!(set_role(operator, 100, 1, READ).is_err());
}

#[test]
fn test_delegation_chain() {
    let (_lock, system, root) = setup();

    // Root -> Admin -> Operator -> User
    let admin = create_entity("admin").unwrap();
    let operator = create_entity("operator").unwrap();
    let user = create_entity("user").unwrap();

    // Root grants admin full access
    grant(root, admin, system, GRANT | ADMIN | VIEW).unwrap();

    // Admin grants operator limited access
    grant(admin, operator, system, GRANT | VIEW).unwrap();

    // Operator grants user view-only
    grant(operator, user, system, VIEW).unwrap();

    // Verify permissions
    assert!(check(admin, system, ADMIN).unwrap());
    assert!(!check(operator, system, ADMIN).unwrap());
    assert!(check(operator, system, GRANT).unwrap());
    assert!(!check(user, system, GRANT).unwrap());
    assert!(check(user, system, VIEW).unwrap());
}

// === Adversarial ===

#[test]
fn test_adversarial_unprivileged_user() {
    let (_lock, system, _root) = setup();

    let attacker = create_entity("attacker").unwrap();

    // All protected ops fail
    assert!(grant(attacker, attacker, system, ADMIN).is_err());
    assert!(grant(attacker, 1, 100, READ).is_err());
    assert!(revoke(attacker, 1, 100).is_err());
    assert!(set_role(attacker, 100, 1, READ).is_err());
    assert!(set_inherit(attacker, 100, 1, 2).is_err());
    assert!(list_for_object(attacker, 100).is_err());
}

#[test]
fn test_adversarial_self_escalation() {
    let (_lock, system, root) = setup();

    // User with VIEW tries to escalate to GRANT
    let user = create_entity("user").unwrap();
    grant(root, user, system, VIEW).unwrap();

    // Cannot grant themselves more permissions
    assert!(grant(user, user, system, GRANT).is_err());
    assert!(grant(user, user, system, ADMIN).is_err());

    // Still only has VIEW
    assert!(check(user, system, VIEW).unwrap());
    assert!(!check(user, system, GRANT).unwrap());
}

#[test]
fn test_view_only_cannot_modify() {
    let (_lock, system, root) = setup();

    let viewer = create_entity("viewer").unwrap();
    grant(root, viewer, system, VIEW).unwrap();

    // Viewer can list
    grant(root, 1, 100, READ).unwrap();
    assert!(list_for_object(viewer, 100).is_ok());

    // But cannot modify anything
    assert!(grant(viewer, 2, 100, READ).is_err());
    assert!(revoke(viewer, 1, 100).is_err());
    assert!(set_role(viewer, 100, 1, READ).is_err());
}

#[test]
fn test_separate_permission_bits() {
    let (_lock, system, root) = setup();

    // Create users with different individual permissions
    let granter = create_entity("granter").unwrap();
    let admin = create_entity("admin").unwrap();
    let viewer = create_entity("viewer").unwrap();

    grant(root, granter, system, GRANT).unwrap();
    grant(root, admin, system, ADMIN).unwrap();
    grant(root, viewer, system, VIEW).unwrap();

    // Granter: can grant/revoke, cannot set roles
    grant(granter, 1, 100, READ).unwrap();
    revoke(granter, 1, 100).unwrap();
    assert!(set_role(granter, 100, 1, READ).is_err());

    // Admin: can set roles/inherit, cannot grant (no GRANT bit)
    set_role(admin, 100, 1, READ).unwrap();
    set_inherit(admin, 100, 1, 2).unwrap();
    assert!(grant(admin, 1, 100, READ).is_err());

    // Viewer: can only list
    grant(root, 1, 100, READ).unwrap();
    list_for_object(viewer, 100).unwrap();
    assert!(grant(viewer, 2, 100, READ).is_err());
    assert!(set_role(viewer, 100, 2, READ).is_err());
}

// === Bootstrap ===

#[test]
fn test_bootstrap() {
    let lock = test_lock();
    INIT.call_once(|| {
        init(&test_db_path()).unwrap();
    });
    clear_all().unwrap();

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

    drop(lock);
}

#[test]
fn test_bootstrap_only_once() {
    let (_lock, _, _) = setup();
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
fn test_check_zero_required() {
    let (_lock, _, _root) = setup();
    // 0 required = always true (no permissions needed)
    assert!(check(1, 100, 0).unwrap());
}

#[test]
fn test_get_mask_empty() {
    let (_lock, _, _root) = setup();
    assert_eq!(get_mask(999, 999).unwrap(), 0);
}

#[test]
fn test_grant_idempotent() {
    let (_lock, _, root) = setup();
    grant(root, 1, 100, READ | WRITE).unwrap();
    grant(root, 1, 100, READ | WRITE).unwrap();
    assert_eq!(get_mask(1, 100).unwrap(), READ | WRITE);
}

#[test]
fn test_revoke_nonexistent() {
    let (_lock, _, root) = setup();
    assert!(!revoke(root, 999, 999).unwrap());
}

// === More Role Tests ===

#[test]
fn test_role_fallback_to_mask() {
    let (_lock, _, root) = setup();
    // No role defined, uses mask directly
    grant(root, 1, 100, READ | WRITE).unwrap();
    assert!(check(1, 100, READ | WRITE).unwrap());
}

#[test]
fn test_role_per_object_scoped() {
    let (_lock, _, root) = setup();
    set_role(root, 100, 1, READ | WRITE | DELETE).unwrap();
    set_role(root, 101, 1, READ).unwrap();
    grant(root, 1, 100, 1).unwrap();
    grant(root, 1, 101, 1).unwrap();
    assert!(check(1, 100, DELETE).unwrap());
    assert!(!check(1, 101, DELETE).unwrap());
}

#[test]
fn test_get_role_undefined() {
    let (_lock, _, _root) = setup();
    // Undefined role returns role ID itself
    assert_eq!(get_role(100, 99).unwrap(), 99);
}

// === More Inheritance Tests ===

#[test]
fn test_inherit_combines_with_direct() {
    let (_lock, _, root) = setup();
    grant(root, 1, 100, READ).unwrap();
    grant(root, 1000, 100, WRITE).unwrap();
    set_inherit(root, 100, 1, 1000).unwrap();
    assert_eq!(get_mask(1, 100).unwrap(), READ | WRITE);
}

#[test]
fn test_inherit_dynamic_updates() {
    let (_lock, _, root) = setup();
    grant(root, 1000, 100, READ | WRITE).unwrap();
    set_inherit(root, 100, 1, 1000).unwrap();
    assert!(check(1, 100, READ | WRITE).unwrap());

    // Revoke parent's permission
    revoke(root, 1000, 100).unwrap();
    assert!(!check(1, 100, READ).unwrap());
}

#[test]
fn test_inherit_per_object_scoped() {
    let (_lock, _, root) = setup();
    grant(root, 1000, 100, READ).unwrap();
    grant(root, 1000, 101, WRITE).unwrap();
    set_inherit(root, 100, 1, 1000).unwrap();
    // Inheritance only on object 100, not 101
    assert!(check(1, 100, READ).unwrap());
    assert!(!check(1, 101, WRITE).unwrap());
}

#[test]
fn test_cycle_allowed_different_objects() {
    let (_lock, _, root) = setup();
    set_inherit(root, 100, 1, 2).unwrap();
    // Different object = ok
    set_inherit(root, 101, 2, 1).unwrap();
}

#[test]
fn test_cycle_allowed_after_remove() {
    let (_lock, _, root) = setup();
    set_inherit(root, 100, 1, 2).unwrap();
    set_inherit(root, 100, 2, 3).unwrap();
    remove_inherit(root, 100, 1).unwrap();
    // Now 3 -> 1 is ok
    set_inherit(root, 100, 3, 1).unwrap();
}

// === Labels ===

#[test]
fn test_label_unicode() {
    let (_lock, _, _root) = setup();
    set_label(1, "日本語").unwrap();
    set_label(2, "🎉").unwrap();
    assert_eq!(get_label(1).unwrap(), Some("日本語".to_string()));
    assert_eq!(get_label(2).unwrap(), Some("🎉".to_string()));
}

#[test]
fn test_label_update() {
    let (_lock, _, _root) = setup();
    set_label(1, "alice").unwrap();
    set_label(1, "alicia").unwrap();
    assert_eq!(get_label(1).unwrap(), Some("alicia".to_string()));
}

#[test]
fn test_list_labels() {
    let (_lock, _, _root) = setup();
    // _system and _root_user already exist from bootstrap
    let count_before = list_labels().unwrap().len();
    set_label(100, "alice").unwrap();
    set_label(101, "bob").unwrap();
    assert_eq!(list_labels().unwrap().len(), count_before + 2);
}

// === Batch Edge Cases ===

#[test]
fn test_batch_grant_empty() {
    let (_lock, _, root) = setup();
    batch_grant(root, &[]).unwrap();
}

#[test]
fn test_batch_revoke_empty() {
    let (_lock, _, root) = setup();
    assert_eq!(batch_revoke(root, &[]).unwrap(), 0);
}

#[test]
fn test_batch_grant_accumulates() {
    let (_lock, _, root) = setup();
    batch_grant(root, &[(1, 100, READ), (1, 100, WRITE), (1, 100, DELETE)]).unwrap();
    assert_eq!(get_mask(1, 100).unwrap(), READ | WRITE | DELETE);
}
