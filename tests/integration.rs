//! Capbit Integration Tests - 150+ test cases
use capbit::*;

macro_rules! test {
    ($name:ident, $body:expr) => {
        #[test] fn $name() { let _l = test_lock(); init("./data/test.mdb").unwrap(); clear_all().unwrap(); $body }
    };
}

// ═══════════════════════════════════════════════════════════════════════════
// CORE OPERATIONS (grant, revoke, check, get_mask)
// ═══════════════════════════════════════════════════════════════════════════

test!(grant_single, { grant(1, 100, 0x07).unwrap(); assert!(check(1, 100, 0x01).unwrap()); });
test!(grant_all_bits, { grant(1, 100, 0x07).unwrap(); assert!(check(1, 100, 0x07).unwrap()); });
test!(grant_missing_bit, { grant(1, 100, 0x07).unwrap(); assert!(!check(1, 100, 0x08).unwrap()); });
test!(grant_accumulates, { grant(1, 100, 0x01).unwrap(); grant(1, 100, 0x02).unwrap(); assert_eq!(get_mask(1, 100).unwrap(), 0x03); });
test!(grant_idempotent, { grant(1, 100, 0x07).unwrap(); grant(1, 100, 0x07).unwrap(); assert_eq!(get_mask(1, 100).unwrap(), 0x07); });
test!(revoke_clears, { grant(1, 100, 0x07).unwrap(); revoke(1, 100).unwrap(); assert!(!check(1, 100, 0x01).unwrap()); });
test!(revoke_nonexistent, { assert!(!revoke(1, 100).unwrap()); });
test!(get_mask_empty, { assert_eq!(get_mask(1, 100).unwrap(), 0); });
test!(check_zero_required, { assert!(check(1, 100, 0).unwrap()); }); // 0 required = always true

// ═══════════════════════════════════════════════════════════════════════════
// TABLE-DRIVEN: Capability bit combinations
// ═══════════════════════════════════════════════════════════════════════════

test!(cap_bits_all, {
    let cases: &[(u64, u64, bool)] = &[
        (READ, READ, true), (READ, WRITE, false), (READ|WRITE, READ, true),
        (READ|WRITE, WRITE, true), (READ|WRITE, DELETE, false),
        (READ|WRITE|DELETE, READ|WRITE, true), (0xFF, 0x0F, true), (0x0F, 0xFF, false),
        (ADMIN, ADMIN, true), (ADMIN, READ, false), (u64::MAX, u64::MAX, true),
        (u64::MAX, 1, true), (1, u64::MAX, false), (0, 0, true), (0, 1, false),
    ];
    for (i, &(mask, req, exp)) in cases.iter().enumerate() {
        grant(1, 100+i as u64, mask).unwrap();
        assert_eq!(check(1, 100+i as u64, req).unwrap(), exp, "case {i}: mask={mask:x} req={req:x}");
    }
});

// ═══════════════════════════════════════════════════════════════════════════
// TABLE-DRIVEN: Edge case IDs
// ═══════════════════════════════════════════════════════════════════════════

test!(edge_ids, {
    let ids: &[u64] = &[0, 1, 2, 100, 1000, u64::MAX/2, u64::MAX-1, u64::MAX];
    for &s in ids { for &o in ids {
        grant(s, o, 0x07).unwrap();
        assert!(check(s, o, 0x07).unwrap(), "s={s} o={o}");
    }}
});

// ═══════════════════════════════════════════════════════════════════════════
// ISOLATION: No cross-contamination
// ═══════════════════════════════════════════════════════════════════════════

test!(isolation_subjects, {
    for s in 1..=10 { grant(s, 100, s).unwrap(); }
    for s in 1..=10 { assert_eq!(get_mask(s, 100).unwrap(), s); }
});

test!(isolation_objects, {
    for o in 100..=110 { grant(1, o, o-99).unwrap(); }
    for o in 100..=110 { assert_eq!(get_mask(1, o).unwrap(), o-99); }
});

test!(isolation_pairs, {
    grant(1, 100, 0x01).unwrap(); grant(2, 101, 0x02).unwrap();
    assert!(!check(1, 101, 0x01).unwrap()); assert!(!check(2, 100, 0x02).unwrap());
});

