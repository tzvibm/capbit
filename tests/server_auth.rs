//! Server authentication tests (TDD)
//!
//! These tests verify the HTTP-level authentication behavior.
//! Run the server with: ./scripts/server.sh start

// Note: These are integration tests that require a running server.
// For now, they serve as specification of expected behavior.
// In CI, we would use a test harness that starts the server.

#[test]
#[ignore] // Requires running server
fn test_bootstrap_returns_token() {
    // POST /bootstrap {"root_id": "root"}
    // Should return: {"success": true, "data": {"root_entity": "user:root", "token": "..."}}
}

#[test]
#[ignore]
fn test_authenticated_request_succeeds() {
    // POST /entity with Authorization: Bearer <token>
    // Should succeed
}

#[test]
#[ignore]
fn test_unauthenticated_request_fails() {
    // POST /entity without Authorization header
    // Should return 401
}

#[test]
#[ignore]
fn test_invalid_token_returns_401() {
    // POST /entity with Authorization: Bearer invalid-token
    // Should return 401
}

#[test]
#[ignore]
fn test_get_me_returns_current_entity() {
    // GET /me with valid token
    // Should return the entity associated with the token
}

#[test]
#[ignore]
fn test_delete_session_revokes_token() {
    // DELETE /session with valid token
    // Token should no longer work after this
}

#[test]
#[ignore]
fn test_get_sessions_lists_my_sessions() {
    // GET /sessions with valid token
    // Should list all sessions for the authenticated entity
}

#[test]
#[ignore]
fn test_delete_sessions_revokes_all() {
    // DELETE /sessions with valid token
    // All tokens for this entity should stop working
}

#[test]
#[ignore]
fn test_post_session_creates_new_token() {
    // POST /session with valid token
    // Should create a new session and return a new token
}
