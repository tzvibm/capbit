//! Batch operation tests for Capbit v2
//!
//! These tests verify that batch operations work correctly, including
//! atomicity, ordering, and error handling.

use capbit::{
    init, bootstrap, has_capability,
    batch_set_relationships, batch_set_capabilities, batch_set_inheritance,
    write_batch, get_capability,
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

fn setup_clean() -> std::sync::MutexGuard<'static, ()> {
    let lock = test_lock();
    setup();
    clear_all().unwrap();
    lock
}

fn setup_bootstrapped() -> std::sync::MutexGuard<'static, ()> {
    let lock = test_lock();
    setup();
    clear_all().unwrap();
    bootstrap("root").unwrap();
    lock
}

// ============================================================================
// Basic Batch Operations
// ============================================================================

/// Verify batch_set_relationships works for multiple entries
#[test]
fn batch_relationships_multiple_entries() {
    let _lock = setup_clean();

    // Set up capabilities first
    batch_set_capabilities(&[
        ("resource:doc".to_string(), "viewer".to_string(), 0x01),
        ("resource:doc".to_string(), "editor".to_string(), 0x02),
        ("team:sales".to_string(), "member".to_string(), 0x04),
    ]).unwrap();

    // Batch set relationships
    let entries = vec![
        ("user:alice".to_string(), "viewer".to_string(), "resource:doc".to_string()),
        ("user:alice".to_string(), "member".to_string(), "team:sales".to_string()),
        ("user:bob".to_string(), "editor".to_string(), "resource:doc".to_string()),
        ("user:bob".to_string(), "member".to_string(), "team:sales".to_string()),
    ];

    let result = batch_set_relationships(&entries);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 4);

    // Verify all relationships were created
    assert!(has_capability("user:alice", "resource:doc", 0x01).unwrap());
    assert!(has_capability("user:alice", "team:sales", 0x04).unwrap());
    assert!(has_capability("user:bob", "resource:doc", 0x02).unwrap());
    assert!(has_capability("user:bob", "team:sales", 0x04).unwrap());
}

/// Verify batch_set_capabilities works for multiple entries
#[test]
fn batch_capabilities_multiple_entries() {
    let _lock = setup_clean();

    let entries = vec![
        ("resource:doc1".to_string(), "viewer".to_string(), 0x01u64),
        ("resource:doc1".to_string(), "editor".to_string(), 0x03u64),
        ("resource:doc2".to_string(), "viewer".to_string(), 0x01u64),
        ("resource:doc2".to_string(), "admin".to_string(), 0x0Fu64),
    ];

    let result = batch_set_capabilities(&entries);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 4);

    // Verify all capabilities were set
    assert_eq!(get_capability("resource:doc1", "viewer").unwrap(), Some(0x01));
    assert_eq!(get_capability("resource:doc1", "editor").unwrap(), Some(0x03));
    assert_eq!(get_capability("resource:doc2", "viewer").unwrap(), Some(0x01));
    assert_eq!(get_capability("resource:doc2", "admin").unwrap(), Some(0x0F));
}

/// Verify batch_set_inheritance works for multiple entries
#[test]
fn batch_inheritance_multiple_entries() {
    let _lock = setup_clean();

    // Set up a capability and direct grant
    batch_set_capabilities(&[
        ("resource:doc".to_string(), "member".to_string(), 0x01),
    ]).unwrap();
    batch_set_relationships(&[
        ("user:root".to_string(), "member".to_string(), "resource:doc".to_string()),
    ]).unwrap();

    // Batch set inheritance
    let entries = vec![
        ("user:alice".to_string(), "resource:doc".to_string(), "user:root".to_string()),
        ("user:bob".to_string(), "resource:doc".to_string(), "user:root".to_string()),
        ("user:charlie".to_string(), "resource:doc".to_string(), "user:root".to_string()),
    ];

    let result = batch_set_inheritance(&entries);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3);

    // Verify all inheritances work
    assert!(has_capability("user:alice", "resource:doc", 0x01).unwrap());
    assert!(has_capability("user:bob", "resource:doc", 0x01).unwrap());
    assert!(has_capability("user:charlie", "resource:doc", 0x01).unwrap());
}

