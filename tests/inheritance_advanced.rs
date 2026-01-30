//! Advanced inheritance pattern tests for Capbit v2
//!
//! These tests verify complex delegation scenarios including diamond patterns,
//! wide inheritance, mixed paths, and edge cases.

use capbit::{
    init, bootstrap, protected, check_access,
    set_inheritance, get_inheritance, get_inheritors_from_source,
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
// Diamond Inheritance Pattern
// ============================================================================

/// Verify diamond inheritance pattern works correctly
///     A (0x0F)
///    / \
///   B   C
///    \ /
///     D
/// D should inherit from both B and C, which both inherit from A
#[test]
fn diamond_inheritance_pattern() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "a").unwrap();
    protected::create_entity("user:root", "user", "b").unwrap();
    protected::create_entity("user:root", "user", "c").unwrap();
    protected::create_entity("user:root", "user", "d").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // A has direct access
    protected::set_capability("user:root", "resource:doc", "admin", 0x0F).unwrap();
    protected::set_grant("user:root", "user:a", "admin", "resource:doc").unwrap();

    // B and C inherit from A
    set_inheritance("user:b", "resource:doc", "user:a").unwrap();
    set_inheritance("user:c", "resource:doc", "user:a").unwrap();

    // D inherits from both B and C
    set_inheritance("user:d", "resource:doc", "user:b").unwrap();
    set_inheritance("user:d", "resource:doc", "user:c").unwrap();

    // All should have access
    assert_eq!(check_access("user:a", "resource:doc", None).unwrap(), 0x0F);
    assert_eq!(check_access("user:b", "resource:doc", None).unwrap(), 0x0F);
    assert_eq!(check_access("user:c", "resource:doc", None).unwrap(), 0x0F);
    assert_eq!(check_access("user:d", "resource:doc", None).unwrap(), 0x0F);
}

// ============================================================================
// Wide Inheritance
// ============================================================================

/// Verify inheriting from many sources works
/// User inherits from 10 different sources
#[test]
fn wide_inheritance_many_sources() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "collector").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Create 10 sources, each with a different capability bit
    for i in 0..10 {
        let source_name = format!("source{}", i);
        protected::create_entity("user:root", "user", &source_name).unwrap();

        let role = format!("role{}", i);
        let cap = 1u64 << i;
        protected::set_capability("user:root", "resource:doc", &role, cap).unwrap();
        protected::set_grant("user:root", &format!("user:{}", source_name), &role, "resource:doc").unwrap();

        // Collector inherits from each source
        set_inheritance("user:collector", "resource:doc", &format!("user:{}", source_name)).unwrap();
    }

    // Collector should have all 10 bits (0x3FF = bits 0-9)
    let caps = check_access("user:collector", "resource:doc", None).unwrap();
    assert_eq!(caps, 0x3FF);
}

// ============================================================================
// Mixed Direct and Inherited
// ============================================================================

/// Verify direct grants combine with inherited caps
#[test]
fn mixed_direct_and_inherited() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Alice has bits 0,1 directly
    protected::set_capability("user:root", "resource:doc", "viewer", 0x03).unwrap();
    protected::set_grant("user:root", "user:alice", "viewer", "resource:doc").unwrap();

    // Bob has bits 2,3 directly
    protected::set_capability("user:root", "resource:doc", "editor", 0x0C).unwrap();
    protected::set_grant("user:root", "user:bob", "editor", "resource:doc").unwrap();

    // Bob also inherits from alice
    set_inheritance("user:bob", "resource:doc", "user:alice").unwrap();

    // Bob should have all 4 bits (direct 0x0C + inherited 0x03 = 0x0F)
    let bob_caps = check_access("user:bob", "resource:doc", None).unwrap();
    assert_eq!(bob_caps, 0x0F);

    // Alice still has only her direct bits
    let alice_caps = check_access("user:alice", "resource:doc", None).unwrap();
    assert_eq!(alice_caps, 0x03);
}

// ============================================================================
// Deep Inheritance Chain
// ============================================================================

