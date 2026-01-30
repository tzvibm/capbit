//! Integration tests for capbit

use capbit::{
    init, set_relationship, get_relationships, delete_relationship,
    set_capability, get_capability, set_inheritance, get_inheritance,
    check_access, has_capability, WriteBatch, batch_set_relationships,
    list_accessible, list_subjects,
};
use tempfile::TempDir;

// Capability bits
const READ: u64 = 0x01;
const WRITE: u64 = 0x02;
const DELETE: u64 = 0x04;
const ADMIN: u64 = 0x08;

fn setup() -> TempDir {
    let dir = TempDir::new().unwrap();
    init(dir.path().to_str().unwrap()).unwrap();
    dir
}

#[test]
fn test_relationships() {
    let _dir = setup();

    // Set relationships
    set_relationship("john", "editor", "project42").unwrap();
    set_relationship("john", "viewer", "project42").unwrap();

    // Get relationships
    let rels = get_relationships("john", "project42").unwrap();
    assert!(rels.contains(&"editor".to_string()));
    assert!(rels.contains(&"viewer".to_string()));

    // Delete relationship
    delete_relationship("john", "editor", "project42").unwrap();
    let rels = get_relationships("john", "project42").unwrap();
    assert!(!rels.contains(&"editor".to_string()));
    assert!(rels.contains(&"viewer".to_string()));
}

#[test]
fn test_capabilities() {
    let _dir = setup();

    // Define per-entity capability semantics
    set_capability("slack", "editor", READ | WRITE | DELETE | ADMIN).unwrap();
    set_capability("github", "editor", READ | WRITE).unwrap();

    // Same relationship type, different capabilities per entity
    let slack_caps = get_capability("slack", "editor").unwrap();
    let github_caps = get_capability("github", "editor").unwrap();

    assert_eq!(slack_caps, Some(0x0F));
    assert_eq!(github_caps, Some(0x03));
}

#[test]
fn test_access_checks() {
    let _dir = setup();

    // Setup
    set_capability("docs", "editor", READ | WRITE).unwrap();
    set_relationship("alice", "editor", "docs").unwrap();

    // Check access
    assert!(has_capability("alice", "docs", READ).unwrap());
    assert!(has_capability("alice", "docs", WRITE).unwrap());
    assert!(!has_capability("alice", "docs", DELETE).unwrap());

    // check_access returns effective capability mask
    let caps = check_access("alice", "docs", None).unwrap();
    assert_eq!(caps, READ | WRITE);
}

#[test]
fn test_inheritance() {
    let _dir = setup();

    // Mary is admin on sales
    set_capability("sales", "admin", READ | WRITE | DELETE | ADMIN).unwrap();
    set_relationship("mary", "admin", "sales").unwrap();

    // John inherits mary's relationship to sales
    set_inheritance("john", "sales", "mary").unwrap();

    // John should have mary's access
    assert!(has_capability("john", "sales", ADMIN).unwrap());
    assert!(has_capability("john", "sales", DELETE).unwrap());

    // Verify inheritance sources
    let sources = get_inheritance("john", "sales").unwrap();
    assert!(sources.contains(&"mary".to_string()));
}

#[test]
fn test_write_batch() {
    let _dir = setup();

    // Create batch
    let mut batch = WriteBatch::new();
    batch.set_capability("batch-resource", "admin", READ | WRITE | DELETE);
    batch.set_relationship("batch-user1", "admin", "batch-resource");
    batch.set_relationship("batch-user2", "viewer", "batch-resource");
    batch.set_capability("batch-resource", "viewer", READ);

    assert_eq!(batch.len(), 4);

    // Execute atomically
    let epoch = batch.execute().unwrap();
    assert!(epoch > 0);

    // Verify
    assert!(has_capability("batch-user1", "batch-resource", DELETE).unwrap());
    assert!(has_capability("batch-user2", "batch-resource", READ).unwrap());
    assert!(!has_capability("batch-user2", "batch-resource", DELETE).unwrap());
}

#[test]
fn test_batch_set_relationships() {
    let _dir = setup();

    set_capability("bulk-res", "member", READ).unwrap();

    let entries = vec![
        ("user1".to_string(), "member".to_string(), "bulk-res".to_string()),
        ("user2".to_string(), "member".to_string(), "bulk-res".to_string()),
        ("user3".to_string(), "member".to_string(), "bulk-res".to_string()),
    ];

    let count = batch_set_relationships(&entries).unwrap();
    assert_eq!(count, 3);

    assert!(has_capability("user1", "bulk-res", READ).unwrap());
    assert!(has_capability("user2", "bulk-res", READ).unwrap());
    assert!(has_capability("user3", "bulk-res", READ).unwrap());
}

#[test]
fn test_query_operations() {
    let _dir = setup();

    // Setup
    set_relationship("query-user", "member", "org1").unwrap();
    set_relationship("query-user", "viewer", "doc1").unwrap();
    set_relationship("query-user", "editor", "doc2").unwrap();

    // List accessible entities
    let accessible = list_accessible("query-user").unwrap();
    assert!(accessible.len() >= 3);

    let objects: Vec<_> = accessible.iter().map(|(obj, _)| obj.as_str()).collect();
    assert!(objects.contains(&"org1"));
    assert!(objects.contains(&"doc1"));
    assert!(objects.contains(&"doc2"));

    // List subjects with access
    set_relationship("user1", "editor", "shared-doc").unwrap();
    set_relationship("user2", "viewer", "shared-doc").unwrap();

    let subjects = list_subjects("shared-doc").unwrap();
    let users: Vec<_> = subjects.iter().map(|(subj, _)| subj.as_str()).collect();
    assert!(users.contains(&"user1"));
    assert!(users.contains(&"user2"));
}

#[test]
fn test_deep_inheritance() {
    let _dir = setup();

    let depth = 10;
    let resource = "deep-resource";

    // Setup: chain-0 has direct access
    set_capability(resource, "admin", 0xFF).unwrap();
    set_relationship("chain-0", "admin", resource).unwrap();

    // Create inheritance chain
    for i in 1..depth {
        set_inheritance(&format!("chain-{}", i), resource, &format!("chain-{}", i - 1)).unwrap();
    }

    // Entity at end of chain should have access
    let caps = check_access(&format!("chain-{}", depth - 1), resource, None).unwrap();
    assert_eq!(caps, 0xFF);
}

#[test]
fn test_cycle_detection() {
    let _dir = setup();

    let resource = "cycle-resource";

    set_capability(resource, "member", 0x0F).unwrap();
    set_relationship("cycle-A", "member", resource).unwrap();

    // Create cycle: A -> B -> C -> A
    set_inheritance("cycle-B", resource, "cycle-A").unwrap();
    set_inheritance("cycle-C", resource, "cycle-B").unwrap();
    set_inheritance("cycle-A", resource, "cycle-C").unwrap(); // Creates cycle!

    // Should not hang or crash
    let caps = check_access("cycle-C", resource, None).unwrap();
    assert_eq!(caps, 0x0F);
}
