//! Authentication tests (TDD - write tests first)

use capbit::{auth, bootstrap, clear_all, init, test_lock};

fn setup() -> std::sync::MutexGuard<'static, ()> {
    let guard = test_lock();
    init("./test_data/capbit_auth.mdb").unwrap();
    clear_all().unwrap();
    guard
}

// ============================================================================
// Token Generation
// ============================================================================

#[test]
fn test_generate_token_is_random() {
    let t1 = auth::generate_token();
    let t2 = auth::generate_token();
    assert_ne!(t1, t2);
    assert!(t1.len() >= 32); // At least 256 bits entropy
}

#[test]
fn test_token_is_url_safe() {
    let token = auth::generate_token();
    // Base64url uses only alphanumeric, -, _
    assert!(token.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
}

// ============================================================================
// Session CRUD
// ============================================================================

#[test]
fn test_create_and_validate_session() {
    let _g = setup();
    bootstrap("root").unwrap();

    let token = auth::create_session("user:root", None).unwrap();
    let entity = auth::validate_session(&token).unwrap();
    assert_eq!(entity, "user:root");
}

#[test]
fn test_invalid_token_fails() {
    let _g = setup();
    bootstrap("root").unwrap();

    let result = auth::validate_session("invalid-token-here");
    assert!(result.is_err());
}

#[test]
fn test_revoke_session() {
    let _g = setup();
    bootstrap("root").unwrap();

    let token = auth::create_session("user:root", None).unwrap();
    assert!(auth::validate_session(&token).is_ok());

    let revoked = auth::revoke_session(&token).unwrap();
    assert!(revoked);

    // Token should no longer work
    assert!(auth::validate_session(&token).is_err());
}

#[test]
fn test_revoke_nonexistent_session() {
    let _g = setup();
    bootstrap("root").unwrap();

    let revoked = auth::revoke_session("nonexistent").unwrap();
    assert!(!revoked);
}

#[test]
fn test_session_with_ttl_expires() {
    let _g = setup();
    bootstrap("root").unwrap();

    // Create session with 0 TTL (already expired)
    let token = auth::create_session("user:root", Some(0)).unwrap();

    // Should fail validation (expired)
    std::thread::sleep(std::time::Duration::from_millis(10));
    let result = auth::validate_session(&token);
    assert!(result.is_err());
}

#[test]
fn test_list_sessions() {
    let _g = setup();
    bootstrap("root").unwrap();

    let t1 = auth::create_session("user:root", None).unwrap();
    let t2 = auth::create_session("user:root", None).unwrap();

    let sessions = auth::list_sessions("user:root").unwrap();
    assert_eq!(sessions.len(), 2);

    // Clean up
    auth::revoke_session(&t1).unwrap();
    auth::revoke_session(&t2).unwrap();
}

#[test]
fn test_revoke_all_sessions() {
    let _g = setup();
    bootstrap("root").unwrap();

    let t1 = auth::create_session("user:root", None).unwrap();
    let t2 = auth::create_session("user:root", None).unwrap();

    let count = auth::revoke_all_sessions("user:root").unwrap();
    assert_eq!(count, 2);

    assert!(auth::validate_session(&t1).is_err());
    assert!(auth::validate_session(&t2).is_err());
}

// ============================================================================
// Bootstrap Integration
// ============================================================================

#[test]
fn test_bootstrap_returns_token() {
    let _g = setup();

    let result = auth::bootstrap_with_token("root").unwrap();
    assert_eq!(result.root_entity, "user:root");
    assert!(!result.token.is_empty());

    // Token should be valid
    let entity = auth::validate_session(&result.token).unwrap();
    assert_eq!(entity, "user:root");
}

#[test]
fn test_bootstrap_token_has_no_expiry() {
    let _g = setup();

    auth::bootstrap_with_token("root").unwrap();

    // Session should have no expiry
    let sessions = auth::list_sessions("user:root").unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].expires_at, 0); // 0 = never expires
}