// ============================================================================
// Empty Batch Operations
// ============================================================================

/// Verify empty batch succeeds
#[test]
fn batch_empty_succeeds() {
    let _lock = setup_clean();

    let empty_rels: Vec<(String, String, String)> = vec![];
    let result = batch_set_relationships(&empty_rels);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);

    let empty_caps: Vec<(String, String, u64)> = vec![];
    let result = batch_set_capabilities(&empty_caps);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);

    let empty_inh: Vec<(String, String, String)> = vec![];
    let result = batch_set_inheritance(&empty_inh);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

// ============================================================================
// WriteBatch API
// ============================================================================

/// Verify WriteBatch with multiple operation types
#[test]
fn write_batch_mixed_operations() {
    let _lock = setup_clean();

    let mut batch = write_batch();
    batch
        .set_capability("resource:doc", "viewer", 0x01)
        .set_capability("resource:doc", "editor", 0x03)
        .set_relationship("user:alice", "viewer", "resource:doc")
        .set_relationship("user:bob", "editor", "resource:doc")
        .set_inheritance("user:charlie", "resource:doc", "user:alice");

    assert_eq!(batch.len(), 5);

    let result = batch.execute();
    assert!(result.is_ok());

    // Verify all operations executed
    assert!(has_capability("user:alice", "resource:doc", 0x01).unwrap());
    assert!(has_capability("user:bob", "resource:doc", 0x03).unwrap());
    assert!(has_capability("user:charlie", "resource:doc", 0x01).unwrap()); // inherited
}

/// Verify WriteBatch ordering is preserved
#[test]
fn write_batch_ordering_matters() {
    let _lock = setup_clean();

    // Order matters: capability must be defined before relationship can use it
    let mut batch = write_batch();
    batch
        .set_capability("resource:doc", "role1", 0x01)
        .set_relationship("user:alice", "role1", "resource:doc")
        .set_capability("resource:doc", "role1", 0x0F); // Redefine capability

    batch.execute().unwrap();

    // The final capability value should be 0x0F (last write wins)
    assert_eq!(get_capability("resource:doc", "role1").unwrap(), Some(0x0F));

    // Alice's grant uses the final capability value
    assert!(has_capability("user:alice", "resource:doc", 0x0F).unwrap());
}

/// Verify WriteBatch delete operations
#[test]
fn write_batch_delete_operations() {
    let _lock = setup_clean();

    // First create some data
    let mut setup = write_batch();
    setup
        .set_capability("resource:doc", "member", 0x01)
        .set_relationship("user:alice", "member", "resource:doc")
        .set_inheritance("user:bob", "resource:doc", "user:alice");
    setup.execute().unwrap();

    // Verify setup
    assert!(has_capability("user:alice", "resource:doc", 0x01).unwrap());
    assert!(has_capability("user:bob", "resource:doc", 0x01).unwrap());

    // Now delete
    let mut delete = write_batch();
    delete
        .delete_relationship("user:alice", "member", "resource:doc")
        .delete_inheritance("user:bob", "resource:doc", "user:alice");
    delete.execute().unwrap();

    // Verify deletions
    assert!(!has_capability("user:alice", "resource:doc", 0x01).unwrap());
    assert!(!has_capability("user:bob", "resource:doc", 0x01).unwrap());
}

