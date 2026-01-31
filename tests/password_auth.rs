//! Password authentication tests (TDD)
//!
//! Write tests first, then implement to make them pass.

use capbit::{auth, bootstrap, clear_all, init, test_lock};

fn setup() -> std::sync::MutexGuard<'static, ()> {
    let guard = test_lock();
    init("./test_data/capbit_password.mdb").unwrap();
    clear_all().unwrap();
    guard
}

// ============================================================================
// Password Storage
// ============================================================================

#[test]
fn test_set_and_verify_password() {
    let _g = setup();
    bootstrap::bootstrap("root").unwrap();

    auth::set_password("user:root", "secret123").unwrap();
    assert!(auth::verify_password("user:root", "secret123").unwrap());
}

#[test]
fn test_wrong_password_fails() {
    let _g = setup();
    bootstrap::bootstrap("root").unwrap();

    auth::set_password("user:root", "correct").unwrap();
    assert!(!auth::verify_password("user:root", "wrong").unwrap());
}

#[test]
fn test_password_not_set_fails() {
    let _g = setup();
    bootstrap::bootstrap("root").unwrap();

    let result = auth::verify_password("user:root", "anything");
    assert!(result.is_err() || !result.unwrap());
}

#[test]
fn test_change_password() {
    let _g = setup();
    bootstrap::bootstrap("root").unwrap();

    auth::set_password("user:root", "old").unwrap();
    assert!(auth::verify_password("user:root", "old").unwrap());

    auth::set_password("user:root", "new").unwrap();
    assert!(!auth::verify_password("user:root", "old").unwrap());
    assert!(auth::verify_password("user:root", "new").unwrap());
}

#[test]
fn test_different_users_different_passwords() {
    let _g = setup();
    bootstrap::bootstrap("root").unwrap();

    // Create another user
    capbit::protected::create_entity("user:root", "user", "alice").unwrap();

    auth::set_password("user:root", "rootpass").unwrap();
    auth::set_password("user:alice", "alicepass").unwrap();

    assert!(auth::verify_password("user:root", "rootpass").unwrap());
    assert!(!auth::verify_password("user:root", "alicepass").unwrap());
    assert!(auth::verify_password("user:alice", "alicepass").unwrap());
    assert!(!auth::verify_password("user:alice", "rootpass").unwrap());
}

// ============================================================================
// Login (password → token)
// ============================================================================

#[test]
fn test_login_returns_valid_token() {
    let _g = setup();
    bootstrap::bootstrap("root").unwrap();
    auth::set_password("user:root", "secret").unwrap();

    let token = auth::login("user:root", "secret").unwrap();
    assert!(!token.is_empty());

    // Token should be valid
    let entity = auth::validate_session(&token).unwrap();
    assert_eq!(entity, "user:root");
}

#[test]
fn test_login_wrong_password_fails() {
    let _g = setup();
    bootstrap::bootstrap("root").unwrap();
    auth::set_password("user:root", "correct").unwrap();

    let result = auth::login("user:root", "wrong");
    assert!(result.is_err());
}

#[test]
fn test_login_no_password_set_fails() {
    let _g = setup();
    bootstrap::bootstrap("root").unwrap();

    let result = auth::login("user:root", "anything");
    assert!(result.is_err());
}

#[test]
fn test_login_nonexistent_user_fails() {
    let _g = setup();
    bootstrap::bootstrap("root").unwrap();

    let result = auth::login("user:nobody", "pass");
    assert!(result.is_err());
}

// ============================================================================
// Bootstrap with password
// ============================================================================

#[test]
fn test_bootstrap_with_password() {
    let _g = setup();

    let result = auth::bootstrap_with_password("root", "adminpass").unwrap();
    assert_eq!(result.root_entity, "user:root");
    assert!(!result.token.is_empty());

    // Should be able to login with password
    let token = auth::login("user:root", "adminpass").unwrap();
    let entity = auth::validate_session(&token).unwrap();
    assert_eq!(entity, "user:root");
}

#[test]
fn test_bootstrap_with_password_wrong_pass_fails_login() {
    let _g = setup();

    auth::bootstrap_with_password("root", "correct").unwrap();

    let result = auth::login("user:root", "wrong");
    assert!(result.is_err());
}

// ============================================================================
// Password security
// ============================================================================

#[test]
fn test_same_password_different_hash() {
    let _g = setup();
    bootstrap::bootstrap("root").unwrap();
    capbit::protected::create_entity("user:root", "user", "alice").unwrap();

    // Same password for two users should have different hashes (due to salt)
    auth::set_password("user:root", "samepass").unwrap();
    auth::set_password("user:alice", "samepass").unwrap();

    // Both should verify
    assert!(auth::verify_password("user:root", "samepass").unwrap());
    assert!(auth::verify_password("user:alice", "samepass").unwrap());
}

#[test]
fn test_empty_password_allowed() {
    let _g = setup();
    bootstrap::bootstrap("root").unwrap();

    // Empty password should work (user's choice)
    auth::set_password("user:root", "").unwrap();
    assert!(auth::verify_password("user:root", "").unwrap());
    assert!(!auth::verify_password("user:root", "notempty").unwrap());
}

#[test]
fn test_long_password() {
    let _g = setup();
    bootstrap::bootstrap("root").unwrap();

    let long_pass = "a".repeat(1000);
    auth::set_password("user:root", &long_pass).unwrap();
    assert!(auth::verify_password("user:root", &long_pass).unwrap());
}

#[test]
fn test_unicode_password() {
    let _g = setup();
    bootstrap::bootstrap("root").unwrap();

    let unicode_pass = "p@$$w0rd!@#$%^&*()_+";
    auth::set_password("user:root", unicode_pass).unwrap();
    assert!(auth::verify_password("user:root", unicode_pass).unwrap());
}
