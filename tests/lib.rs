use capbit::*;

fn setup() -> (tempfile::TempDir, Capbit, u64, u64) {
    let dir = tempfile::tempdir().unwrap();
    let cb = Capbit::open(dir.path()).unwrap();
    let (sys, root) = cb.bootstrap().unwrap();
    (dir, cb, sys, root)
}

#[test]
fn bootstrap_once() {
    let (_d, cb, sys, root) = setup();
    assert_eq!((sys, root), (_SYSTEM, _ROOT));
    assert!(cb.check(root, sys, OWNER_BITS).unwrap());
    assert_eq!(cb.bootstrap(), Err(Error::Exists));
}

#[test]
fn grant_revoke_check() {
    let (_d, cb, sys, root) = setup();
    cb.grant(root, 10, sys, _VIEWER, Policy::Necessary).unwrap();
    assert_eq!(cb.check_subject(10, sys, _VIEWER).unwrap(), Some(Policy::Necessary));
    assert!(cb.check(10, sys, VIEWER_BITS).unwrap());
    assert!(!cb.check(10, sys, _GRANT).unwrap());
    cb.revoke(root, 10, sys, _VIEWER).unwrap();
    assert_eq!(cb.check_subject(10, sys, _VIEWER).unwrap(), None);
    assert!(!cb.check(10, sys, VIEWER_BITS).unwrap());
}

#[test]
fn undefined_role_grants_nothing() {
    // The v0.5 escalation: undefined roles must resolve to 0, and granting
    // an undefined role must be rejected outright.
    let (_d, cb, sys, root) = setup();
    assert_eq!(cb.grant(root, 10, sys, u64::MAX, Policy::Necessary), Err(Error::NotFound));
}

#[test]
fn deleted_role_cascades_grants() {
    let (_d, cb, sys, root) = setup();
    cb.define_role(root, sys, 100, APP_READ).unwrap();
    cb.grant(root, 10, sys, 100, Policy::Necessary).unwrap();
    assert!(cb.check(10, sys, APP_READ).unwrap());
    cb.delete_role(root, sys, 100).unwrap();
    // No dangling grant, no resurrection with a different mask.
    assert_eq!(cb.check_subject(10, sys, 100).unwrap(), None);
    assert_eq!(cb.get_mask(10, sys).unwrap(), 0);
}

#[test]
fn create_object_breaks_deadlock() {
    let (_d, cb, _sys, root) = setup();
    // v0.5 could never govern a second object. Now: _CREATE_OBJ on _SYSTEM,
    // creator becomes owner atomically and can immediately administer it.
    cb.create_object(root, 50).unwrap();
    assert!(cb.check(root, 50, OWNER_BITS).unwrap());
    cb.define_role(root, 50, 100, APP_READ | APP_WRITE).unwrap();
    cb.grant(root, 10, 50, 100, Policy::Necessary).unwrap();
    assert!(cb.check(10, 50, APP_READ | APP_WRITE).unwrap());
    assert_eq!(cb.create_object(root, 50), Err(Error::Exists));
    // An unprivileged actor cannot create objects.
    assert_eq!(cb.create_object(10, 60), Err(Error::Denied));
}

#[test]
fn deny_overrides() {
    let (_d, cb, sys, root) = setup();
    cb.grant(root, 10, sys, _EDITOR, Policy::Necessary).unwrap();
    assert!(cb.check(10, sys, APP_WRITE).unwrap());
    // Explicit deny of the viewer mask strips read bits even though editor grants them.
    cb.grant(root, 10, sys, _VIEWER, Policy::Not).unwrap();
    let m = cb.get_masks(10, sys).unwrap();
    assert_eq!(m.denied & APP_READ, APP_READ);
    assert!(!cb.check(10, sys, APP_READ).unwrap());
    assert!(cb.check(10, sys, APP_WRITE).unwrap()); // write not covered by the denied mask
}

#[test]
fn possible_vs_necessary_buckets() {
    let (_d, cb, sys, root) = setup();
    cb.grant(root, 10, sys, _EDITOR, Policy::Possible).unwrap();
    cb.grant(root, 10, sys, _VIEWER, Policy::Necessary).unwrap();
    let m = cb.get_masks(10, sys).unwrap();
    assert_eq!(m.necessary & APP_READ, APP_READ);
    assert_eq!(m.possible & APP_WRITE, APP_WRITE);
    assert!(cb.check(10, sys, APP_READ | APP_WRITE).unwrap());
}