// ═══════════════════════════════════════════════════════════════════════════
// LIST OPERATIONS
// ═══════════════════════════════════════════════════════════════════════════

test!(list_subject_empty, { assert!(list_for_subject(1).unwrap().is_empty()); });
test!(list_object_empty, { assert!(list_for_object(100).unwrap().is_empty()); });
test!(list_subject_multi, {
    for o in 100..=105 { grant(1, o, o-99).unwrap(); }
    let l = list_for_subject(1).unwrap();
    assert_eq!(l.len(), 6);
});
test!(list_object_multi, {
    for s in 1..=5 { grant(s, 100, s).unwrap(); }
    let l = list_for_object(100).unwrap();
    assert_eq!(l.len(), 5);
});
test!(list_after_revoke, {
    grant(1, 100, 1).unwrap(); grant(1, 101, 2).unwrap();
    revoke(1, 100).unwrap();
    assert_eq!(list_for_subject(1).unwrap().len(), 1);
});

// ═══════════════════════════════════════════════════════════════════════════
// BATCH OPERATIONS
// ═══════════════════════════════════════════════════════════════════════════

test!(batch_grant_empty, { batch_grant(&[]).unwrap(); });
test!(batch_grant_single, { batch_grant(&[(1, 100, 7)]).unwrap(); assert!(check(1, 100, 7).unwrap()); });
test!(batch_grant_many, {
    let g: Vec<_> = (0..100).map(|i| (i, 1000+i, 7u64)).collect();
    batch_grant(&g).unwrap();
    for i in 0..100 { assert!(check(i, 1000+i, 7).unwrap()); }
});
test!(batch_grant_accumulates, {
    batch_grant(&[(1, 100, 1), (1, 100, 2), (1, 100, 4)]).unwrap();
    assert_eq!(get_mask(1, 100).unwrap(), 7);
});
test!(batch_revoke_empty, { assert_eq!(batch_revoke(&[]).unwrap(), 0); });
test!(batch_revoke_many, {
    batch_grant(&[(1, 100, 7), (2, 101, 7), (3, 102, 7)]).unwrap();
    assert_eq!(batch_revoke(&[(1, 100), (2, 101), (3, 102)]).unwrap(), 3);
});
test!(batch_revoke_partial, {
    batch_grant(&[(1, 100, 7)]).unwrap();
    assert_eq!(batch_revoke(&[(1, 100), (2, 101), (3, 102)]).unwrap(), 1);
});

// ═══════════════════════════════════════════════════════════════════════════
// ROLES
// ═══════════════════════════════════════════════════════════════════════════

test!(role_basic, {
    set_role(100, 1, 0x03).unwrap();
    grant(1, 100, 1).unwrap();
    assert!(check(1, 100, 0x03).unwrap());
});
test!(role_multiple, {
    set_role(100, 1, 0x0F).unwrap(); set_role(100, 2, 0x01).unwrap();
    grant(1, 100, 1).unwrap(); grant(2, 100, 2).unwrap();
    assert!(check(1, 100, 0x0F).unwrap()); assert!(check(2, 100, 0x01).unwrap());
    assert!(!check(2, 100, 0x02).unwrap());
});
test!(role_update, {
    set_role(100, 1, 0x03).unwrap(); grant(1, 100, 1).unwrap();
    assert!(check(1, 100, 0x03).unwrap());
    set_role(100, 1, 0x0F).unwrap();
    assert!(check(1, 100, 0x0F).unwrap());
});
test!(role_fallback, {
    grant(1, 100, 0x03).unwrap(); // no role defined, uses mask directly
    assert!(check(1, 100, 0x03).unwrap());
});
test!(role_per_object, {
    set_role(100, 1, 0x0F).unwrap(); set_role(101, 1, 0x01).unwrap();
    grant(1, 100, 1).unwrap(); grant(1, 101, 1).unwrap();
    assert!(check(1, 100, 0x0F).unwrap()); assert!(check(1, 101, 0x01).unwrap());
});
test!(get_role_undefined, { assert_eq!(get_role(100, 99).unwrap(), 99); });
test!(get_role_defined, { set_role(100, 1, 0xFF).unwrap(); assert_eq!(get_role(100, 1).unwrap(), 0xFF); });