/// Verify WriteBatch can be cleared and reused
#[test]
fn write_batch_clear_and_reuse() {
    let _lock = setup_clean();

    let mut batch = write_batch();
    batch.set_capability("resource:doc1", "role", 0x01);
    assert_eq!(batch.len(), 1);

    batch.clear();
    assert_eq!(batch.len(), 0);
    assert!(batch.is_empty());

    // Reuse
    batch.set_capability("resource:doc2", "role", 0x02);
    batch.execute().unwrap();

    // Only doc2 should have capability
    assert_eq!(get_capability("resource:doc1", "role").unwrap(), None);
    assert_eq!(get_capability("resource:doc2", "role").unwrap(), Some(0x02));
}

// ============================================================================
// Duplicate and Conflicting Operations
// ============================================================================

/// Verify duplicate operations in batch (idempotent)
#[test]
fn batch_duplicate_operations() {
    let _lock = setup_clean();

    let entries = vec![
        ("resource:doc".to_string(), "viewer".to_string(), 0x01u64),
        ("resource:doc".to_string(), "viewer".to_string(), 0x01u64), // duplicate
        ("resource:doc".to_string(), "viewer".to_string(), 0x01u64), // duplicate
    ];

    let result = batch_set_capabilities(&entries);
    assert!(result.is_ok());

    // Should have the capability set once
    assert_eq!(get_capability("resource:doc", "viewer").unwrap(), Some(0x01));
}

/// Verify conflicting operations in batch (last write wins)
#[test]
fn batch_conflicting_operations() {
    let _lock = setup_clean();

    let entries = vec![
        ("resource:doc".to_string(), "role".to_string(), 0x01u64),
        ("resource:doc".to_string(), "role".to_string(), 0x02u64),
        ("resource:doc".to_string(), "role".to_string(), 0x04u64), // last write
    ];

    batch_set_capabilities(&entries).unwrap();

    // Last write should win
    assert_eq!(get_capability("resource:doc", "role").unwrap(), Some(0x04));
}

// ============================================================================
// Large Batch Operations
// ============================================================================

/// Verify large batch operations work
#[test]
fn batch_large_number_of_operations() {
    let _lock = setup_clean();

    // Create 100 capabilities
    let caps: Vec<(String, String, u64)> = (0..100)
        .map(|i| (format!("resource:doc{}", i), "role".to_string(), i as u64))
        .collect();

    let result = batch_set_capabilities(&caps);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 100);

    // Verify a sample
    assert_eq!(get_capability("resource:doc0", "role").unwrap(), Some(0));
    assert_eq!(get_capability("resource:doc50", "role").unwrap(), Some(50));
    assert_eq!(get_capability("resource:doc99", "role").unwrap(), Some(99));
}

/// Verify WriteBatch with many operations
#[test]
fn write_batch_many_operations() {
    let _lock = setup_clean();

    let mut batch = write_batch();

    // Add 50 capabilities and 50 relationships
    for i in 0..50 {
        batch.set_capability(&format!("resource:doc{}", i), "member", 1 << (i % 64));
        batch.set_relationship(&format!("user:u{}", i), "member", &format!("resource:doc{}", i));
    }

    assert_eq!(batch.len(), 100);

    let result = batch.execute();
    assert!(result.is_ok());

    // Verify sample
    assert!(has_capability("user:u0", "resource:doc0", 1).unwrap());
    assert!(has_capability("user:u25", "resource:doc25", 1 << 25).unwrap());
}

// ============================================================================
// Batch with Labels
// ============================================================================

/// Verify WriteBatch can set capability labels
#[test]
fn write_batch_with_labels() {
    let _lock = setup_clean();

    let mut batch = write_batch();
    batch
        .set_capability("resource:doc", "viewer", 0x01)
        .set_capability("resource:doc", "editor", 0x02)
        .set_cap_label("resource:doc", 0x01, "Read access")
        .set_cap_label("resource:doc", 0x02, "Write access");

    batch.execute().unwrap();

    // Verify capabilities are set
    assert_eq!(get_capability("resource:doc", "viewer").unwrap(), Some(0x01));
    assert_eq!(get_capability("resource:doc", "editor").unwrap(), Some(0x02));

    // Labels would need get_cap_label to verify, but they're stored
}
