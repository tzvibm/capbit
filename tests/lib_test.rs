//! Integration tests for Capbit public API

use capbit::*;
use std::sync::Once;
use std::thread;
use std::time::Duration;

static INIT: Once = Once::new();

/// Wait for planner to flush (auto-flushes every 20ms)
fn wait_flush() {
    thread::sleep(Duration::from_millis(30));
}

fn test_db_path() -> String {
    std::env::var("CAPBIT_TEST_DB").unwrap_or_else(|_| {
        let tmp = std::env::temp_dir();
        tmp.join("capbit_test.mdb").to_string_lossy().to_string()
    })
}

fn setup() -> std::sync::MutexGuard<'static, ()> {
    let lock = test_lock();
    INIT.call_once(|| { init(&test_db_path()).unwrap(); });
    clear_all().unwrap();
    lock
}

// === Core Operations (use transact for setup) ===

#[test]
fn test_grant_and_check() {
    let _lock = setup();
    transact(|tx| tx.grant(1, 100, READ | WRITE)).unwrap();

    assert!(check(1, 100, READ).unwrap());
    assert!(check(1, 100, WRITE).unwrap());
    assert!(check(1, 100, READ | WRITE).unwrap());
    assert!(!check(1, 100, DELETE).unwrap());
}

#[test]
fn test_grant_accumulates() {
    let _lock = setup();
    transact(|tx| { tx.grant(1, 100, READ)?; tx.grant(1, 100, WRITE) }).unwrap();

    assert_eq!(get_mask(1, 100).unwrap(), READ | WRITE);
}

#[test]
fn test_grant_set_replaces() {
    let _lock = setup();
    transact(|tx| tx.grant(1, 100, READ | WRITE | DELETE)).unwrap();
    transact(|tx| tx.grant_set(1, 100, READ)).unwrap();

    assert_eq!(get_mask(1, 100).unwrap(), READ);
}

#[test]
fn test_revoke() {
    let _lock = setup();
    transact(|tx| tx.grant(1, 100, READ | WRITE)).unwrap();
    transact(|tx| tx.revoke(1, 100)).unwrap();

    assert!(!check(1, 100, READ).unwrap());
}

// === Roles ===

#[test]
fn test_roles() {
    let _lock = setup();
    const EDITOR: u64 = 1000;

    transact(|tx| {
        tx.set_role(100, EDITOR, READ | WRITE)?;
        tx.grant(1, 100, EDITOR)
    }).unwrap();

    assert!(check(1, 100, READ).unwrap());
    assert!(check(1, 100, WRITE).unwrap());
    assert!(!check(1, 100, DELETE).unwrap());
}

#[test]
fn test_role_update_affects_checks() {
    let _lock = setup();
    const EDITOR: u64 = 1000;

    transact(|tx| {
        tx.set_role(100, EDITOR, READ)?;
        tx.grant(1, 100, EDITOR)
    }).unwrap();
    assert!(!check(1, 100, WRITE).unwrap());

    transact(|tx| tx.set_role(100, EDITOR, READ | WRITE)).unwrap();
    assert!(check(1, 100, WRITE).unwrap());
}

// === Inheritance ===

#[test]
fn test_inheritance() {
    let _lock = setup();
    transact(|tx| {
        tx.set_inherit(100, 1, 10)?;
        tx.grant(10, 100, READ | WRITE)
    }).unwrap();

    assert!(check(1, 100, READ).unwrap());
    assert!(check(1, 100, WRITE).unwrap());
}

#[test]
fn test_inheritance_chain() {
    let _lock = setup();
    transact(|tx| {
        tx.set_inherit(100, 1, 10)?;
        tx.set_inherit(100, 10, 20)?;
        tx.grant(20, 100, ADMIN)
    }).unwrap();

    assert!(check(1, 100, ADMIN).unwrap());
}

#[test]
fn test_inheritance_cycle_prevention() {
    let _lock = setup();
    transact(|tx| {
        tx.set_inherit(100, 1, 2)?;
        tx.set_inherit(100, 2, 3)
    }).unwrap();

    assert!(transact(|tx| tx.set_inherit(100, 3, 1)).is_err());
}

