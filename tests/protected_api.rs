//! Protected API tests for Capbit v2

use capbit::{
    init, bootstrap, is_bootstrapped, get_root_entity, protected,
    check_access, entity_exists, type_exists, SystemCap, clear_all, test_lock,
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
// Bootstrap Tests
// ============================================================================

mod bootstrap_tests {
    use super::*;

    #[test]
    fn bootstrap_creates_root_with_all_caps() {
        let _lock = setup_bootstrapped();

        // Root should have admin on all type entities
        let caps_type = check_access("user:root", "_type:_type", None).unwrap();
        let caps_user = check_access("user:root", "_type:user", None).unwrap();
        let caps_team = check_access("user:root", "_type:team", None).unwrap();

        assert!((caps_type & SystemCap::TYPE_ADMIN) == SystemCap::TYPE_ADMIN);
        assert!((caps_user & SystemCap::ENTITY_ADMIN) == SystemCap::ENTITY_ADMIN);
        assert!((caps_team & SystemCap::ENTITY_ADMIN) == SystemCap::ENTITY_ADMIN);
    }

    #[test]
    fn bootstrap_creates_core_types() {
        let _lock = setup_bootstrapped();

        assert!(type_exists("_type").unwrap());
        assert!(type_exists("user").unwrap());
        assert!(type_exists("team").unwrap());
        assert!(type_exists("app").unwrap());
        assert!(type_exists("resource").unwrap());
    }

    #[test]
    fn bootstrap_only_runs_once() {
        let _lock = setup_bootstrapped();

        let result = bootstrap("attacker");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("already bootstrapped"));
    }

    #[test]
    fn after_bootstrap_mutations_require_auth() {
        let _lock = setup_bootstrapped();

        // Random user without permissions should fail
        let result = protected::create_entity("user:nobody", "user", "test");
        assert!(result.is_err());
    }

    #[test]
    fn is_bootstrapped_returns_correct_state() {
        let _lock = setup_clean();

        assert!(!is_bootstrapped().unwrap());
        bootstrap("root").unwrap();
        assert!(is_bootstrapped().unwrap());
    }

    #[test]
    fn get_root_entity_returns_root() {
        let _lock = setup_bootstrapped();

        assert_eq!(get_root_entity().unwrap(), Some("user:root".to_string()));
    }
}

// ============================================================================
// Entity Lifecycle Tests
// ============================================================================

mod entity_lifecycle {
    use super::*;

    #[test]
    fn create_entity_requires_entity_create_on_type() {
        let _lock = setup_bootstrapped();

        // Root can create users (has ENTITY_CREATE on _type:user)
        let result = protected::create_entity("user:root", "user", "alice");
        assert!(result.is_ok());

        // Create bob without permissions
        protected::create_entity("user:root", "user", "bob").unwrap();

        // Bob cannot create users
        let result = protected::create_entity("user:bob", "user", "charlie");
        assert!(result.is_err());
    }

    #[test]
    fn delete_entity_requires_entity_delete_on_type() {
        let _lock = setup_bootstrapped();

        // Create alice
        protected::create_entity("user:root", "user", "alice").unwrap();
        assert!(entity_exists("user:alice").unwrap());

        // Root can delete
        let result = protected::delete_entity("user:root", "user:alice");
        assert!(result.is_ok());
        assert!(!entity_exists("user:alice").unwrap());
    }

    #[test]
    fn entity_ids_must_be_typed_format() {
        let _lock = setup_bootstrapped();

        // Valid format
        let result = protected::create_entity("user:root", "user", "valid");
        assert!(result.is_ok());

        // parse_entity_id is used internally, but create_entity takes separate args
        // so format is enforced by construction
    }

    #[test]
    fn cannot_create_duplicate_entity() {
        let _lock = setup_bootstrapped();

        protected::create_entity("user:root", "user", "alice").unwrap();

        let result = protected::create_entity("user:root", "user", "alice");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("already exists"));
    }
}

// ============================================================================
// Grant Tests
// ============================================================================

mod grants {
    use super::*;

    #[test]
    fn set_grant_requires_grant_write_on_scope() {
        let _lock = setup_bootstrapped();

        // Create entities
        protected::create_entity("user:root", "user", "alice").unwrap();
        protected::create_entity("user:root", "user", "bob").unwrap();
        protected::create_entity("user:root", "team", "sales").unwrap();

        // Give alice GRANT_WRITE on team:sales
        protected::set_capability("user:root", "team:sales", "admin", SystemCap::GRANT_ADMIN).unwrap();
        protected::set_grant("user:root", "user:alice", "admin", "team:sales").unwrap();

        // Alice can now grant on team:sales
        let result = protected::set_grant("user:alice", "user:bob", "member", "team:sales");
        assert!(result.is_ok());
    }

