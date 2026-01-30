//! # Capbit
//!
//! High-performance access control library with string-based relationships
//! and bitmask capabilities.
//!
//! ## Features
//!
//! - **O(1) Evaluation**: Bitmask AND operations for permission checks
//! - **O(log N) Lookup**: LMDB B-tree storage
//! - **String Relationships**: Human-readable types ("editor", "viewer", "member")
//! - **Per-Entity Semantics**: Each entity defines what relationships mean to it
//! - **Inheritance**: Inherit relationships without graph traversal
//! - **Bidirectional**: Query "what can X access" or "who can access X"
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use capbit::{init, set_capability, set_relationship, has_capability};
//!
//! // Capability bits
//! const READ: u64 = 0x01;
//! const WRITE: u64 = 0x02;
//!
//! // Initialize database
//! init("/tmp/capbit.mdb").unwrap();
//!
//! // "editor" on "project42" grants read+write
//! set_capability("project42", "editor", READ | WRITE).unwrap();
//!
//! // John is an editor
//! set_relationship("john", "editor", "project42").unwrap();
//!
//! // Check access
//! assert!(has_capability("john", "project42", WRITE).unwrap());
//! ```
//!
//! ## Write Strategies
//!
//! Three strategies for different use cases:
//!
//! ```rust,no_run
//! use capbit::{set_relationship, WriteBatch, batch_set_relationships};
//!
//! // Strategy 1: Single-op (simple, one txn per call)
//! set_relationship("a", "editor", "b").unwrap();
//!
//! // Strategy 2: WriteBatch (explicit transaction, atomic)
//! let mut batch = WriteBatch::new();
//! batch.set_relationship("a", "editor", "b");
//! batch.set_relationship("c", "viewer", "d");
//! batch.execute().unwrap(); // One transaction for both
//!
//! // Strategy 3: Batch functions (bulk, high throughput)
//! batch_set_relationships(&[
//!     ("x".into(), "member".into(), "y".into()),
//!     ("z".into(), "admin".into(), "w".into()),
//! ]).unwrap();
//! ```

mod core;

// Re-export everything from core
pub use core::{
    // Types
    CapbitError,
    Result,
    WriteBatch,
    WriteOp,

    // Initialization
    init,
    close,

    // Relationships
    set_relationship,
    get_relationships,
    delete_relationship,

    // Capabilities
    set_capability,
    get_capability,

    // Inheritance
    set_inheritance,
    get_inheritance,
    delete_inheritance,
    get_inheritors_from_source,
    get_inheritance_for_object,

    // Labels
    set_cap_label,
    get_cap_label,

    // Access checks
    check_access,
    has_capability,

    // Batch operations
    batch_set_relationships,
    batch_set_capabilities,
    batch_set_inheritance,

    // Query operations
    list_accessible,
    list_subjects,

    // WriteBatch
    write_batch,
};