// ═══════════════════════════════════════════════════════════════════════════
// INHERITANCE
// ═══════════════════════════════════════════════════════════════════════════

test!(inherit_basic, {
    grant(1000, 100, 0x03).unwrap();
    set_inherit(100, 1, 1000).unwrap();
    assert!(check(1, 100, 0x03).unwrap());
});
test!(inherit_combines, {
    grant(1, 100, 0x01).unwrap(); grant(1000, 100, 0x02).unwrap();
    set_inherit(100, 1, 1000).unwrap();
    assert_eq!(get_mask(1, 100).unwrap(), 0x03);
});
test!(inherit_chain, {
    grant(1000, 100, 0x01).unwrap();
    set_inherit(100, 500, 1000).unwrap();
    set_inherit(100, 1, 500).unwrap();
    assert!(check(1, 100, 0x01).unwrap());
});
test!(inherit_remove, {
    grant(1000, 100, 0x03).unwrap();
    set_inherit(100, 1, 1000).unwrap();
    remove_inherit(100, 1).unwrap();
    assert!(!check(1, 100, 0x01).unwrap());
});
test!(inherit_dynamic, {
    grant(1000, 100, 0x03).unwrap();
    set_inherit(100, 1, 1000).unwrap();
    assert!(check(1, 100, 0x03).unwrap());
    revoke(1000, 100).unwrap();
    assert!(!check(1, 100, 0x01).unwrap());
});
test!(inherit_per_object, {
    grant(1000, 100, 0x01).unwrap(); grant(1000, 101, 0x02).unwrap();
    set_inherit(100, 1, 1000).unwrap();
    assert!(check(1, 100, 0x01).unwrap());
    assert!(!check(1, 101, 0x01).unwrap()); // no inheritance on 101
});
test!(get_inherit_none, { assert_eq!(get_inherit(100, 1).unwrap(), None); });
test!(get_inherit_some, { set_inherit(100, 1, 1000).unwrap(); assert_eq!(get_inherit(100, 1).unwrap(), Some(1000)); });

// Roles + Inheritance combined
test!(roles_with_inherit, {
    set_role(100, 1, 0x0F).unwrap(); set_role(100, 2, 0x01).unwrap();
    grant(1000, 100, 2).unwrap();
    grant(1, 100, 1).unwrap();
    set_inherit(100, 2, 1000).unwrap();
    assert!(check(1, 100, 0x0F).unwrap());
    assert!(check(2, 100, 0x01).unwrap());
});

// ═══════════════════════════════════════════════════════════════════════════
// CYCLE PREVENTION
// ═══════════════════════════════════════════════════════════════════════════

test!(no_self_inherit, { assert!(set_inherit(100, 1, 1).is_err()); });
test!(no_direct_cycle, {
    set_inherit(100, 1, 2).unwrap();
    assert!(set_inherit(100, 2, 1).is_err());
});
test!(no_chain_cycle, {
    set_inherit(100, 1, 2).unwrap();
    set_inherit(100, 2, 3).unwrap();
    set_inherit(100, 3, 4).unwrap();
    assert!(set_inherit(100, 4, 1).is_err());
});
test!(cycle_different_objects, {
    set_inherit(100, 1, 2).unwrap();
    set_inherit(101, 2, 1).unwrap(); // different object, ok
});
test!(cycle_after_remove, {
    set_inherit(100, 1, 2).unwrap();
    set_inherit(100, 2, 3).unwrap();
    remove_inherit(100, 1).unwrap();
    set_inherit(100, 3, 1).unwrap(); // now ok
});

// ═══════════════════════════════════════════════════════════════════════════
// BOOTSTRAP
// ═══════════════════════════════════════════════════════════════════════════

test!(bootstrap_initial, { assert!(!is_bootstrapped().unwrap()); });
test!(bootstrap_sets_flag, { bootstrap(1).unwrap(); assert!(is_bootstrapped().unwrap()); });
test!(bootstrap_sets_root, { bootstrap(1).unwrap(); assert_eq!(get_root().unwrap(), Some(1)); });
test!(bootstrap_grants_admin, { bootstrap(1).unwrap(); assert!(check(1, 1, ADMIN).unwrap()); });
test!(bootstrap_once_only, { bootstrap(1).unwrap(); assert!(bootstrap(2).is_err()); });
test!(bootstrap_custom_root, { bootstrap(999).unwrap(); assert_eq!(get_root().unwrap(), Some(999)); });