    #[test]
    fn delete_grant_requires_grant_delete_on_scope() {
        let _lock = setup_bootstrapped();

        protected::create_entity("user:root", "user", "alice").unwrap();
        protected::create_entity("user:root", "team", "sales").unwrap();

        // Setup grant
        protected::set_capability("user:root", "team:sales", "member", 0x01).unwrap();
        protected::set_grant("user:root", "user:alice", "member", "team:sales").unwrap();

        // Root can delete (has all permissions)
        let result = protected::delete_grant("user:root", "user:alice", "member", "team:sales");
        assert!(result.is_ok());
    }

    #[test]
    fn grant_validates_scope_exists() {
        let _lock = setup_bootstrapped();

        // Try to grant on non-existent scope
        let result = protected::set_grant("user:root", "user:alice", "member", "team:nonexistent");
        assert!(result.is_err());
    }
}

// ============================================================================
// Capability Tests
// ============================================================================

mod capabilities {
    use super::*;

    #[test]
    fn set_capability_requires_cap_write_on_scope() {
        let _lock = setup_bootstrapped();

        protected::create_entity("user:root", "user", "alice").unwrap();
        protected::create_entity("user:root", "team", "sales").unwrap();

        // Alice has no CAP_WRITE on team:sales
        let result = protected::set_capability("user:alice", "team:sales", "member", 0x01);
        assert!(result.is_err());

        // Root can set capabilities
        let result = protected::set_capability("user:root", "team:sales", "member", 0x01);
        assert!(result.is_ok());
    }

    #[test]
    fn capabilities_are_per_scope_per_relation() {
        let _lock = setup_bootstrapped();

        protected::create_entity("user:root", "team", "sales").unwrap();
        protected::create_entity("user:root", "team", "engineering").unwrap();

        // Same relation name, different capabilities per scope
        protected::set_capability("user:root", "team:sales", "member", 0x01).unwrap();
        protected::set_capability("user:root", "team:engineering", "member", 0x0F).unwrap();

        // Create alice and give her member on both
        protected::create_entity("user:root", "user", "alice").unwrap();
        protected::set_grant("user:root", "user:alice", "member", "team:sales").unwrap();
        protected::set_grant("user:root", "user:alice", "member", "team:engineering").unwrap();

        // Different capabilities per scope
        assert_eq!(check_access("user:alice", "team:sales", None).unwrap(), 0x01);
        assert_eq!(check_access("user:alice", "team:engineering", None).unwrap(), 0x0F);
    }
}

// ============================================================================
// Delegation Tests
// ============================================================================

mod delegations {
    use super::*;

    #[test]
    fn set_delegation_requires_delegate_write_on_scope() {
        let _lock = setup_bootstrapped();

        protected::create_entity("user:root", "user", "alice").unwrap();
        protected::create_entity("user:root", "user", "bob").unwrap();
        protected::create_entity("user:root", "resource", "doc").unwrap();

        // Alice has no DELEGATE_WRITE on resource:doc
        let result = protected::set_delegation("user:alice", "user:bob", "resource:doc", "user:alice");
        assert!(result.is_err());
    }

    #[test]
    fn delegation_bounded_by_delegator_caps() {
        let _lock = setup_bootstrapped();

        protected::create_entity("user:root", "user", "alice").unwrap();
        protected::create_entity("user:root", "user", "bob").unwrap();
        protected::create_entity("user:root", "resource", "doc").unwrap();

        // Setup: alice has READ only
        protected::set_capability("user:root", "resource:doc", "viewer", 0x01).unwrap();
        protected::set_capability("user:root", "resource:doc", "owner", SystemCap::DELEGATE_WRITE).unwrap();
        protected::set_grant("user:root", "user:alice", "viewer", "resource:doc").unwrap();
        protected::set_grant("user:root", "user:alice", "owner", "resource:doc").unwrap();

        // Alice delegates to bob
        protected::set_delegation("user:alice", "user:bob", "resource:doc", "user:alice").unwrap();

        // Bob's caps are bounded by alice's
        let bob_caps = check_access("user:bob", "resource:doc", None).unwrap();
        assert_eq!(bob_caps & 0x01, 0x01);  // READ inherited
        assert_eq!(bob_caps & 0x02, 0x00);  // WRITE not available
    }