#[test]
fn groups_basic() {
    let (_d, cb, _sys, root) = setup();
    cb.create_object(root, 50).unwrap(); // resource
    cb.create_object(root, 90).unwrap(); // group
    cb.add_member(root, 10, 90, Policy::Necessary).unwrap();
    cb.grant(root, 90, 50, _EDITOR, Policy::Necessary).unwrap();
    // 10 gets editor on 50 through the group.
    assert!(cb.check(10, 50, APP_WRITE).unwrap());
    cb.remove_member(root, 10, 90).unwrap();
    assert!(!cb.check(10, 50, APP_WRITE).unwrap());
}

#[test]
fn nested_groups_and_attenuation() {
    let (_d, cb, _sys, root) = setup();
    cb.create_object(root, 50).unwrap();
    cb.create_object(root, 90).unwrap(); // child group
    cb.create_object(root, 91).unwrap(); // parent group
    cb.add_member(root, 10, 90, Policy::Necessary).unwrap();
    // Nest child into parent at Possible strength: membership attenuates.
    cb.add_member(root, 90, 91, Policy::Possible).unwrap();
    cb.grant(root, 91, 50, _EDITOR, Policy::Necessary).unwrap();
    let m = cb.get_masks(10, 50).unwrap();
    assert_eq!(m.possible & APP_WRITE, APP_WRITE); // min(Possible, Necessary) = Possible
    assert_eq!(m.necessary & APP_WRITE, 0);
    // Closure is materialized: groups_of answers without traversal.
    let groups = cb.groups_of(10, 10).unwrap();
    assert!(groups.contains(&(90, Policy::Necessary)));
    assert!(groups.contains(&(91, Policy::Possible)));
}

#[test]
fn deny_membership_excludes() {
    // "All of eng except 10": Not-membership cuts every path through the group.
    let (_d, cb, _sys, root) = setup();
    cb.create_object(root, 50).unwrap();
    cb.create_object(root, 90).unwrap(); // eng
    cb.create_object(root, 91).unwrap(); // eng-all
    cb.add_member(root, 10, 90, Policy::Necessary).unwrap();
    cb.add_member(root, 11, 90, Policy::Necessary).unwrap();
    cb.add_member(root, 90, 91, Policy::Necessary).unwrap();
    cb.grant(root, 91, 50, _EDITOR, Policy::Necessary).unwrap();
    assert!(cb.check(10, 50, APP_WRITE).unwrap());
    // Exclude 10 from eng-all explicitly: positive path through eng is cut.
    cb.add_member(root, 10, 91, Policy::Not).unwrap();
    assert!(!cb.check(10, 50, APP_WRITE).unwrap());
    assert!(cb.check(11, 50, APP_WRITE).unwrap()); // others unaffected
    assert!(cb.groups_of(10, 10).unwrap().contains(&(91, Policy::Not)));
}

#[test]
fn membership_cycles_rejected() {
    let (_d, cb, _sys, root) = setup();
    cb.create_object(root, 90).unwrap();
    cb.create_object(root, 91).unwrap();
    cb.add_member(root, 90, 91, Policy::Necessary).unwrap();
    assert_eq!(cb.add_member(root, 91, 90, Policy::Necessary), Err(Error::Cycle));
    assert_eq!(cb.add_member(root, 90, 90, Policy::Necessary), Err(Error::Cycle));
}

#[test]
fn group_grants_compose_with_role_policy() {
    let (_d, cb, _sys, root) = setup();
    cb.create_object(root, 50).unwrap();
    cb.create_object(root, 90).unwrap();
    cb.add_member(root, 10, 90, Policy::Necessary).unwrap();
    // Group itself is denied the role: members inherit the denial.
    cb.grant(root, 90, 50, _VIEWER, Policy::Not).unwrap();
    cb.grant(root, 10, 50, _EDITOR, Policy::Necessary).unwrap();
    assert!(!cb.check(10, 50, APP_READ).unwrap()); // read denied via group
    assert!(cb.check(10, 50, APP_WRITE).unwrap());
}