#[test]
fn test_self_inherit_prevention() {
    let _lock = setup();
    assert!(transact(|tx| tx.set_inherit(100, 1, 1)).is_err());
}

#[test]
fn test_remove_inherit() {
    let _lock = setup();
    transact(|tx| {
        tx.set_inherit(100, 1, 10)?;
        tx.grant(10, 100, READ)
    }).unwrap();
    assert!(check(1, 100, READ).unwrap());

    transact(|tx| tx.remove_inherit(100, 1)).unwrap();
    assert!(!check(1, 100, READ).unwrap());
}

// === Transact ===

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

#[test]
fn test_planner_auto_flush() {
    let _lock = setup();
    // Give actor GRANT on object 100
    transact(|tx| tx.grant(99, 100, GRANT)).unwrap();

    for i in 0..10u64 {
        grant(99, i, 100, READ).unwrap();
    }
    wait_flush();  // Planner auto-flushes within 20ms

    for i in 0..10u64 {
        assert!(check(i, 100, READ).unwrap());
    }
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
    transact(|tx| {
        tx.grant(1, 100, READ)?;
        tx.grant(1, 101, WRITE)?;
        tx.grant(1, 102, DELETE)
    }).unwrap();

    let list = list_for_subject(1).unwrap();
    assert_eq!(list.len(), 3);
}

#[test]
fn test_list_for_object() {
    let _lock = setup();
    transact(|tx| {
        tx.grant(1, 100, READ)?;
        tx.grant(2, 100, WRITE)?;
        tx.grant(3, 100, DELETE)?;
        tx.grant(99, 100, VIEW)  // Give actor VIEW on 100
    }).unwrap();

    let list = list_for_object(99, 100).unwrap();
    assert_eq!(list.len(), 4);
}

#[test]
fn test_count_functions() {
    let _lock = setup();
    transact(|tx| {
        tx.grant(1, 100, READ)?;
        tx.grant(1, 101, READ)?;
        tx.grant(2, 100, READ)
    }).unwrap();

    assert_eq!(count_for_subject(1).unwrap(), 2);
    assert_eq!(count_for_object(100).unwrap(), 2);
}

// === Protection Semantics (per-object) ===

#[test]
fn test_grant_requires_permission_on_target() {
    let _lock = setup();

    // alice has no permissions
    let alice = 1u64;
    assert!(grant(alice, 2, 100, READ).is_err());

    // Give alice GRANT on object 100
    transact(|tx| tx.grant(alice, 100, GRANT)).unwrap();
    grant(alice, 2, 100, READ).unwrap();
    wait_flush();
    assert!(check(2, 100, READ).unwrap());
}

#[test]
fn test_revoke_requires_permission_on_target() {
    let _lock = setup();
    transact(|tx| tx.grant(2, 100, READ)).unwrap();

    let alice = 1u64;
    assert!(revoke(alice, 2, 100).is_err());

    transact(|tx| tx.grant(alice, 100, GRANT)).unwrap();
    revoke(alice, 2, 100).unwrap();
    wait_flush();
}

#[test]
fn test_set_role_requires_admin_on_system() {
    let _lock = setup();
    let (system, _) = bootstrap().unwrap();

    let alice = 1u64;
    assert!(set_role(alice, 100, 1, READ | WRITE).is_err());

    // Give alice ADMIN on _system
    transact(|tx| tx.grant(alice, system, ADMIN)).unwrap();
    set_role(alice, 100, 1, READ | WRITE).unwrap();
    wait_flush();
    assert_eq!(get_role(100, 1).unwrap(), READ | WRITE);
}

#[test]
fn test_inherit_requires_admin_on_system() {
    let _lock = setup();
    let (system, _) = bootstrap().unwrap();

    let alice = 1u64;
    assert!(set_inherit(alice, 100, 1, 2).is_err());
    assert!(remove_inherit(alice, 100, 1).is_err());

    // Give alice ADMIN on _system
    transact(|tx| tx.grant(alice, system, ADMIN)).unwrap();
    set_inherit(alice, 100, 1, 2).unwrap();
    wait_flush();
    assert_eq!(get_inherit(100, 1).unwrap(), Some(2));
    remove_inherit(alice, 100, 1).unwrap();
    wait_flush();
    assert_eq!(get_inherit(100, 1).unwrap(), None);
}