// ═══════════════════════════════════════════════════════════════════════════
// PROTECTED OPERATIONS
// ═══════════════════════════════════════════════════════════════════════════

test!(protected_grant_root, {
    bootstrap(1).unwrap();
    protected_grant(1, 2, 100, 0xFF).unwrap();
    assert!(check(2, 100, 0xFF).unwrap());
});
test!(protected_grant_admin, {
    bootstrap(1).unwrap();
    grant(1, 100, ADMIN).unwrap();
    protected_grant(1, 2, 100, 0xFF).unwrap();
});
test!(protected_grant_subset, {
    bootstrap(1).unwrap();
    grant(1, 100, ADMIN).unwrap();
    protected_grant(1, 2, 100, 0x0F).unwrap();
    protected_grant(2, 3, 100, 0x03).unwrap();
    assert!(check(3, 100, 0x03).unwrap());
});
test!(protected_grant_exceeds, {
    bootstrap(1).unwrap();
    grant(1, 100, ADMIN).unwrap();
    protected_grant(1, 2, 100, 0x0F).unwrap();
    assert!(protected_grant(2, 3, 100, 0xFF).is_err());
});
test!(protected_revoke_admin, {
    bootstrap(1).unwrap();
    grant(1, 100, ADMIN).unwrap();
    grant(2, 100, 0x07).unwrap();
    protected_revoke(1, 2, 100).unwrap();
    assert!(!check(2, 100, 0x01).unwrap());
});
test!(protected_revoke_no_admin, {
    bootstrap(1).unwrap();
    grant(2, 100, 0x07).unwrap();
    grant(3, 100, 0x07).unwrap();
    assert!(protected_revoke(2, 3, 100).is_err());
});
test!(protected_set_role_admin, {
    bootstrap(1).unwrap();
    grant(1, 100, ADMIN).unwrap();
    protected_set_role(1, 100, 1, 0xFF).unwrap();
    assert_eq!(get_role(100, 1).unwrap(), 0xFF);
});
test!(protected_set_role_no_admin, {
    bootstrap(1).unwrap();
    grant(2, 100, 0x07).unwrap();
    assert!(protected_set_role(2, 100, 1, 0xFF).is_err());
});
test!(protected_inherit_admin, {
    bootstrap(1).unwrap();
    grant(1, 100, ADMIN).unwrap();
    protected_set_inherit(1, 100, 2, 3).unwrap();
});
test!(protected_inherit_no_admin, {
    bootstrap(1).unwrap();
    grant(2, 100, 0x07).unwrap();
    assert!(protected_set_inherit(2, 100, 3, 4).is_err());
});

// ═══════════════════════════════════════════════════════════════════════════
// LABELS
// ═══════════════════════════════════════════════════════════════════════════

test!(label_set_get, {
    set_label(1, "alice").unwrap();
    assert_eq!(get_label(1).unwrap(), Some("alice".into()));
});
test!(label_get_none, { assert_eq!(get_label(999).unwrap(), None); });
test!(label_by_name, {
    set_label(1, "alice").unwrap();
    assert_eq!(get_id_by_label("alice").unwrap(), Some(1));
});
test!(label_by_name_none, { assert_eq!(get_id_by_label("nobody").unwrap(), None); });
test!(label_update, {
    set_label(1, "alice").unwrap();
    set_label(1, "alicia").unwrap();
    assert_eq!(get_label(1).unwrap(), Some("alicia".into()));
});
test!(label_list, {
    set_label(1, "alice").unwrap(); set_label(2, "bob").unwrap();
    assert_eq!(list_labels().unwrap().len(), 2);
});
test!(label_unicode, {
    set_label(1, "日本語").unwrap(); set_label(2, "🎉").unwrap();
    assert_eq!(get_label(1).unwrap(), Some("日本語".into()));
    assert_eq!(get_label(2).unwrap(), Some("🎉".into()));
});