/// Verify inheritance chain of 20 levels works
#[test]
fn inheritance_depth_20_levels() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "resource", "doc").unwrap();
    protected::set_capability("user:root", "resource:doc", "member", 0x01).unwrap();

    // Create chain: u0 -> u1 -> u2 -> ... -> u19
    let depth = 20;
    for i in 0..depth {
        protected::create_entity("user:root", "user", &format!("u{}", i)).unwrap();
    }

    // u0 has direct access
    protected::set_grant("user:root", "user:u0", "member", "resource:doc").unwrap();

    // Create inheritance chain
    for i in 1..depth {
        set_inheritance(
            &format!("user:u{}", i),
            "resource:doc",
            &format!("user:u{}", i - 1),
        ).unwrap();
    }

    // All users in chain should have access
    for i in 0..depth {
        let caps = check_access(&format!("user:u{}", i), "resource:doc", None).unwrap();
        assert_eq!(caps, 0x01, "user:u{} should have capability", i);
    }
}

// ============================================================================
// Different Relations in Inheritance
// ============================================================================

/// Verify inheritance works with different relation names
#[test]
fn inheritance_with_different_relations() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Alice has multiple relations
    protected::set_capability("user:root", "resource:doc", "viewer", 0x01).unwrap();
    protected::set_capability("user:root", "resource:doc", "commenter", 0x02).unwrap();
    protected::set_capability("user:root", "resource:doc", "editor", 0x04).unwrap();

    protected::set_grant("user:root", "user:alice", "viewer", "resource:doc").unwrap();
    protected::set_grant("user:root", "user:alice", "commenter", "resource:doc").unwrap();
    protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();

    // Bob inherits from alice
    set_inheritance("user:bob", "resource:doc", "user:alice").unwrap();

    // Bob gets all of alice's relations
    let bob_caps = check_access("user:bob", "resource:doc", None).unwrap();
    assert_eq!(bob_caps, 0x07);
}

// ============================================================================
// Multiple Paths to Same Source
// ============================================================================

/// Verify multiple inheritance paths to same source don't double-count
#[test]
fn multiple_paths_to_same_source() {
    let _lock = setup_bootstrapped();

    //     A (0x01)
    //    /|\
    //   B C D
    //    \|/
    //     E
    // E inherits from B, C, D which all inherit from A

    protected::create_entity("user:root", "user", "a").unwrap();
    protected::create_entity("user:root", "user", "b").unwrap();
    protected::create_entity("user:root", "user", "c").unwrap();
    protected::create_entity("user:root", "user", "d").unwrap();
    protected::create_entity("user:root", "user", "e").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    protected::set_capability("user:root", "resource:doc", "member", 0x01).unwrap();
    protected::set_grant("user:root", "user:a", "member", "resource:doc").unwrap();

    // B, C, D inherit from A
    set_inheritance("user:b", "resource:doc", "user:a").unwrap();
    set_inheritance("user:c", "resource:doc", "user:a").unwrap();
    set_inheritance("user:d", "resource:doc", "user:a").unwrap();

    // E inherits from B, C, D
    set_inheritance("user:e", "resource:doc", "user:b").unwrap();
    set_inheritance("user:e", "resource:doc", "user:c").unwrap();
    set_inheritance("user:e", "resource:doc", "user:d").unwrap();

    // E should have 0x01 (not 0x03 from double/triple counting)
    let e_caps = check_access("user:e", "resource:doc", None).unwrap();
    assert_eq!(e_caps, 0x01);
}

// ============================================================================
// Inheritance Edge Cases
// ============================================================================

/// Verify self-inheritance is handled (should be no-op or ignored)
#[test]
fn self_inheritance_ignored() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    protected::set_capability("user:root", "resource:doc", "member", 0x01).unwrap();
    protected::set_grant("user:root", "user:alice", "member", "resource:doc").unwrap();

    // Alice inherits from herself (weird but should not break)
    set_inheritance("user:alice", "resource:doc", "user:alice").unwrap();

    // Should still have exactly 0x01
    let caps = check_access("user:alice", "resource:doc", None).unwrap();
    assert_eq!(caps, 0x01);
}

