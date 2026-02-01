//! Tests for protected read functions
//!
//! Verifies that list_* functions properly filter based on permissions.

use capbit::{
    bootstrap, clear_all, init, protected, test_lock, SystemCap,
    set_capability, set_relationship, set_cap_label,
};

fn setup() -> std::sync::MutexGuard<'static, ()> {
    let guard = test_lock();
    init("./test_data/capbit_reads.mdb").unwrap();
    clear_all().unwrap();
    guard
}

// ============================================================================
// list_entities tests
// ============================================================================

#[test]
fn test_list_entities_root_sees_all() {
    let _g = setup();
    bootstrap("root").unwrap();

    // Root has SYSTEM_READ, should see all entities including _type:*
    let entities = protected::list_entities("user:root").unwrap();
    assert!(entities.contains(&"user:root".to_string()));
    assert!(entities.contains(&"_type:user".to_string()));
    assert!(entities.contains(&"_type:_type".to_string()));
}

#[test]
fn test_list_entities_user_without_system_read_no_type_entities() {
    let _g = setup();
    bootstrap("root").unwrap();

    // Create alice with no special permissions
    protected::create_entity("user:root", "user", "alice").unwrap();

    // Alice should not see _type:* entities (no SYSTEM_READ)
    let entities = protected::list_entities("user:alice").unwrap();
    assert!(!entities.iter().any(|e| e.starts_with("_type:")));
}

#[test]
fn test_list_entities_user_sees_entities_with_access() {
    let _g = setup();
    bootstrap("root").unwrap();

    // Create alice and a resource
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc1").unwrap();

    // Give alice access to doc1
    set_capability("resource:doc1", "viewer", 0x01).unwrap();
    set_relationship("user:alice", "viewer", "resource:doc1").unwrap();

    // Alice should see doc1
    let entities = protected::list_entities("user:alice").unwrap();
    assert!(entities.contains(&"resource:doc1".to_string()));
}

#[test]
fn test_list_entities_user_no_access_sees_nothing() {
    let _g = setup();
    bootstrap("root").unwrap();

    // Create alice and bob with a resource
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "resource", "secret").unwrap();

    // Only bob has access
    set_capability("resource:secret", "owner", 0xFF).unwrap();
    set_relationship("user:bob", "owner", "resource:secret").unwrap();

    // Alice should not see secret
    let entities = protected::list_entities("user:alice").unwrap();
    assert!(!entities.contains(&"resource:secret".to_string()));
}

// ============================================================================
// list_grants tests
// ============================================================================

#[test]
fn test_list_grants_requires_grant_read() {
    let _g = setup();
    bootstrap("root").unwrap();

    // Create entities and a grant
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc1").unwrap();
    set_capability("resource:doc1", "viewer", 0x01).unwrap();
    protected::set_grant("user:root", "user:alice", "viewer", "resource:doc1").unwrap();

    // Root has GRANT_READ on _type:resource, should see grant
    let grants = protected::list_grants("user:root").unwrap();
    assert!(grants.iter().any(|(s, r, sc)| s == "user:alice" && r == "viewer" && sc == "resource:doc1"));
}

#[test]
fn test_list_grants_user_without_grant_read_sees_nothing() {
    let _g = setup();
    bootstrap("root").unwrap();

    // Create alice and bob
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "resource", "doc1").unwrap();

    // Create a grant for alice
    set_capability("resource:doc1", "viewer", 0x01).unwrap();
    protected::set_grant("user:root", "user:alice", "viewer", "resource:doc1").unwrap();

    // Bob has no GRANT_READ on resource:doc1
    let grants = protected::list_grants("user:bob").unwrap();
    assert!(!grants.iter().any(|(_, _, sc)| sc == "resource:doc1"));
}

#[test]
fn test_list_grants_user_with_grant_read_sees_scoped() {
    let _g = setup();
    bootstrap("root").unwrap();

    // Create entities
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc1").unwrap();
    protected::create_entity("user:root", "resource", "doc2").unwrap();

    // Set up capabilities
    set_capability("resource:doc1", "viewer", 0x01).unwrap();
    set_capability("resource:doc1", "admin", SystemCap::GRANT_READ).unwrap();
    set_capability("resource:doc2", "viewer", 0x01).unwrap();

    // Alice is admin on doc1 (has GRANT_READ), viewer on doc2 (no GRANT_READ)
    protected::set_grant("user:root", "user:alice", "admin", "resource:doc1").unwrap();
    protected::set_grant("user:root", "user:alice", "viewer", "resource:doc2").unwrap();

    // Alice should see grants on doc1 but not doc2
    let grants = protected::list_grants("user:alice").unwrap();
    assert!(grants.iter().any(|(_, _, sc)| sc == "resource:doc1"));
    assert!(!grants.iter().any(|(_, _, sc)| sc == "resource:doc2"));
}

// ============================================================================
// list_capabilities tests
// ============================================================================

