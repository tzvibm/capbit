# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

## Project Overview

Capbit is a minimal, high-performance access control system where everything is an entity, relationships are strings (e.g., "editor", "viewer"), and capability semantics are defined per-entity as bitmasks. The system achieves O(log N) lookup and O(1) evaluation with linear scaling, no global schema, and deterministic ordering via epochs.

## Commands

```bash
npm run build          # Build Rust native module (release)
npm run build:debug    # Build Rust native module (debug)
npm test               # Run all tests (Vitest)
npm test:watch         # Run tests in watch mode
```

## Architecture

**Stack:** Rust (NAPI-RS) + LMDB + Node.js bindings

### Core Abstraction

Everything is an **entity**. The system doesn't know what entities represent—that's business context. Entities could be users, teams, apps, rooms, dates, events, services, or anything else.

The storage layer contains only:
- IDs (entity identifiers)
- Relationship types (strings like "editor", "viewer", "member")
- Capability bitmasks (for O(1) permission evaluation)
- Epochs (timestamps for ordering)

No types. No schema. Just paths, strings, and bits.

### Path Patterns

Four patterns define the entire system:

| Pattern | Purpose |
|---------|---------|
| `subject/rel_type/object` | Relationship between entities |
| `entity/rel_type` → cap_mask | Capability definition (per-entity) |
| `subject/object/source` | Inheritance reference |
| `entity/cap_bit` → label | Human-readable capability name |

All paths store **epoch** as value (except capabilities which store cap_mask).

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

### Key Concepts

**Relationship**: `john/editor/slack → epoch`
Entity "john" has relationship type "editor" with entity "slack". The system doesn't know john is a user or slack is an app—that's business knowledge.

**Per-Entity Capability Semantics**: The same relationship type grants different capabilities on different entities:
```
slack/editor → 0x0F    (editor in slack: read, write, delete, admin)
github/editor → 0x03   (editor in github: read, write only)
```

**Inheritance**: `john/sales/mary → epoch`
John inherits whatever relationship mary has with sales.

### Access Evaluation

Three lookups, left to right:

```
Can subject perform action on object?

Step 1: subject/*/object
→ get all existing rel_types (direct relationships)
→ e.g., ["editor", "viewer"]

Step 2: subject/object/*
→ if inheritance exists, get source entities
→ do step 1 for each source (inherited relationships)

Step 3: object/rel_type → cap_mask
→ for each rel_type from steps 1 and 2
→ look up capability mask for that relationship type
→ OR all capability masks together
→ evaluate requested action against effective capability bits
```

### Complexity

| Operation | Complexity |
|-----------|------------|
| Key lookup | O(log N) via B-tree |
| Prefix scan | O(log N + K), K = results |
| Bitmask evaluation | O(1) |
| Access check (3 lookups) | O(log N) |

### Bidirectional Storage

Every write is transactional on forward and reverse paths:
```
Transaction:
  john/editor/sales → epoch
  sales/editor/john → epoch
```

Enables O(log N) queries from either direction:
- "What can john access?" → scan `john/*/*`
- "Who can access sales?" → scan `sales/*/*`

## Usage Example

```javascript
const capbit = require('./capbit.node');

// Initialize
capbit.init('./data/capbit.mdb');

// Define capability bits (bitmasks for O(1) evaluation)
const READ = 0x01;
const WRITE = 0x02;
const DELETE = 0x04;
const ADMIN = 0x08;

// "editor" relationship on "project42" grants read+write
capbit.setCapability('project42', 'editor', READ | WRITE);

// "viewer" relationship on "project42" grants read only
capbit.setCapability('project42', 'viewer', READ);

// John is an editor on project42
capbit.setRelationship('john', 'editor', 'project42');

// Bob inherits mary's relationship to project42
capbit.setInheritance('bob', 'project42', 'mary');

// Check access
const caps = capbit.checkAccess('john', 'project42');
const canWrite = (caps & WRITE) !== 0;  // true

// Or use the helper
const hasWrite = capbit.hasCapability('john', 'project42', WRITE);  // true
```

## File Structure

```
capbit/
├── src/
│   ├── lib.rs          # NAPI bindings (thin wrappers)
│   ├── core.rs         # Core database operations
│   └── server.rs       # HTTP server (optional, --features server)
├── tests/
│   └── capbit.test.js  # Vitest tests
├── Cargo.toml          # Rust package config
├── build.rs            # NAPI build script
├── package.json        # Node.js package config
├── index.d.ts          # TypeScript definitions
├── capbit.node         # Native module (generated)
└── data/               # LMDB data directory
```

## Write Strategies

Three strategies for different use cases:

| Strategy | API | Use Case |
|----------|-----|----------|
| Single-op | `setRelationship()` | Simple apps, low write volume |
| WriteBatch | `new WriteBatch()` | Atomicity, controlled batching |
| Batch functions | `batchSetRelationships()` | High-throughput bulk inserts |

**WriteBatch example:**
```javascript
const batch = new capbit.WriteBatch();
batch.setRelationship('john', 'editor', 'doc1');
batch.setCapability('doc1', 'editor', READ | WRITE);
batch.execute(); // Single atomic transaction
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