#[test]
fn test_list_requires_view_on_target() {
    let _lock = setup();
    transact(|tx| tx.grant(1, 100, READ)).unwrap();

    let alice = 2u64;
    assert!(list_for_object(alice, 100).is_err());

    transact(|tx| tx.grant(alice, 100, VIEW)).unwrap();
    // 2 grants: (1, READ) and (alice, VIEW)
    assert_eq!(list_for_object(alice, 100).unwrap().len(), 2);
}

// === Adversarial ===

#[test]
fn test_adversarial_unprivileged_user() {
    let _lock = setup();

    let attacker = 999u64;

    // All protected ops fail (attacker has no permissions on any object)
    assert!(grant(attacker, 1, 100, READ).is_err());
    assert!(revoke(attacker, 1, 100).is_err());
    assert!(set_role(attacker, 100, 1, READ).is_err());
    assert!(set_inherit(attacker, 100, 1, 2).is_err());
    assert!(list_for_object(attacker, 100).is_err());
}

#[test]
fn test_adversarial_self_escalation() {
    let _lock = setup();

    // User with VIEW on object 100 tries to escalate
    let user = 1u64;
    transact(|tx| tx.grant(user, 100, VIEW)).unwrap();

    // Cannot grant themselves more permissions (no GRANT bit)
    assert!(grant(user, user, 100, GRANT).is_err());
    assert!(grant(user, user, 100, ADMIN).is_err());

    // Still only has VIEW
    assert!(check(user, 100, VIEW).unwrap());
    assert!(!check(user, 100, GRANT).unwrap());
}

#[test]
fn test_view_only_cannot_modify() {
    let _lock = setup();

    let viewer = 1u64;
    transact(|tx| {
        tx.grant(viewer, 100, VIEW)?;
        tx.grant(2, 100, READ)
    }).unwrap();

    // Viewer can list
    assert!(list_for_object(viewer, 100).is_ok());

    // But cannot modify anything
    assert!(grant(viewer, 3, 100, READ).is_err());
    assert!(revoke(viewer, 2, 100).is_err());
    assert!(set_role(viewer, 100, 1, READ).is_err());
}