#[test]
fn delegation_actually_delegates() {
    // v0.5 inheritance only worked if the child already had the role. Now a
    // subject with no direct grant resolves through the chain.
    let (_d, cb, _sys, root) = setup();
    cb.create_object(root, 50).unwrap();
    cb.grant(root, 20, 50, _EDITOR, Policy::Necessary).unwrap();
    cb.delegate(root, 10, 50, _EDITOR, 20, Policy::Necessary).unwrap();
    assert!(cb.check(10, 50, APP_WRITE).unwrap());
    // Attenuated link: delegation can only weaken.
    cb.delegate(root, 11, 50, _EDITOR, 10, Policy::Possible).unwrap();
    let m = cb.get_masks(11, 50).unwrap();
    assert_eq!(m.possible & APP_WRITE, APP_WRITE);
    assert_eq!(m.necessary & APP_WRITE, 0);
    // Revoking the root of the chain kills everything downstream.
    cb.revoke(root, 20, 50, _EDITOR).unwrap();
    assert!(!cb.check(10, 50, APP_WRITE).unwrap());
    assert!(!cb.check(11, 50, APP_WRITE).unwrap());
}

#[test]
fn delegation_cycles_rejected() {
    let (_d, cb, _sys, root) = setup();
    cb.create_object(root, 50).unwrap();
    cb.delegate(root, 10, 50, _EDITOR, 20, Policy::Necessary).unwrap();
    assert_eq!(cb.delegate(root, 20, 50, _EDITOR, 10, Policy::Necessary), Err(Error::Cycle));
    assert_eq!(cb.delegate(root, 30, 50, _EDITOR, 30, Policy::Necessary), Err(Error::Cycle));
}

#[test]
fn delegation_from_group_granted_parent() {
    let (_d, cb, _sys, root) = setup();
    cb.create_object(root, 50).unwrap();
    cb.create_object(root, 90).unwrap();
    cb.add_member(root, 20, 90, Policy::Necessary).unwrap();
    cb.grant(root, 90, 50, _EDITOR, Policy::Necessary).unwrap();
    cb.delegate(root, 10, 50, _EDITOR, 20, Policy::Necessary).unwrap();
    assert!(cb.check(10, 50, APP_WRITE).unwrap());
}

#[test]
fn type_as_object() {
    let (_d, cb, _sys, root) = setup();
    cb.create_object(root, 70).unwrap(); // type: "document"
    cb.create_object(root, 51).unwrap();
    cb.define_role(root, 70, 100, APP_READ).unwrap();
    cb.set_parent(root, 51, 70).unwrap();
    cb.grant(root, 10, 51, 100, Policy::Necessary).unwrap(); // role defined on type, not instance
    assert!(cb.check(10, 51, APP_READ).unwrap());
    // One change on the type repolicies the instance.
    cb.update_role(root, 70, 100, APP_READ | APP_WRITE).unwrap();
    assert!(cb.check(10, 51, APP_WRITE).unwrap());
    // Instance override takes precedence.
    cb.define_role(root, 51, 100, APP_READ).unwrap();
    assert!(!cb.check(10, 51, APP_WRITE).unwrap());
    // Parent cycles rejected.
    assert_eq!(cb.set_parent(root, 70, 51), Err(Error::Cycle));
}

#[test]
fn self_governance() {
    let (_d, cb, sys, root) = setup();
    // 99 with viewer cannot grant; with admin (holds _GRANT) it can.
    assert_eq!(cb.grant(99, 10, sys, _VIEWER, Policy::Necessary), Err(Error::Denied));
    cb.grant(root, 99, sys, _VIEWER, Policy::Necessary).unwrap();
    assert_eq!(cb.grant(99, 10, sys, _VIEWER, Policy::Necessary), Err(Error::Denied));
    cb.grant(root, 99, sys, _ADMIN, Policy::Necessary).unwrap();
    cb.grant(99, 10, sys, _VIEWER, Policy::Necessary).unwrap();
    assert!(cb.check(10, sys, APP_READ).unwrap());
    // Admin does not hold _DEFINE: role definitions stay owner-only by default.
    assert_eq!(cb.define_role(99, sys, 100, APP_READ), Err(Error::Denied));
}

