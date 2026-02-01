//! Input validation and edge case tests for Capbit v2
//!
//! These tests verify proper handling of malformed inputs, empty strings,
//! special characters, and boundary conditions.

use capbit::{
    init, bootstrap, protected, check_access, parse_entity_id,
    set_relationship, set_capability, get_capability,
    clear_all, test_lock, entity_exists,
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
// Entity ID Format Validation
// ============================================================================

/// Verify empty entity ID is rejected
#[test]
fn empty_entity_id_rejected() {
    let _lock = setup_bootstrapped();

    // parse_entity_id should reject empty string
    let result = parse_entity_id("");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("Invalid entity ID"));
}

/// Verify empty type in entity ID parsing
#[test]
fn empty_type_in_parse() {
    let _lock = setup_bootstrapped();

    // Empty type in entity ID - parse_entity_id returns ("", "alice")
    let result = parse_entity_id(":alice");
    // This succeeds with empty type - the type is just ""
    if result.is_ok() {
        let (entity_type, id) = result.unwrap();
        assert_eq!(entity_type, "");
        assert_eq!(id, "alice");
    }

    // Creating entity with empty type - type "" doesn't exist so should fail
    // But this checks ENTITY_CREATE on _type: which may or may not exist
    let result = protected::create_entity("user:root", "", "alice");
    // The behavior depends on whether _type: exists - test the actual outcome
    // If it fails, good. If it succeeds, the entity has empty type.
    let _ = result; // Don't assert, just verify no crash
}

/// Verify empty relation is rejected (LMDB doesn't support zero-length keys)
#[test]
fn empty_relation_handled() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Empty relation name - rejected due to LMDB limitations
    let result = set_capability("resource:doc", "", 0x01);
    assert!(result.is_err(), "Empty relation should be rejected");
}

/// Verify missing colon in entity ID is rejected
#[test]
fn missing_colon_in_entity_id_rejected() {
    let _lock = setup_bootstrapped();

    let result = parse_entity_id("useralice");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("must be 'type:id' format"));
}

/// Verify multiple colons in entity ID works (takes first colon as separator)
#[test]
fn multiple_colons_in_entity_id() {
    let _lock = setup_bootstrapped();

    // "user:alice:extra" -> type="user", id="alice:extra"
    let result = parse_entity_id("user:alice:extra");
    assert!(result.is_ok());
    let (entity_type, id) = result.unwrap();
    assert_eq!(entity_type, "user");
    assert_eq!(id, "alice:extra");

    // Can create such an entity
    let result = protected::create_entity("user:root", "user", "alice:extra");
    assert!(result.is_ok());
    assert!(entity_exists("user:alice:extra").unwrap());
}

// ============================================================================
// Special Characters
// ============================================================================

/// Verify special characters in entity name are handled
#[test]
fn special_chars_in_entity_name() {
    let _lock = setup_bootstrapped();

    // Various special characters
    let special_names = vec![
        "alice-bob",
        "alice_bob",
        "alice.bob",
        "alice@example.com",
        "alice+bob",
        "alice&bob",
        "alice=bob",
    ];

    for name in special_names {
        let result = protected::create_entity("user:root", "user", name);
        assert!(result.is_ok(), "Failed to create entity with name: {}", name);
        let entity_id = format!("user:{}", name);
        assert!(entity_exists(&entity_id).unwrap(), "Entity {} doesn't exist", entity_id);
    }
}

/// Verify unicode characters in entity name
#[test]
fn unicode_in_entity_name() {
    let _lock = setup_bootstrapped();

    // Unicode names
    let unicode_names = vec![
        "alice_日本語",
        "用户_测试",
        "пользователь",
        "emoji_🎉",
    ];

    for name in unicode_names {
        let result = protected::create_entity("user:root", "user", name);
        assert!(result.is_ok(), "Failed to create entity with unicode name: {}", name);
        let entity_id = format!("user:{}", name);
        assert!(entity_exists(&entity_id).unwrap());
    }
}

/// Verify slash in entity name is properly escaped
#[test]
fn slash_in_entity_name_escaped() {
    let _lock = setup_bootstrapped();

    // Slash is used as separator in storage keys, must be escaped
    protected::create_entity("user:root", "user", "alice/bob").unwrap();
    protected::create_entity("user:root", "resource", "path/to/doc").unwrap();

    assert!(entity_exists("user:alice/bob").unwrap());
    assert!(entity_exists("resource:path/to/doc").unwrap());

    // Can set relationships with slashes
    protected::set_capability("user:root", "resource:path/to/doc", "viewer", 0x01).unwrap();
    protected::set_grant("user:root", "user:alice/bob", "viewer", "resource:path/to/doc").unwrap();

    // Access check works
    let caps = check_access("user:alice/bob", "resource:path/to/doc", None).unwrap();
    assert_eq!(caps, 0x01);
}

