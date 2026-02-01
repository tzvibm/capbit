//! Integration tests for capbit
//!
//! These tests verify the core API works correctly with the normalized schema.

use capbit::{
    init, bootstrap, protected, set_relationship, get_relationships, delete_relationship,
    set_capability, get_capability, set_inheritance, get_inheritance,
    check_access, has_capability, WriteBatch, batch_set_relationships,
    list_accessible, list_subjects, clear_all, test_lock,
};
use tempfile::TempDir;
use std::sync::Once;

// Capability bits
const READ: u64 = 0x01;
const WRITE: u64 = 0x02;
const DELETE: u64 = 0x04;
const ADMIN: u64 = 0x08;

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

#[test]
fn test_relationships() {
    let _lock = setup_bootstrapped();

    // Create entities first
    protected::create_entity("user:root", "user", "john").unwrap();
    protected::create_entity("user:root", "resource", "project42").unwrap();
    protected::set_capability("user:root", "resource:project42", "editor", READ).unwrap();
    protected::set_capability("user:root", "resource:project42", "viewer", READ).unwrap();

    // Set relationships
    set_relationship("user:john", "editor", "resource:project42").unwrap();
    set_relationship("user:john", "viewer", "resource:project42").unwrap();

    // Get relationships
    let rels = get_relationships("user:john", "resource:project42").unwrap();
    assert!(rels.contains(&"editor".to_string()));
    assert!(rels.contains(&"viewer".to_string()));

    // Delete relationship
    delete_relationship("user:john", "editor", "resource:project42").unwrap();
    let rels = get_relationships("user:john", "resource:project42").unwrap();
    assert!(!rels.contains(&"editor".to_string()));
    assert!(rels.contains(&"viewer".to_string()));
}

#[test]
fn test_capabilities() {
    let _lock = setup_bootstrapped();

    // Create resources first
    protected::create_entity("user:root", "app", "slack").unwrap();
    protected::create_entity("user:root", "app", "github").unwrap();

    // Define per-entity capability semantics
    set_capability("app:slack", "editor", READ | WRITE | DELETE | ADMIN).unwrap();
    set_capability("app:github", "editor", READ | WRITE).unwrap();

    // Same relationship type, different capabilities per entity
    let slack_caps = get_capability("app:slack", "editor").unwrap();
    let github_caps = get_capability("app:github", "editor").unwrap();

    assert_eq!(slack_caps, Some(0x0F));
    assert_eq!(github_caps, Some(0x03));
}

#[test]
fn test_access_checks() {
    let _lock = setup_bootstrapped();

    // Create entities
    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "docs").unwrap();

    // Setup
    set_capability("resource:docs", "editor", READ | WRITE).unwrap();
    set_relationship("user:alice", "editor", "resource:docs").unwrap();

    // Check access
    assert!(has_capability("user:alice", "resource:docs", READ).unwrap());
    assert!(has_capability("user:alice", "resource:docs", WRITE).unwrap());
    assert!(!has_capability("user:alice", "resource:docs", DELETE).unwrap());

    // check_access returns effective capability mask
    let caps = check_access("user:alice", "resource:docs", None).unwrap();
    assert_eq!(caps, READ | WRITE);
}

#[test]
fn test_inheritance() {
    let _lock = setup_bootstrapped();

    // Create entities
    protected::create_entity("user:root", "user", "mary").unwrap();
    protected::create_entity("user:root", "user", "john").unwrap();
    protected::create_entity("user:root", "team", "sales").unwrap();

    // Mary is admin on sales
    set_capability("team:sales", "admin", READ | WRITE | DELETE | ADMIN).unwrap();
    set_relationship("user:mary", "admin", "team:sales").unwrap();

    // John inherits mary's relationship to sales
    set_inheritance("user:john", "team:sales", "user:mary").unwrap();

    // John should have mary's access
    assert!(has_capability("user:john", "team:sales", ADMIN).unwrap());
    assert!(has_capability("user:john", "team:sales", DELETE).unwrap());

    // Verify inheritance sources
    let sources = get_inheritance("user:john", "team:sales").unwrap();
    assert!(sources.contains(&"user:mary".to_string()));
}

#[test]
fn test_write_batch() {
    let _lock = setup_bootstrapped();

    // Create entities first
    protected::create_entity("user:root", "resource", "batch-resource").unwrap();
    protected::create_entity("user:root", "user", "batch-user1").unwrap();
    protected::create_entity("user:root", "user", "batch-user2").unwrap();

    // Create batch
    let mut batch = WriteBatch::new();
    batch.set_capability("resource:batch-resource", "admin", READ | WRITE | DELETE);
    batch.set_relationship("user:batch-user1", "admin", "resource:batch-resource");
    batch.set_capability("resource:batch-resource", "viewer", READ);
    batch.set_relationship("user:batch-user2", "viewer", "resource:batch-resource");

    assert_eq!(batch.len(), 4);

    // Execute atomically
    let epoch = batch.execute().unwrap();
    assert!(epoch > 0);

    // Verify
    assert!(has_capability("user:batch-user1", "resource:batch-resource", DELETE).unwrap());
    assert!(has_capability("user:batch-user2", "resource:batch-resource", READ).unwrap());
    assert!(!has_capability("user:batch-user2", "resource:batch-resource", DELETE).unwrap());
}