#[test]
fn population_algebra() {
    let (_d, cb, _sys, root) = setup();
    cb.create_object(root, 90).unwrap();
    cb.create_object(root, 91).unwrap();
    for m in [10, 11, 12] {
        cb.add_member(root, m, 90, Policy::Necessary).unwrap();
    }
    for m in [11, 12, 13] {
        cb.add_member(root, m, 91, Policy::Necessary).unwrap();
    }
    assert_eq!(cb.population_intersect(root, 90, 91).unwrap(), vec![11, 12]);
    assert_eq!(cb.population_subtract(root, 90, 91).unwrap(), vec![10]);
    let members = cb.members_of(root, 90, None, 100).unwrap();
    assert_eq!(members.len(), 3);
    // Pagination resumes after a cursor.
    let page = cb.members_of(root, 90, Some(10), 100).unwrap();
    assert_eq!(page.iter().map(|(m, _)| *m).collect::<Vec<_>>(), vec![11, 12]);
}

#[test]
fn list_subjects_paginated() {
    let (_d, cb, _sys, root) = setup();
    cb.create_object(root, 50).unwrap();
    for s in [10, 11, 12, 13] {
        cb.grant(root, s, 50, _VIEWER, Policy::Necessary).unwrap();
    }
    let p1 = cb.list_subjects(root, 50, None, 3).unwrap();
    assert_eq!(p1.len(), 3); // root's owner grant + first two viewers
    let last = p1.last().unwrap().0;
    let p2 = cb.list_subjects(root, 50, Some(last), 10).unwrap();
    assert_eq!(p1.len() + p2.len(), 5);
}

#[test]
fn delete_object_cascades() {
    let (_d, cb, _sys, root) = setup();
    cb.create_object(root, 50).unwrap();
    cb.create_object(root, 90).unwrap();
    cb.add_member(root, 10, 90, Policy::Necessary).unwrap();
    cb.grant(root, 90, 50, _EDITOR, Policy::Necessary).unwrap();
    assert!(cb.check(10, 50, APP_WRITE).unwrap());
    cb.delete_object(root, 90).unwrap();
    assert!(!cb.check(10, 50, APP_WRITE).unwrap());
    assert!(cb.groups_of(10, 10).unwrap().is_empty());
    assert!(cb.list_grants(10, 10).unwrap().is_empty());
}

#[test]
fn audit_log() {
    let (_d, cb, sys, root) = setup();
    cb.grant(root, 10, sys, _VIEWER, Policy::Necessary).unwrap();
    cb.revoke(root, 10, sys, _VIEWER).unwrap();
    let log = cb.audit_read(root, None, 100).unwrap();
    assert_eq!(log.len(), 3); // bootstrap, grant, revoke
    assert_eq!(log[1].op, op::GRANT);
    assert_eq!(log[1].args, [10, sys, _VIEWER, Policy::Necessary as u64]);
    assert_eq!(log[2].op, op::REVOKE);
    // Unprivileged actors cannot read the log.
    assert_eq!(cb.audit_read(10, None, 100), Err(Error::Denied));
    // Cursor pagination.
    let tail = cb.audit_read(root, Some(log[0].seq), 100).unwrap();
    assert_eq!(tail.len(), 2);
}

#[test]
fn audit_seq_survives_reopen() {
    let dir = tempfile::tempdir().unwrap();
    {
        let cb = Capbit::open(dir.path()).unwrap();
        cb.bootstrap().unwrap();
        cb.grant(_ROOT, 10, _SYSTEM, _VIEWER, Policy::Necessary).unwrap();
    }
    let cb = Capbit::open(dir.path()).unwrap();
    cb.revoke(_ROOT, 10, _SYSTEM, _VIEWER).unwrap();
    let log = cb.audit_read(_ROOT, None, 100).unwrap();
    let seqs: Vec<u64> = log.iter().map(|e| e.seq).collect();
    assert_eq!(seqs, vec![0, 1, 2]); // no reuse, no gaps
}

#[test]
fn meta_and_app_bits_disjoint() {
    assert_eq!(META_BITS & APP_BITS, 0);
    assert_eq!(META_BITS | APP_BITS, u64::MAX);
    assert_eq!(ADMIN_BITS & _DEFINE, 0);
    assert_eq!(app_bit(0) & META_BITS, 0);
}
