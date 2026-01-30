//! Query operation tests for Capbit v2
//!
//! These tests verify that query functions return complete and correct results,
//! including list_accessible, list_subjects, and relationship queries.

use capbit::{
    init, bootstrap, protected,
    list_accessible, list_subjects, get_relationships,
    get_inheritance, get_inheritors_from_source, get_inheritance_for_object,
    set_relationship, set_capability, set_inheritance, delete_relationship,
    clear_all, test_lock,
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
// list_accessible Tests
// ============================================================================

/// Verify list_accessible returns all accessible resources
#[test]
fn list_accessible_returns_all() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc1").unwrap();
    protected::create_entity("user:root", "resource", "doc2").unwrap();
    protected::create_entity("user:root", "resource", "doc3").unwrap();
    protected::create_entity("user:root", "team", "sales").unwrap();

    // Grant alice access to multiple resources
    set_capability("resource:doc1", "viewer", 0x01).unwrap();
    set_capability("resource:doc2", "editor", 0x03).unwrap();
    set_capability("resource:doc3", "admin", 0x0F).unwrap();
    set_capability("team:sales", "member", 0x01).unwrap();

    set_relationship("user:alice", "viewer", "resource:doc1").unwrap();
    set_relationship("user:alice", "editor", "resource:doc2").unwrap();
    set_relationship("user:alice", "admin", "resource:doc3").unwrap();
    set_relationship("user:alice", "member", "team:sales").unwrap();

    let accessible = list_accessible("user:alice").unwrap();

    assert_eq!(accessible.len(), 4);

    let objects: Vec<&str> = accessible.iter().map(|(obj, _)| obj.as_str()).collect();
    assert!(objects.contains(&"resource:doc1"));
    assert!(objects.contains(&"resource:doc2"));
    assert!(objects.contains(&"resource:doc3"));
    assert!(objects.contains(&"team:sales"));
}

/// Verify list_accessible excludes revoked grants
#[test]
fn list_accessible_excludes_revoked() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc1").unwrap();
    protected::create_entity("user:root", "resource", "doc2").unwrap();

    set_capability("resource:doc1", "viewer", 0x01).unwrap();
    set_capability("resource:doc2", "viewer", 0x01).unwrap();

    set_relationship("user:alice", "viewer", "resource:doc1").unwrap();
    set_relationship("user:alice", "viewer", "resource:doc2").unwrap();

    // Verify both accessible
    let accessible = list_accessible("user:alice").unwrap();
    assert_eq!(accessible.len(), 2);

    // Revoke access to doc2
    delete_relationship("user:alice", "viewer", "resource:doc2").unwrap();

    // Now only doc1 should be accessible
    let accessible = list_accessible("user:alice").unwrap();
    assert_eq!(accessible.len(), 1);
    assert_eq!(accessible[0].0, "resource:doc1");
}

/// Verify list_accessible with multiple relations to same object
#[test]
fn list_accessible_multiple_relations() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    set_capability("resource:doc", "viewer", 0x01).unwrap();
    set_capability("resource:doc", "editor", 0x02).unwrap();
    set_capability("resource:doc", "admin", 0x04).unwrap();

    set_relationship("user:alice", "viewer", "resource:doc").unwrap();
    set_relationship("user:alice", "editor", "resource:doc").unwrap();
    set_relationship("user:alice", "admin", "resource:doc").unwrap();

    let accessible = list_accessible("user:alice").unwrap();

    // Should have 3 entries (one per relation)
    assert_eq!(accessible.len(), 3);

    let relations: Vec<&str> = accessible.iter().map(|(_, rel)| rel.as_str()).collect();
    assert!(relations.contains(&"viewer"));
    assert!(relations.contains(&"editor"));
    assert!(relations.contains(&"admin"));
}

// ============================================================================
// list_subjects Tests
// ============================================================================

/// Verify list_subjects returns all subjects with access
#[test]
fn list_subjects_returns_all() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "user", "charlie").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    set_capability("resource:doc", "viewer", 0x01).unwrap();
    set_capability("resource:doc", "editor", 0x02).unwrap();

    set_relationship("user:alice", "viewer", "resource:doc").unwrap();
    set_relationship("user:bob", "editor", "resource:doc").unwrap();
    set_relationship("user:charlie", "viewer", "resource:doc").unwrap();

    let subjects = list_subjects("resource:doc").unwrap();

    assert_eq!(subjects.len(), 3);

    let users: Vec<&str> = subjects.iter().map(|(subj, _)| subj.as_str()).collect();
    assert!(users.contains(&"user:alice"));
    assert!(users.contains(&"user:bob"));
    assert!(users.contains(&"user:charlie"));
}

