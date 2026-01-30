# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

## Project Overview

Capbit is a pure Rust library for high-performance access control. Everything is an entity, relationships are strings (e.g., "editor", "viewer"), and capability semantics are defined per-entity as bitmasks. The system achieves O(log N) lookup and O(1) evaluation.

## Commands

```bash
cargo build            # Build library
cargo build --release  # Build optimized
cargo test             # Run all tests
cargo doc --open       # Generate and view documentation
```

## Architecture

**Stack:** Pure Rust + LMDB (via heed)

### Core Abstraction

Everything is an **entity**. The system doesn't know what entities represent—that's business context. Entities could be users, teams, apps, rooms, dates, events, services, or anything else.

### Storage Patterns

| Pattern | Purpose |
|---------|---------|
| `subject/rel_type/object` | Relationship between entities |
| `entity/rel_type` → cap_mask | Capability definition (per-entity) |
| `subject/object/source` | Inheritance reference |
| `entity/cap_bit` → label | Human-readable capability name |

### Sub-Databases

```
LMDB
├── relationships/           (subject/rel_type/object → epoch)
├── relationships_rev/       (object/rel_type/subject → epoch)
├── capabilities/            (entity/rel_type → cap_mask)
├── inheritance/             (subject/object/source → epoch)
├── inheritance_by_source/   (source/object/subject → epoch)
├── inheritance_by_object/   (object/source/subject → epoch)
└── cap_labels/              (entity/cap_bit → label)
```

**Inheritance indexes enable three query patterns:**
1. `subject/object/*` → "Who does subject inherit from for object?"
2. `source/object/*` → "Who inherits from source for object?"
3. `object/*/*` → "What inheritance rules affect object?" (audit/admin)

### Access Evaluation

Three lookups, left to right:

```
Can subject perform action on object?

Step 1: subject/*/object
→ get all existing rel_types (direct relationships)

Step 2: subject/object/*
→ if inheritance exists, get source entities
→ do step 1 for each source (inherited relationships)

Step 3: object/rel_type → cap_mask
→ for each rel_type from steps 1 and 2
→ look up capability mask
→ OR all capability masks together
→ evaluate requested action against effective bits
```

### Complexity

| Operation | Complexity |
|-----------|------------|
| Key lookup | O(log N) via B-tree |
| Prefix scan | O(log N + K), K = results |
| Bitmask evaluation | O(1) |
| Access check (3 lookups) | O(log N) |

## Write Strategies

Three strategies for different use cases:

| Strategy | API | Use Case |
|----------|-----|----------|
| Single-op | `set_relationship()` | Simple apps, low write volume |
| WriteBatch | `WriteBatch::new()` | Atomicity, controlled batching |
| Batch functions | `batch_set_relationships()` | High-throughput bulk inserts |

## Usage Example

```rust
use capbit::{init, set_capability, set_relationship, has_capability, WriteBatch};

// Capability bits
const READ: u64 = 0x01;
const WRITE: u64 = 0x02;
const DELETE: u64 = 0x04;

fn main() -> capbit::Result<()> {
    init("./data/capbit.mdb")?;

    // Define capability semantics
    set_capability("project42", "editor", READ | WRITE)?;
    set_capability("project42", "viewer", READ)?;

    // Set relationships
    set_relationship("john", "editor", "project42")?;

    // Check access
    assert!(has_capability("john", "project42", WRITE)?);

    // WriteBatch for atomic operations
    let mut batch = WriteBatch::new();
    batch.set_relationship("alice", "viewer", "project42");
    batch.set_relationship("bob", "editor", "project42");
    batch.execute()?;

    Ok(())
}
```

## File Structure

```
capbit/
├── src/
│   ├── lib.rs          # Public API re-exports
│   └── core.rs         # Core implementation
├── tests/
│   └── integration.rs  # Integration tests
├── Cargo.toml          # Rust package config
└── README.md           # User documentation
```

## Design Principles

1. **Type Agnostic**: No types in paths; business layer defines meaning
2. **String Relationships**: Human-readable types ("editor", "viewer", "member")
3. **Bitmask Capabilities**: O(1) permission evaluation via AND operations
4. **O(log N) Access**: LMDB B-tree lookups
5. **Per-Entity Semantics**: Each entity defines its own capability mappings
6. **Inheritance**: Path reference, not graph traversal
7. **Deterministic**: Epochs order all operations
8. **ACID**: Transactional forward/reverse writes
9. **Bidirectional**: Query from either entity's perspective
10. **Configurable Write Strategy**: Single-op, batch, or explicit transactions