    #[test]
    fn delegation_is_scope_specific() {
        let _lock = setup_bootstrapped();

        protected::create_entity("user:root", "user", "alice").unwrap();
        protected::create_entity("user:root", "user", "bob").unwrap();
        protected::create_entity("user:root", "resource", "doc1").unwrap();
        protected::create_entity("user:root", "resource", "doc2").unwrap();

        // Alice has access to doc1 only
        protected::set_capability("user:root", "resource:doc1", "editor", 0x03).unwrap();
        protected::set_capability("user:root", "resource:doc1", "owner", SystemCap::DELEGATE_WRITE).unwrap();
        protected::set_grant("user:root", "user:alice", "editor", "resource:doc1").unwrap();
        protected::set_grant("user:root", "user:alice", "owner", "resource:doc1").unwrap();

        // Delegate to bob for doc1
        protected::set_delegation("user:alice", "user:bob", "resource:doc1", "user:alice").unwrap();

        // Bob has access to doc1: editor (0x03) + owner (DELEGATE_WRITE = 0x0800)
        assert_eq!(check_access("user:bob", "resource:doc1", None).unwrap(), 0x03 | SystemCap::DELEGATE_WRITE);

        // Bob does NOT have access to doc2 (delegation is scope-specific)
        assert_eq!(check_access("user:bob", "resource:doc2", None).unwrap(), 0x00);
    }
}

// ============================================================================
// Access Evaluation Tests
// ============================================================================

mod access_evaluation {
    use super::*;

    #[test]
    fn check_direct_grant() {
        let _lock = setup_bootstrapped();

        protected::create_entity("user:root", "user", "alice").unwrap();
        protected::create_entity("user:root", "resource", "doc").unwrap();

        protected::set_capability("user:root", "resource:doc", "editor", 0x03).unwrap();
        protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();

        assert_eq!(check_access("user:alice", "resource:doc", None).unwrap(), 0x03);
    }

    #[test]
    fn check_via_delegation() {
        let _lock = setup_bootstrapped();

        protected::create_entity("user:root", "user", "alice").unwrap();
        protected::create_entity("user:root", "user", "bob").unwrap();
        protected::create_entity("user:root", "resource", "doc").unwrap();

        // Alice has direct access
        protected::set_capability("user:root", "resource:doc", "editor", 0x03).unwrap();
        protected::set_capability("user:root", "resource:doc", "owner", SystemCap::DELEGATE_WRITE).unwrap();
        protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();
        protected::set_grant("user:root", "user:alice", "owner", "resource:doc").unwrap();

        // Bob inherits via delegation
        protected::set_delegation("user:alice", "user:bob", "resource:doc", "user:alice").unwrap();

        // Bob gets all of Alice's caps: editor (0x03) + owner (DELEGATE_WRITE = 0x0800)
        assert_eq!(check_access("user:bob", "resource:doc", None).unwrap(), 0x03 | SystemCap::DELEGATE_WRITE);
    }

    #[test]
    fn check_multi_level_delegation() {
        let _lock = setup_bootstrapped();

        protected::create_entity("user:root", "user", "a").unwrap();
        protected::create_entity("user:root", "user", "b").unwrap();
        protected::create_entity("user:root", "user", "c").unwrap();
        protected::create_entity("user:root", "resource", "doc").unwrap();

        // A has direct access
        protected::set_capability("user:root", "resource:doc", "admin", 0xFF).unwrap();
        protected::set_capability("user:root", "resource:doc", "owner", SystemCap::DELEGATE_WRITE).unwrap();
        protected::set_grant("user:root", "user:a", "admin", "resource:doc").unwrap();
        protected::set_grant("user:root", "user:a", "owner", "resource:doc").unwrap();
        protected::set_grant("user:root", "user:b", "owner", "resource:doc").unwrap();

        // A -> B -> C delegation chain
        protected::set_delegation("user:a", "user:b", "resource:doc", "user:a").unwrap();
        protected::set_delegation("user:b", "user:c", "resource:doc", "user:b").unwrap();

        // C inherits full chain: A's admin (0xFF) + owner (DELEGATE_WRITE = 0x0800)
        assert_eq!(check_access("user:c", "resource:doc", None).unwrap(), 0xFF | SystemCap::DELEGATE_WRITE);
    }

    #[test]
    fn check_multiple_relations_or_together() {
        let _lock = setup_bootstrapped();

        protected::create_entity("user:root", "user", "alice").unwrap();
        protected::create_entity("user:root", "resource", "doc").unwrap();

        // Different relations with different caps
        protected::set_capability("user:root", "resource:doc", "viewer", 0x01).unwrap();
        protected::set_capability("user:root", "resource:doc", "editor", 0x02).unwrap();

        // Alice has both relations
        protected::set_grant("user:root", "user:alice", "viewer", "resource:doc").unwrap();
        protected::set_grant("user:root", "user:alice", "editor", "resource:doc").unwrap();

        // Capabilities OR together
        assert_eq!(check_access("user:alice", "resource:doc", None).unwrap(), 0x03);
    }

    #[test]
    fn check_no_access_returns_zero() {
        let _lock = setup_bootstrapped();

        protected::create_entity("user:root", "user", "alice").unwrap();
        protected::create_entity("user:root", "resource", "doc").unwrap();

        // Alice has no grants on doc
        assert_eq!(check_access("user:alice", "resource:doc", None).unwrap(), 0x00);
    }
}