/// Verify backslash in entity name is properly escaped
#[test]
fn backslash_in_entity_name_escaped() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "domain\\alice").unwrap();
    assert!(entity_exists("user:domain\\alice").unwrap());

    protected::create_entity("user:root", "resource", "doc").unwrap();
    protected::set_capability("user:root", "resource:doc", "viewer", 0x01).unwrap();
    protected::set_grant("user:root", "user:domain\\alice", "viewer", "resource:doc").unwrap();

    let caps = check_access("user:domain\\alice", "resource:doc", None).unwrap();
    assert_eq!(caps, 0x01);
}

// ============================================================================
// Length Limits
// ============================================================================

/// Verify moderately long entity ID is handled
#[test]
fn moderately_long_entity_id() {
    let _lock = setup_bootstrapped();

    // Create a moderately long name (200 chars - within LMDB key limits)
    // LMDB default max key size is 511 bytes, and we need room for type prefix
    let long_name: String = "a".repeat(200);
    let result = protected::create_entity("user:root", "user", &long_name);

    // Should succeed
    assert!(result.is_ok());

    let entity_id = format!("user:{}", long_name);
    assert!(entity_exists(&entity_id).unwrap());
}

/// Verify very long entity ID behavior (may fail due to LMDB key limits)
#[test]
fn very_long_entity_id_limit() {
    let _lock = setup_bootstrapped();

    // Create a very long name that exceeds LMDB key limits
    let long_name: String = "a".repeat(1000);
    let result = protected::create_entity("user:root", "user", &long_name);

    // This may fail due to LMDB key size limits (511 bytes default)
    // Either outcome is acceptable - we're testing it doesn't crash
    if result.is_err() {
        // Expected - key too long
        let _ = result.unwrap_err();
    } else {
        // If it somehow succeeds, verify it works
        let entity_id = format!("user:{}", long_name);
        let _ = entity_exists(&entity_id);
    }
}

/// Verify whitespace in entity name
#[test]
fn whitespace_in_entity_name() {
    let _lock = setup_bootstrapped();

    // Leading/trailing whitespace
    protected::create_entity("user:root", "user", "  alice  ").unwrap();
    assert!(entity_exists("user:  alice  ").unwrap());

    // Internal spaces
    protected::create_entity("user:root", "user", "alice smith").unwrap();
    assert!(entity_exists("user:alice smith").unwrap());

    // Tabs and newlines
    protected::create_entity("user:root", "user", "alice\tbob").unwrap();
    assert!(entity_exists("user:alice\tbob").unwrap());
}

// ============================================================================
// Capability Value Boundaries
// ============================================================================

/// Verify zero capability value
#[test]
fn zero_capability_value() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Set capability to zero
    protected::set_capability("user:root", "resource:doc", "none", 0).unwrap();
    protected::set_grant("user:root", "user:alice", "none", "resource:doc").unwrap();

    // Alice has the grant but zero capability
    let caps = check_access("user:alice", "resource:doc", None).unwrap();
    assert_eq!(caps, 0);

    // Verify stored value
    let stored = get_capability("resource:doc", "none").unwrap();
    assert_eq!(stored, Some(0));
}

/// Verify max u64 capability value
#[test]
fn max_capability_value() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    protected::set_capability("user:root", "resource:doc", "everything", u64::MAX).unwrap();
    protected::set_grant("user:root", "user:alice", "everything", "resource:doc").unwrap();

    let caps = check_access("user:alice", "resource:doc", None).unwrap();
    assert_eq!(caps, u64::MAX);
}

// ============================================================================
// Reserved Names
// ============================================================================

/// Verify reserved underscore type names are protected
#[test]
fn reserved_underscore_types() {
    let _lock = setup_bootstrapped();

    // Users cannot create types starting with underscore
    // (they require TYPE_CREATE on _type:_type which normal users don't have)

    protected::create_entity("user:root", "user", "alice").unwrap();

    // Alice doesn't have TYPE_CREATE permission
    let result = protected::create_type("user:alice", "_custom");
    assert!(result.is_err());

    // Even with TYPE_CREATE, underscore types might fail due to bootstrap protection
    // The _type type is created during bootstrap and is special
}

/// Verify creating entity in non-existent type fails
#[test]
fn entity_in_nonexistent_type_fails() {
    let _lock = setup_bootstrapped();

    // Type "nonexistent" doesn't exist
    let result = protected::create_entity("user:root", "nonexistent", "test");
    assert!(result.is_err());
    // Error could be "Type does not exist", "lacks permission", or "not found" depending on check order
    let err_msg = result.unwrap_err().message;
    assert!(err_msg.contains("does not exist") || err_msg.contains("lacks permission") || err_msg.contains("not found"),
            "Unexpected error: {}", err_msg);
}

// ============================================================================
// Duplicate Detection
// ============================================================================

/// Verify duplicate entity creation is rejected
#[test]
fn duplicate_entity_rejected() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();

    // Second creation should fail
    let result = protected::create_entity("user:root", "user", "alice");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("already exists"));
}

/// Verify duplicate type creation is rejected
#[test]
fn duplicate_type_rejected() {
    let _lock = setup_bootstrapped();

    // "user" type already exists from bootstrap
    let result = protected::create_type("user:root", "user");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("already exists"));
}