/// Verify list_subjects excludes revoked
#[test]
fn list_subjects_excludes_revoked() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    set_capability("resource:doc", "viewer", 0x01).unwrap();

    set_relationship("user:alice", "viewer", "resource:doc").unwrap();
    set_relationship("user:bob", "viewer", "resource:doc").unwrap();

    // Both have access
    let subjects = list_subjects("resource:doc").unwrap();
    assert_eq!(subjects.len(), 2);

    // Revoke bob's access
    delete_relationship("user:bob", "viewer", "resource:doc").unwrap();

    // Only alice should remain
    let subjects = list_subjects("resource:doc").unwrap();
    assert_eq!(subjects.len(), 1);
    assert_eq!(subjects[0].0, "user:alice");
}

// ============================================================================
// Empty Results
// ============================================================================

/// Verify query with no results returns empty
#[test]
fn query_empty_result() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Alice has no grants
    let accessible = list_accessible("user:alice").unwrap();
    assert!(accessible.is_empty());

    // Doc has no subjects
    let subjects = list_subjects("resource:doc").unwrap();
    assert!(subjects.is_empty());

    // No relationships
    let rels = get_relationships("user:alice", "resource:doc").unwrap();
    assert!(rels.is_empty());
}

/// Verify query for non-existent entity returns empty
#[test]
fn query_nonexistent_entity() {
    let _lock = setup_bootstrapped();

    // Query non-existent user
    let accessible = list_accessible("user:nonexistent").unwrap();
    assert!(accessible.is_empty());

    // Query non-existent resource
    let subjects = list_subjects("resource:nonexistent").unwrap();
    assert!(subjects.is_empty());
}

// ============================================================================
// Large Result Sets
// ============================================================================

/// Verify query with many results
#[test]
fn query_large_result_set() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    set_capability("resource:shared", "member", 0x01).unwrap();

    // Create 50 resources and grant alice access to all
    for i in 0..50 {
        protected::create_entity("user:root", "resource", &format!("doc{}", i)).unwrap();
        set_capability(&format!("resource:doc{}", i), "viewer", 0x01).unwrap();
        set_relationship("user:alice", "viewer", &format!("resource:doc{}", i)).unwrap();
    }

    let accessible = list_accessible("user:alice").unwrap();
    assert_eq!(accessible.len(), 50);
}

/// Verify query with many subjects
#[test]
fn query_many_subjects() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "resource", "shared").unwrap();
    set_capability("resource:shared", "member", 0x01).unwrap();

    // Create 50 users and grant all access to shared
    for i in 0..50 {
        protected::create_entity("user:root", "user", &format!("u{}", i)).unwrap();
        set_relationship(&format!("user:u{}", i), "member", "resource:shared").unwrap();
    }

    let subjects = list_subjects("resource:shared").unwrap();
    assert_eq!(subjects.len(), 50);
}

// ============================================================================
// get_relationships Tests
// ============================================================================

/// Verify get_relationships returns all relations between subject and object
#[test]
fn get_relationships_complete() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    set_relationship("user:alice", "viewer", "resource:doc").unwrap();
    set_relationship("user:alice", "commenter", "resource:doc").unwrap();
    set_relationship("user:alice", "editor", "resource:doc").unwrap();

    let rels = get_relationships("user:alice", "resource:doc").unwrap();

    assert_eq!(rels.len(), 3);
    assert!(rels.contains(&"viewer".to_string()));
    assert!(rels.contains(&"commenter".to_string()));
    assert!(rels.contains(&"editor".to_string()));
}

/// Verify get_relationships after modifications
#[test]
fn get_relationships_after_modifications() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Add some relations
    set_relationship("user:alice", "viewer", "resource:doc").unwrap();
    set_relationship("user:alice", "editor", "resource:doc").unwrap();

    let rels = get_relationships("user:alice", "resource:doc").unwrap();
    assert_eq!(rels.len(), 2);

    // Delete one
    delete_relationship("user:alice", "viewer", "resource:doc").unwrap();

    let rels = get_relationships("user:alice", "resource:doc").unwrap();
    assert_eq!(rels.len(), 1);
    assert!(rels.contains(&"editor".to_string()));
    assert!(!rels.contains(&"viewer".to_string()));

    // Add a new one
    set_relationship("user:alice", "admin", "resource:doc").unwrap();

    let rels = get_relationships("user:alice", "resource:doc").unwrap();
    assert_eq!(rels.len(), 2);
    assert!(rels.contains(&"editor".to_string()));
    assert!(rels.contains(&"admin".to_string()));
}

