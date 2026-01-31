//! # Capbit
//!
//! High-performance access control library with string-based relationships
//! and bitmask capabilities.
//!
//! ## Features
//!
//! - **O(1) Bitmask Eval**: Final permission check is a single AND operation
//! - **O(log N) Lookup**: LMDB B-tree storage (full check is O(k × log N))
//! - **String Relationships**: Human-readable types ("editor", "viewer", "member")
//! - **Per-Entity Semantics**: Each entity defines what relationships mean to it
//! - **Inheritance**: Inherit relationships without graph traversal
//! - **Bidirectional**: Query "what can X access" or "who can access X"
//! - **Protected Mutations** (v2): All writes require authorization
//!
//! ## Quick Start (v2 Protected API)
//!
//! ```rust,no_run
//! use capbit::{init, bootstrap, protected, SystemCap};
//!
//! // Initialize database
//! init("/tmp/capbit.mdb").unwrap();
//!
//! // Bootstrap creates root user with full access
//! bootstrap("root").unwrap();
//!
//! // Root creates a team
//! protected::create_entity("user:root", "team", "sales").unwrap();
//!
//! // Root defines what "member" means on team:sales
//! protected::set_capability("user:root", "team:sales", "member", 0x01).unwrap();
//!
//! // Root grants alice membership
//! protected::set_grant("user:root", "user:alice", "member", "team:sales").unwrap();
//! ```
//!
//! ## v1 Compatibility
//!
//! The original unprotected API is still available for simple use cases:
//!
//! ```rust,no_run
//! use capbit::{init, set_capability, set_relationship, has_capability};
//!
//! init("/tmp/capbit.mdb").unwrap();
//! set_capability("project42", "editor", 0x03).unwrap();
//! set_relationship("john", "editor", "project42").unwrap();
//! assert!(has_capability("john", "project42", 0x01).unwrap());
//! ```

mod core;
pub mod caps;
pub mod bootstrap;
pub mod protected;
pub mod auth;

// Re-export SystemCap for convenience
pub use caps::SystemCap;

// Re-export bootstrap functions
pub use bootstrap::{bootstrap, is_bootstrapped, get_root_entity};

// Re-export core types and functions
pub use core::{
    // Types
    CapbitError,
    Result,
    WriteBatch,
    WriteOp,

    // Initialization
    init,
    close,

    // v2: Entity helpers
    parse_entity_id,  // Legacy: returns (&str, &str)
    parse_entity,     // v3: returns EntityId with O(1) type extraction
    entity_exists,
    type_exists,
    get_meta,

    // v1 Compatibility: Unprotected Relationships (use protected:: for v2)
    set_relationship,
    get_relationships,
    delete_relationship,

    // v1 Compatibility: Unprotected Capabilities (use protected:: for v2)
    set_capability,
    get_capability,

    // v1 Compatibility: Unprotected Inheritance (use protected:: for v2)
    set_inheritance,
    get_inheritance,
    delete_inheritance,
    get_inheritors_from_source,
    get_inheritance_for_object,

    // Labels
    set_cap_label,
    get_cap_label,

    // Access checks (read-only, no protection needed)
    check_access,
    has_capability,

    // Batch operations (v1 compat)
    batch_set_relationships,
    batch_set_capabilities,
    batch_set_inheritance,

    // Query operations (read-only)
    list_accessible,
    list_subjects,

    // List all (for admin/demo)
    list_all_entities,
    list_all_grants,
    list_all_capabilities,
    list_all_cap_labels,

    // WriteBatch
    write_batch,
};

// v2 naming aliases (seeker/scope/relation terminology)
pub use core::check_access as get_effective_capabilities;

// Test utilities (also available for integration tests)
pub use core::{clear_all, test_lock};

// v3: Compact entity ID representation
pub mod entity_id;
pub use entity_id::{EntityId, EntityIdError};
