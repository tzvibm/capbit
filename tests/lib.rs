use capbit::*;
use std::sync::Mutex;

static LOCK: Mutex<()> = Mutex::new(());

fn setup() -> (u64, u64) {
    let _l = LOCK.lock();
    init("target/test_db").unwrap();
    clear().unwrap();
    bootstrap().unwrap()
}

#[test] fn test_bootstrap() {
    let (sys, root) = setup();
    assert_eq!(sys, _SYSTEM);
    assert_eq!(root, _ROOT);
    assert!(check(root, sys, ALL_BITS).unwrap());
    assert!(bootstrap().is_err());
}

#[test] fn test_grant_revoke() {
    let (sys, root) = setup();
    grant(root, 10, sys, _VIEWER).unwrap();
    assert!(check_subject(10, sys, _VIEWER).unwrap());
    assert!(check(10, sys, VIEWER_BITS).unwrap());
    assert!(!check(10, sys, ALL_BITS).unwrap());
    revoke(root, 10, sys).unwrap();
    assert!(!check_subject(10, sys, _VIEWER).unwrap());
}

#[test] fn test_objects_crud() {
    let (sys, root) = setup();
    create(root, sys, 100, 0xFF).unwrap();
    assert_eq!(get_object(root, sys, 100).unwrap(), Some(0xFF));
    assert!(check_object(root, sys, 100).unwrap());
    update(root, sys, 100, 0xAA).unwrap();
    assert_eq!(get_object(root, sys, 100).unwrap(), Some(0xAA));
    delete(root, sys, 100).unwrap();
    assert!(!check_object(root, sys, 100).unwrap());
}

#[test] fn test_inheritance() {
    let (sys, root) = setup();
    grant(root, 20, sys, _ADMIN).unwrap();
    inherit(root, 10, sys, _ADMIN, 20).unwrap();
    grant(root, 10, sys, _ADMIN).unwrap();
    assert!(check(10, sys, ADMIN_BITS).unwrap());
    assert!(check_inherit(root, 10, sys, _ADMIN).unwrap());
    assert_eq!(get_inherit(root, 10, sys, _ADMIN).unwrap(), Some(20));
    remove_inherit(root, 10, sys, _ADMIN).unwrap();
    assert!(!check_inherit(root, 10, sys, _ADMIN).unwrap());
}

#[test] fn test_permissions() {
    let (sys, root) = setup();
    assert!(grant(99, 10, sys, _VIEWER).is_err());
    grant(root, 99, sys, _VIEWER).unwrap();
    assert!(grant(99, 10, sys, _VIEWER).is_err());
    grant(root, 99, sys, _ADMIN).unwrap();
    grant(99, 10, sys, _VIEWER).unwrap();
    assert!(check(10, sys, VIEWER_BITS).unwrap());
}