#[test]
fn test_separate_permission_bits() {
    let _lock = setup();
    let (system, _) = bootstrap().unwrap();

    let granter = 1u64;
    let admin = 2u64;
    let viewer = 3u64;

    transact(|tx| {
        tx.grant(granter, 100, GRANT)?;      // GRANT on target object
        tx.grant(admin, system, ADMIN)?;     // ADMIN on _system
        tx.grant(viewer, 100, VIEW)          // VIEW on target object
    }).unwrap();

    // Granter: can grant/revoke on 100, cannot set roles (no ADMIN on _system)
    grant(granter, 10, 100, READ).unwrap();
    wait_flush();
    revoke(granter, 10, 100).unwrap();
    wait_flush();
    assert!(set_role(granter, 100, 1, READ).is_err());

    // Admin: can set roles/inherit (ADMIN on _system), cannot grant on 100 (no GRANT on 100)
    set_role(admin, 100, 1, READ).unwrap();
    set_inherit(admin, 100, 10, 20).unwrap();
    wait_flush();
    assert!(grant(admin, 10, 100, READ).is_err());

    // Viewer: can only list on 100
    transact(|tx| tx.grant(10, 100, READ)).unwrap();
    list_for_object(viewer, 100).unwrap();
    assert!(grant(viewer, 11, 100, READ).is_err());
    assert!(set_role(viewer, 100, 2, READ).is_err());
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

    // Root user has full system permissions via ROLE_FULL
    assert!(check(root_user, system, GRANT | ADMIN | VIEW).unwrap());

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
fn test_check_zero_required() {
    let _lock = setup();
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
    transact(|tx| {
        tx.grant(1, 100, READ | WRITE)?;
        tx.grant(1, 100, READ | WRITE)
    }).unwrap();
    assert_eq!(get_mask(1, 100).unwrap(), READ | WRITE);
}

#[test]
fn test_revoke_nonexistent() {
    let _lock = setup();
    transact(|tx| tx.grant(99, 999, GRANT)).unwrap();
    revoke(99, 1, 999).unwrap();
    wait_flush();
}

// === More Role Tests ===

#[test]
fn test_role_fallback_to_mask() {
    let _lock = setup();
    transact(|tx| tx.grant(1, 100, READ | WRITE)).unwrap();
    assert!(check(1, 100, READ | WRITE).unwrap());
}

#[test]
fn test_role_per_object_scoped() {
    let _lock = setup();
    transact(|tx| {
        tx.set_role(100, 1, READ | WRITE | DELETE)?;
        tx.set_role(101, 1, READ)?;
        tx.grant(1, 100, 1)?;
        tx.grant(1, 101, 1)
    }).unwrap();
    assert!(check(1, 100, DELETE).unwrap());
    assert!(!check(1, 101, DELETE).unwrap());
}

#[test]
fn test_get_role_undefined() {
    let _lock = setup();
    assert_eq!(get_role(100, 99).unwrap(), 99);
}

// === More Inheritance Tests ===

#[test]
fn test_inherit_combines_with_direct() {
    let _lock = setup();
    transact(|tx| {
        tx.grant(1, 100, READ)?;
        tx.grant(1000, 100, WRITE)?;
        tx.set_inherit(100, 1, 1000)
    }).unwrap();
    assert_eq!(get_mask(1, 100).unwrap(), READ | WRITE);
}

#[test]
fn test_inherit_dynamic_updates() {
    let _lock = setup();
    transact(|tx| {
        tx.grant(1000, 100, READ | WRITE)?;
        tx.set_inherit(100, 1, 1000)
    }).unwrap();
    assert!(check(1, 100, READ | WRITE).unwrap());

    transact(|tx| tx.revoke(1000, 100)).unwrap();
    assert!(!check(1, 100, READ).unwrap());
}

#[test]
fn test_inherit_per_object_scoped() {
    let _lock = setup();
    transact(|tx| {
        tx.grant(1000, 100, READ)?;
        tx.grant(1000, 101, WRITE)?;
        tx.set_inherit(100, 1, 1000)
    }).unwrap();
    assert!(check(1, 100, READ).unwrap());
    assert!(!check(1, 101, WRITE).unwrap());
}

#[test]
fn test_cycle_allowed_different_objects() {
    let _lock = setup();
    transact(|tx| {
        tx.set_inherit(100, 1, 2)?;
        tx.set_inherit(101, 2, 1)
    }).unwrap();
}

#[test]
fn test_cycle_allowed_after_remove() {
    let _lock = setup();
    transact(|tx| {
        tx.set_inherit(100, 1, 2)?;
        tx.set_inherit(100, 2, 3)
    }).unwrap();
    transact(|tx| tx.remove_inherit(100, 1)).unwrap();
    transact(|tx| tx.set_inherit(100, 3, 1)).unwrap();
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
    set_label(100, "alice").unwrap();
    set_label(101, "bob").unwrap();
    let labels = list_labels().unwrap();
    assert!(labels.len() >= 2);
}

// === Inheritance-based Permission Delegation ===

#[test]
fn test_inherit_admin_from_org() {
    let _lock = setup();
    let (system, _) = bootstrap().unwrap();

    let org = 10u64;
    let user = 1u64;

    // Org has ADMIN on _system
    transact(|tx| {
        tx.grant(org, system, ADMIN)?;
        // User inherits from org ON _system
        tx.set_inherit(system, user, org)
    }).unwrap();

    // User can now set_role (inherits ADMIN from org)
    set_role(user, 100, 1, READ | WRITE).unwrap();
    wait_flush();
    assert_eq!(get_role(100, 1).unwrap(), READ | WRITE);

    // User can set_inherit too
    set_inherit(user, 100, 5, 6).unwrap();
    wait_flush();
    assert_eq!(get_inherit(100, 5).unwrap(), Some(6));
}

#[test]
fn test_inherit_grant_from_org() {
    let _lock = setup();

    let org = 10u64;
    let user = 1u64;
    let doc = 100u64;

    // Org has GRANT on doc
    transact(|tx| {
        tx.grant(org, doc, GRANT)?;
        // User inherits from org ON doc
        tx.set_inherit(doc, user, org)
    }).unwrap();

    // User can now grant on doc (inherits GRANT from org)
    grant(user, 50, doc, READ).unwrap();
    wait_flush();
    assert!(check(50, doc, READ).unwrap());

    // User can revoke too
    revoke(user, 50, doc).unwrap();
    wait_flush();
    assert!(!check(50, doc, READ).unwrap());
}

#[test]
fn test_inherit_view_from_org() {
    let _lock = setup();

    let org = 10u64;
    let user = 1u64;
    let doc = 100u64;

    // Setup: some grants on doc
    transact(|tx| {
        tx.grant(50, doc, READ)?;
        tx.grant(51, doc, WRITE)?;
        // Org has VIEW on doc
        tx.grant(org, doc, VIEW)?;
        // User inherits from org ON doc
        tx.set_inherit(doc, user, org)
    }).unwrap();

    // User can now list_for_object (inherits VIEW from org)
    let list = list_for_object(user, doc).unwrap();
    assert!(list.len() >= 2);
}

#[test]
fn test_revoke_org_cascades_to_users() {
    let _lock = setup();
    let (system, _) = bootstrap().unwrap();

    let org = 10u64;
    let user = 1u64;

    // Setup: user inherits ADMIN from org
    transact(|tx| {
        tx.grant(org, system, ADMIN)?;
        tx.set_inherit(system, user, org)
    }).unwrap();

    // User can set_role
    assert!(set_role(user, 100, 1, READ).is_ok());
    wait_flush();

    // Revoke org's ADMIN
    transact(|tx| tx.revoke(org, system)).unwrap();

    // User can no longer set_role
    assert!(set_role(user, 100, 2, WRITE).is_err());
}

#[test]
fn test_multi_level_inheritance() {
    let _lock = setup();
    let (system, _) = bootstrap().unwrap();

    let corp = 100u64;
    let dept = 10u64;
    let user = 1u64;

    // Chain: user → dept → corp, all on _system
    transact(|tx| {
        tx.grant(corp, system, ADMIN)?;
        tx.set_inherit(system, dept, corp)?;
        tx.set_inherit(system, user, dept)
    }).unwrap();

    // User inherits ADMIN through 2 levels
    set_role(user, 200, 1, READ).unwrap();
    wait_flush();
    assert_eq!(get_role(200, 1).unwrap(), READ);
}

#[test]
fn test_inheritance_visibility() {
    let _lock = setup();
    let (system, _) = bootstrap().unwrap();

    let org = 10u64;
    let user1 = 1u64;
    let user2 = 2u64;

    // Both users inherit from org
    transact(|tx| {
        tx.grant(org, system, ADMIN | VIEW)?;
        tx.set_inherit(system, user1, org)?;
        tx.set_inherit(system, user2, org)
    }).unwrap();

    // Can query who has access via list_for_object on _system
    transact(|tx| tx.grant(99, system, VIEW)).unwrap();
    let list = list_for_object(99, system).unwrap();

    // Should see org's grant (users inherit, not direct grants)
    assert!(list.iter().any(|(id, _)| *id == org));
}

// === Concurrent ===

#[test]
fn test_concurrent_writes() {
    let _lock = setup();

    std::thread::scope(|s| {
        for t in 0..4u64 {
            s.spawn(move || {
                transact(|tx| {
                    for i in 0..100u64 {
                        tx.grant(t * 1000 + i, 100, READ)?;
                    }
                    Ok(())
                }).unwrap();
            });
        }
    });

    for t in 0..4u64 {
        for i in 0..100u64 {
            assert_eq!(get_mask(t * 1000 + i, 100).unwrap(), READ);
        }
    }
}