#[test]
fn test_list_capabilities_requires_cap_read() {
    let _g = setup();
    bootstrap("root").unwrap();

    // Root has CAP_READ on all _type:* scopes
    let caps = protected::list_capabilities("user:root").unwrap();
    // Should see admin capability on _type:user (created at bootstrap)
    assert!(caps.iter().any(|(sc, rel, _)| sc == "_type:user" && rel == "admin"));
}

#[test]
fn test_list_capabilities_user_without_cap_read() {
    let _g = setup();
    bootstrap("root").unwrap();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc1").unwrap();

    // Define capability on doc1
    set_capability("resource:doc1", "viewer", 0x01).unwrap();

    // Alice has no CAP_READ, should not see doc1 capabilities
    let caps = protected::list_capabilities("user:alice").unwrap();
    assert!(!caps.iter().any(|(sc, _, _)| sc == "resource:doc1"));
}

#[test]
fn test_list_capabilities_user_with_cap_read_sees_scoped() {
    let _g = setup();
    bootstrap("root").unwrap();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc1").unwrap();

    // Give alice CAP_READ on doc1
    set_capability("resource:doc1", "cap_reader", SystemCap::CAP_READ).unwrap();
    set_capability("resource:doc1", "viewer", 0x01).unwrap();
    protected::set_grant("user:root", "user:alice", "cap_reader", "resource:doc1").unwrap();

    // Alice should see capabilities on doc1
    let caps = protected::list_capabilities("user:alice").unwrap();
    assert!(caps.iter().any(|(sc, rel, _)| sc == "resource:doc1" && rel == "viewer"));
}

// ============================================================================
// list_cap_labels tests
// ============================================================================

#[test]
fn test_list_cap_labels_requires_cap_read() {
    let _g = setup();
    bootstrap("root").unwrap();

    // Bootstrap creates labels on _type:_type
    let labels = protected::list_cap_labels("user:root").unwrap();
    // Root should see system cap labels
    assert!(labels.iter().any(|(sc, _, l)| sc == "_type:_type" && l == "type-create"));
}

#[test]
fn test_list_cap_labels_user_without_cap_read() {
    let _g = setup();
    bootstrap("root").unwrap();

    protected::create_entity("user:root", "user", "alice").unwrap();

    // Define a cap label on _type:resource
    set_cap_label("_type:resource", 0, "read").unwrap();

    // Alice has no CAP_READ on _type:resource
    let labels = protected::list_cap_labels("user:alice").unwrap();
    assert!(!labels.iter().any(|(sc, _, _)| sc == "_type:resource"));
}

#[test]
fn test_list_cap_labels_user_with_cap_read() {
    let _g = setup();
    bootstrap("root").unwrap();

    protected::create_entity("user:root", "user", "alice").unwrap();

    // Give alice CAP_READ on _type:resource via a grant
    set_capability("_type:resource", "cap_reader", SystemCap::CAP_READ).unwrap();
    protected::set_grant("user:root", "user:alice", "cap_reader", "_type:resource").unwrap();

    // Define a cap label
    set_cap_label("_type:resource", 0, "read").unwrap();

    // Alice should see it
    let labels = protected::list_cap_labels("user:alice").unwrap();
    assert!(labels.iter().any(|(sc, bit, l)| sc == "_type:resource" && *bit == 0 && l == "read"));
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn test_nonexistent_user_sees_nothing() {
    let _g = setup();
    bootstrap("root").unwrap();

    // User that doesn't exist - in normalized schema, resolving unknown entity fails
    let result = protected::list_entities("user:nobody");
    assert!(result.is_err() || result.unwrap().is_empty());

    let result = protected::list_grants("user:nobody");
    assert!(result.is_err() || result.unwrap().is_empty());

    let result = protected::list_capabilities("user:nobody");
    assert!(result.is_err() || result.unwrap().is_empty());

    let result = protected::list_cap_labels("user:nobody");
    assert!(result.is_err() || result.unwrap().is_empty());
}

#[test]
fn test_empty_database_returns_empty() {
    let _g = setup();
    // Don't bootstrap - empty database
    // In normalized schema, querying without bootstrap fails because types don't exist
    let result = protected::list_entities("user:anyone");
    assert!(result.is_err() || result.unwrap().is_empty());
}

#[test]
fn test_list_entities_inheritance_grants_visibility() {
    let _g = setup();
    bootstrap("root").unwrap();

    // Create alice, bob, and a resource
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "resource", "doc1").unwrap();

    // Alice has access to doc1
    set_capability("resource:doc1", "owner", 0xFF).unwrap();
    protected::set_grant("user:root", "user:alice", "owner", "resource:doc1").unwrap();

    // Bob inherits from alice on doc1
    protected::set_delegation("user:root", "user:bob", "resource:doc1", "user:alice").unwrap();

    // Bob should see doc1 via inheritance
    let entities = protected::list_entities("user:bob").unwrap();
    assert!(entities.contains(&"resource:doc1".to_string()));
}