#[test]
fn test_batch_set_relationships() {
    let _lock = setup_bootstrapped();

    // Create entities first
    protected::create_entity("user:root", "resource", "bulk-res").unwrap();
    protected::create_entity("user:root", "user", "u1").unwrap();
    protected::create_entity("user:root", "user", "u2").unwrap();
    protected::create_entity("user:root", "user", "u3").unwrap();

    set_capability("resource:bulk-res", "member", READ).unwrap();

    let entries = vec![
        ("user:u1".to_string(), "member".to_string(), "resource:bulk-res".to_string()),
        ("user:u2".to_string(), "member".to_string(), "resource:bulk-res".to_string()),
        ("user:u3".to_string(), "member".to_string(), "resource:bulk-res".to_string()),
    ];

    let count = batch_set_relationships(&entries).unwrap();
    assert_eq!(count, 3);

    assert!(has_capability("user:u1", "resource:bulk-res", READ).unwrap());
    assert!(has_capability("user:u2", "resource:bulk-res", READ).unwrap());
    assert!(has_capability("user:u3", "resource:bulk-res", READ).unwrap());
}

#[test]
fn test_query_operations() {
    let _lock = setup_bootstrapped();

    // Create entities first
    protected::create_entity("user:root", "user", "query-user").unwrap();
    protected::create_entity("user:root", "team", "org1").unwrap();
    protected::create_entity("user:root", "resource", "doc1").unwrap();
    protected::create_entity("user:root", "resource", "doc2").unwrap();

    // Set capabilities
    set_capability("team:org1", "member", READ).unwrap();
    set_capability("resource:doc1", "viewer", READ).unwrap();
    set_capability("resource:doc2", "editor", READ | WRITE).unwrap();

    // Setup
    set_relationship("user:query-user", "member", "team:org1").unwrap();
    set_relationship("user:query-user", "viewer", "resource:doc1").unwrap();
    set_relationship("user:query-user", "editor", "resource:doc2").unwrap();

    // List accessible entities
    let accessible = list_accessible("user:query-user").unwrap();
    assert!(accessible.len() >= 3);

    let objects: Vec<_> = accessible.iter().map(|(obj, _)| obj.as_str()).collect();
    assert!(objects.contains(&"team:org1"));
    assert!(objects.contains(&"resource:doc1"));
    assert!(objects.contains(&"resource:doc2"));

    // List subjects with access
    protected::create_entity("user:root", "user", "u1").unwrap();
    protected::create_entity("user:root", "user", "u2").unwrap();
    protected::create_entity("user:root", "resource", "shared-doc").unwrap();

    set_capability("resource:shared-doc", "editor", READ | WRITE).unwrap();
    set_capability("resource:shared-doc", "viewer", READ).unwrap();
    set_relationship("user:u1", "editor", "resource:shared-doc").unwrap();
    set_relationship("user:u2", "viewer", "resource:shared-doc").unwrap();

    let subjects = list_subjects("resource:shared-doc").unwrap();
    let users: Vec<_> = subjects.iter().map(|(subj, _)| subj.as_str()).collect();
    assert!(users.contains(&"user:u1"));
    assert!(users.contains(&"user:u2"));
}

#[test]
fn test_deep_inheritance() {
    let _lock = setup_bootstrapped();

    let depth = 10;

    // Create resource
    protected::create_entity("user:root", "resource", "deep-resource").unwrap();

    // Create users for chain
    for i in 0..depth {
        protected::create_entity("user:root", "user", &format!("chain-{}", i)).unwrap();
    }

    // Setup: chain-0 has direct access
    set_capability("resource:deep-resource", "admin", 0xFF).unwrap();
    set_relationship("user:chain-0", "admin", "resource:deep-resource").unwrap();

    // Create inheritance chain
    for i in 1..depth {
        set_inheritance(
            &format!("user:chain-{}", i),
            "resource:deep-resource",
            &format!("user:chain-{}", i - 1)
        ).unwrap();
    }

    // Entity at end of chain should have access
    let caps = check_access(&format!("user:chain-{}", depth - 1), "resource:deep-resource", None).unwrap();
    assert_eq!(caps, 0xFF);
}

#[test]
fn test_cycle_detection() {
    let _lock = setup_bootstrapped();

    // Create entities
    protected::create_entity("user:root", "resource", "cycle-resource").unwrap();
    protected::create_entity("user:root", "user", "cycle-A").unwrap();
    protected::create_entity("user:root", "user", "cycle-B").unwrap();
    protected::create_entity("user:root", "user", "cycle-C").unwrap();

    set_capability("resource:cycle-resource", "member", 0x0F).unwrap();
    set_relationship("user:cycle-A", "member", "resource:cycle-resource").unwrap();

    // Create cycle: A -> B -> C -> A
    set_inheritance("user:cycle-B", "resource:cycle-resource", "user:cycle-A").unwrap();
    set_inheritance("user:cycle-C", "resource:cycle-resource", "user:cycle-B").unwrap();
    set_inheritance("user:cycle-A", "resource:cycle-resource", "user:cycle-C").unwrap(); // Creates cycle!

    // Should not hang or crash
    let caps = check_access("user:cycle-C", "resource:cycle-resource", None).unwrap();
    assert_eq!(caps, 0x0F);
}