/// Verify inheritance order doesn't affect result
#[test]
fn inheritance_order_independence() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "a").unwrap();
    protected::create_entity("user:root", "user", "b").unwrap();
    protected::create_entity("user:root", "user", "collector1").unwrap();
    protected::create_entity("user:root", "user", "collector2").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    protected::set_capability("user:root", "resource:doc", "role_a", 0x01).unwrap();
    protected::set_capability("user:root", "resource:doc", "role_b", 0x02).unwrap();
    protected::set_grant("user:root", "user:a", "role_a", "resource:doc").unwrap();
    protected::set_grant("user:root", "user:b", "role_b", "resource:doc").unwrap();

    // collector1 inherits A then B
    set_inheritance("user:collector1", "resource:doc", "user:a").unwrap();
    set_inheritance("user:collector1", "resource:doc", "user:b").unwrap();

    // collector2 inherits B then A (opposite order)
    set_inheritance("user:collector2", "resource:doc", "user:b").unwrap();
    set_inheritance("user:collector2", "resource:doc", "user:a").unwrap();

    // Both should have same result
    let caps1 = check_access("user:collector1", "resource:doc", None).unwrap();
    let caps2 = check_access("user:collector2", "resource:doc", None).unwrap();
    assert_eq!(caps1, caps2);
    assert_eq!(caps1, 0x03);
}

/// Verify direct plus inherited capabilities combine correctly
#[test]
fn inherited_plus_direct_combine() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    protected::set_capability("user:root", "resource:doc", "viewer", 0x01).unwrap();
    protected::set_capability("user:root", "resource:doc", "editor", 0x02).unwrap();

    // Alice has viewer
    protected::set_grant("user:root", "user:alice", "viewer", "resource:doc").unwrap();

    // Bob has editor directly
    protected::set_grant("user:root", "user:bob", "editor", "resource:doc").unwrap();

    // Bob also inherits from alice (gets viewer)
    set_inheritance("user:bob", "resource:doc", "user:alice").unwrap();

    // Bob has both: viewer (inherited) + editor (direct) = 0x03
    let bob_caps = check_access("user:bob", "resource:doc", None).unwrap();
    assert_eq!(bob_caps, 0x03);
}

// ============================================================================
// Inheritance Query Operations
// ============================================================================

/// Verify get_inheritance returns all sources
#[test]
fn get_inheritance_returns_all_sources() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "user", "charlie").unwrap();
    protected::create_entity("user:root", "user", "collector").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Collector inherits from all three
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
fn get_inheritors_returns_all() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "source").unwrap();
    protected::create_entity("user:root", "user", "inheritor1").unwrap();
    protected::create_entity("user:root", "user", "inheritor2").unwrap();
    protected::create_entity("user:root", "user", "inheritor3").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // All inherit from source
    set_inheritance("user:inheritor1", "resource:doc", "user:source").unwrap();
    set_inheritance("user:inheritor2", "resource:doc", "user:source").unwrap();
    set_inheritance("user:inheritor3", "resource:doc", "user:source").unwrap();

    let inheritors = get_inheritors_from_source("user:source", "resource:doc").unwrap();
    assert_eq!(inheritors.len(), 3);
    assert!(inheritors.contains(&"user:inheritor1".to_string()));
    assert!(inheritors.contains(&"user:inheritor2".to_string()));
    assert!(inheritors.contains(&"user:inheritor3".to_string()));
}

// ============================================================================
// Partial Capability Inheritance
// ============================================================================

/// Verify inheritance is bounded by source's actual capabilities
#[test]
fn partial_capability_inheritance() {
    let _lock = setup_bootstrapped();

    protected::create_entity("user:root", "user", "alice").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "resource", "doc").unwrap();

    // Define high capability role
    protected::set_capability("user:root", "resource:doc", "admin", 0xFF).unwrap();
    // Define low capability role
    protected::set_capability("user:root", "resource:doc", "viewer", 0x01).unwrap();

    // Alice only has viewer (0x01), not admin
    protected::set_grant("user:root", "user:alice", "viewer", "resource:doc").unwrap();

    // Bob inherits from alice
    set_inheritance("user:bob", "resource:doc", "user:alice").unwrap();

    // Bob can only get what alice has (0x01), not the full admin (0xFF)
    let bob_caps = check_access("user:bob", "resource:doc", None).unwrap();
    assert_eq!(bob_caps, 0x01);
}
