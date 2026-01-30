# Capbit

High-performance access control library with string-based relationships and bitmask capabilities.

## Features

- **O(1) Evaluation**: Bitmask AND operations for permission checks
- **O(log N) Lookup**: LMDB B-tree storage
- **String Relationships**: Human-readable types ("editor", "viewer", "member")
- **Type Agnostic**: Everything is an entity—users, teams, resources, anything
- **Per-Entity Semantics**: Each entity defines what relationships mean to it
- **Inheritance**: Inherit relationships without graph traversal
- **Bidirectional**: Query "what can X access" or "who can access X"

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
capbit = "0.1"
```

## Quick Start

```rust
use capbit::{init, set_capability, set_relationship, has_capability};

// Capability bits
const READ: u64 = 0x01;
const WRITE: u64 = 0x02;
const DELETE: u64 = 0x04;

fn main() -> capbit::Result<()> {
    // Initialize database
    init("./data/capbit.mdb")?;

    // "editor" on "project42" grants read+write
    set_capability("project42", "editor", READ | WRITE)?;

    // John is an editor
    set_relationship("john", "editor", "project42")?;

    // Check access
    assert!(has_capability("john", "project42", WRITE)?);
    assert!(!has_capability("john", "project42", DELETE)?);

    Ok(())
}
```

## API

### Initialization

- `init(db_path)` - Initialize LMDB environment
- `close()` - Close database

### Relationships

- `set_relationship(subject, rel_type, object)` - Create relationship
- `get_relationships(subject, object)` - Get all relationship types
- `delete_relationship(subject, rel_type, object)` - Remove relationship

### Capabilities

- `set_capability(entity, rel_type, cap_mask)` - Define what a relationship grants
- `get_capability(entity, rel_type)` - Get capability mask for relationship type

### Inheritance

- `set_inheritance(subject, object, source)` - Subject inherits source's relationship
- `get_inheritance(subject, object)` - Get inheritance sources
- `delete_inheritance(subject, object, source)` - Remove inheritance rule
- `get_inheritors_from_source(source, object)` - Get subjects inheriting from source
- `get_inheritance_for_object(object)` - Get all inheritance rules (audit)

### Labels

- `set_cap_label(entity, cap_bit, label)` - Human-readable capability name
- `get_cap_label(entity, cap_bit)` - Get capability label

### Access Checks

- `check_access(subject, object, max_depth)` - Get effective capability mask
- `has_capability(subject, object, required_cap)` - Check specific capability

### Query Operations

- `list_accessible(subject)` - List all (object, rel_type) pairs
- `list_subjects(object)` - List all (subject, rel_type) pairs

### Batch Operations

- `batch_set_relationships(entries)` - Bulk set relationships
- `batch_set_capabilities(entries)` - Bulk set capabilities
- `batch_set_inheritance(entries)` - Bulk set inheritance

### WriteBatch (Explicit Transactions)

For atomicity and better performance:

```rust
use capbit::WriteBatch;

let mut batch = WriteBatch::new();

// Chain operations
batch
    .set_capability("project", "editor", 0x03)
    .set_relationship("john", "editor", "project")
    .set_inheritance("team", "project", "john");

// Execute all in one atomic transaction
batch.execute()?;
```

Methods:
- `WriteBatch::new()` - Create a new batch
- `set_relationship(subject, rel_type, object)` - Add relationship
- `delete_relationship(subject, rel_type, object)` - Delete relationship
- `set_capability(entity, rel_type, cap_mask)` - Set capability
- `set_inheritance(subject, object, source)` - Add inheritance
- `delete_inheritance(subject, object, source)` - Delete inheritance
- `execute()` - Execute all operations atomically
- `clear()` - Clear all operations
- `len()` / `is_empty()` - Check batch size

## Storage

Data is stored in LMDB with these indexes:

| Database | Key Pattern | Purpose |
|----------|-------------|---------|
| relationships | subject/rel_type/object | Forward lookup |
| relationships_rev | object/rel_type/subject | Reverse lookup |
| capabilities | entity/rel_type | Capability definitions |
| inheritance | subject/object/source | Forward inheritance |
| inheritance_by_source | source/object/subject | "Who inherits from X?" |
| inheritance_by_object | object/source/subject | "What rules affect X?" |
| cap_labels | entity/cap_bit | Human-readable capability names |

## License

MIT