// ═══════════════════════════════════════════════════════════════════════════
// ENTITIES (CRUD)
// ═══════════════════════════════════════════════════════════════════════════

test!(entity_create, {
    let id = create_entity("alice").unwrap();
    assert!(id >= 1);
    assert_eq!(get_label(id).unwrap(), Some("alice".into()));
});
test!(entity_create_auto_increment, {
    let a = create_entity("alice").unwrap();
    let b = create_entity("bob").unwrap();
    assert_eq!(b, a + 1);
});
test!(entity_rename, {
    let id = create_entity("alice").unwrap();
    rename_entity(id, "alicia").unwrap();
    assert_eq!(get_label(id).unwrap(), Some("alicia".into()));
});
test!(entity_delete, {
    let id = create_entity("alice").unwrap();
    delete_entity(id).unwrap();
    assert_eq!(get_label(id).unwrap(), None);
});
test!(entity_delete_clears_name, {
    let id = create_entity("alice").unwrap();
    delete_entity(id).unwrap();
    assert_eq!(get_id_by_label("alice").unwrap(), None);
});

// ═══════════════════════════════════════════════════════════════════════════
// CAPABILITY HELPERS
// ═══════════════════════════════════════════════════════════════════════════

test!(caps_to_names_single, { assert_eq!(caps_to_names(READ), vec!["read"]); });
test!(caps_to_names_multi, {
    let n = caps_to_names(READ | WRITE | DELETE);
    assert!(n.contains(&"read")); assert!(n.contains(&"write")); assert!(n.contains(&"delete"));
});
test!(caps_to_names_empty, { assert!(caps_to_names(0).is_empty()); });
test!(names_to_caps_single, { assert_eq!(names_to_caps(&["read"]), READ); });
test!(names_to_caps_multi, { assert_eq!(names_to_caps(&["read", "write"]), READ | WRITE); });
test!(names_to_caps_empty, { assert_eq!(names_to_caps(&[]), 0); });
test!(names_to_caps_unknown, { assert_eq!(names_to_caps(&["foo", "bar"]), 0); });
test!(caps_roundtrip, {
    let m = READ | WRITE | DELETE | CREATE | GRANT | EXECUTE;
    let n = caps_to_names(m);
    assert_eq!(names_to_caps(&n), m);
});

// ═══════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════

test!(const_values, {
    assert_eq!(READ, 1); assert_eq!(WRITE, 2); assert_eq!(DELETE, 4);
    assert_eq!(CREATE, 8); assert_eq!(GRANT, 16); assert_eq!(EXECUTE, 32);
    assert_eq!(VIEW, 1 << 62); assert_eq!(ADMIN, 1 << 63);
});
test!(const_no_overlap, {
    let all = READ | WRITE | DELETE | CREATE | GRANT | EXECUTE | VIEW | ADMIN;
    assert_eq!(all.count_ones(), 8);
});

// ═══════════════════════════════════════════════════════════════════════════
// STRESS / SCALE
// ═══════════════════════════════════════════════════════════════════════════

test!(scale_100_users, {
    for u in 0..100 { grant(u, 1000, u+1).unwrap(); }
    for u in 0..100 { assert_eq!(get_mask(u, 1000).unwrap(), u+1); }
});
test!(scale_100_objects, {
    for o in 0..100 { grant(1, o+1000, o+1).unwrap(); }
    for o in 0..100 { assert_eq!(get_mask(1, o+1000).unwrap(), o+1); }
});
test!(scale_batch_1000, {
    let g: Vec<_> = (0..1000).map(|i| (i, 10000+i, 7u64)).collect();
    batch_grant(&g).unwrap();
    assert_eq!(list_for_subject(500).unwrap().len(), 1);
});

// ═══════════════════════════════════════════════════════════════════════════
// STRESS: Table-driven combinations (generates many checks)
// ═══════════════════════════════════════════════════════════════════════════

test!(stress_all_cap_combos, {
    // Test all 2^6 combinations of basic caps
    for mask in 0..64u64 {
        grant(mask, 1000, mask).unwrap();
        for req in 0..64u64 {
            let exp = (mask & req) == req;
            assert_eq!(check(mask, 1000, req).unwrap(), exp, "mask={mask:x} req={req:x}");
        }
    }
});