// ============================================================================
// Inheritance Query Tests
// ============================================================================

/// Verify get_inheritance returns all sources
#[test]
fn get_inheritance_complete() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "user", "charlie").unwrap();
    protected::create_entity("user:root", "user", "collector").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Collector inherits from multiple sources
    set_inheritance("user:collector", "resource:doc", "user:alice").unwrap();
    set_inheritance("user:collector", "resource:doc", "user:bob").unwrap();
    set_inheritance("user:collector", "resource:doc", "user:charlie").unwrap();

    let sources = get_inheritance("user:collector", "resource:doc").unwrap();

    assert_eq!(sources.len(), 3);
    assert!(sources.contains(&"user:alice".to_string()));
    assert!(sources.contains(&"user:bob".to_string()));
    assert!(sources.contains(&"user:charlie".to_string()));
}

/// Verify get_inheritors_from_source returns all inheritors
#[test]
fn get_inheritors_complete() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "source").unwrap();
    protected::create_entity("user:root", "user", "a").unwrap();
    protected::create_entity("user:root", "user", "b").unwrap();
    protected::create_entity("user:root", "user", "c").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Multiple users inherit from source
    set_inheritance("user:a", "resource:doc", "user:source").unwrap();
    set_inheritance("user:b", "resource:doc", "user:source").unwrap();
    set_inheritance("user:c", "resource:doc", "user:source").unwrap();

    let inheritors = get_inheritors_from_source("user:source", "resource:doc").unwrap();

    assert_eq!(inheritors.len(), 3);
    assert!(inheritors.contains(&"user:a".to_string()));
    assert!(inheritors.contains(&"user:b".to_string()));
    assert!(inheritors.contains(&"user:c".to_string()));
}

/// Verify get_inheritance_for_object returns all inheritance pairs
#[test]
fn get_inheritance_for_object_complete() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "user", "charlie").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Various inheritance relationships
    set_inheritance("user:bob", "resource:doc", "user:alice").unwrap();
    set_inheritance("user:charlie", "resource:doc", "user:alice").unwrap();
    set_inheritance("user:charlie", "resource:doc", "user:bob").unwrap();

    let inheritance = get_inheritance_for_object("resource:doc").unwrap();

    assert_eq!(inheritance.len(), 3);

    // Should contain (source, subject) pairs
    assert!(inheritance.contains(&("user:alice".to_string(), "user:bob".to_string())));
    assert!(inheritance.contains(&("user:alice".to_string(), "user:charlie".to_string())));
    assert!(inheritance.contains(&("user:bob".to_string(), "user:charlie".to_string())));
}

// ============================================================================
// Query After Modifications
// ============================================================================

/// Verify queries reflect modifications
#[test]
fn query_after_modifications() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc1").unwrap();
    protected::create_entity("user:root", "resource", "doc2").unwrap();
    protected::create_entity("user:root", "resource", "doc3").unwrap();

    set_capability("resource:doc1", "viewer", 0x01).unwrap();
    set_capability("resource:doc2", "viewer", 0x01).unwrap();
    set_capability("resource:doc3", "viewer", 0x01).unwrap();

    // Initial state: alice has access to doc1 and doc2
    set_relationship("user:alice", "viewer", "resource:doc1").unwrap();
    set_relationship("user:alice", "viewer", "resource:doc2").unwrap();

    let accessible = list_accessible("user:alice").unwrap();
    assert_eq!(accessible.len(), 2);

    // Modification 1: add doc3
    set_relationship("user:alice", "viewer", "resource:doc3").unwrap();
    let accessible = list_accessible("user:alice").unwrap();
    assert_eq!(accessible.len(), 3);

    // Modification 2: remove doc1
    delete_relationship("user:alice", "viewer", "resource:doc1").unwrap();
    let accessible = list_accessible("user:alice").unwrap();
    assert_eq!(accessible.len(), 2);

    let objects: Vec<&str> = accessible.iter().map(|(obj, _)| obj.as_str()).collect();
    assert!(!objects.contains(&"resource:doc1"));
    assert!(objects.contains(&"resource:doc2"));
    assert!(objects.contains(&"resource:doc3"));
}
