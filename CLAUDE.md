# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

## Project Overview

Capbit is a minimal, high-performance access control system where everything is an entity, relationships are bitmasks, and capability semantics are defined per-entity. The system achieves O(log N) lookup and O(1) evaluation with linear scaling, no global schema, and deterministic ordering via epochs.

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
- Bitmasks (relationships and capabilities)
- Epochs (timestamps for ordering)

No types. No schema. Just paths and bits.

### Path Patterns

Six patterns define the entire system:

| Pattern | Purpose |
|---------|---------|
| `entity/rel_mask/entity` | Relationship between entities |
| `entity/policy/entity` | Conditional relationship (code outputs rel_mask) |
| `entity/rel_mask/cap_mask` | Capability definition (per-entity) |
| `entity/entity/entity` | Inheritance reference |
| `entity/rel_mask/label` | Human-readable relationship name |
| `entity/cap_mask/label` | Human-readable capability name |

All paths store **epoch** as value.

### Sub-Databases

```
LMDB
├── relationships/      (entity/rel_mask/entity)
├── relationships_rev/  (reversed for "who has access to X" queries)
├── policies/           (entity/policy/entity)
├── policies_rev/       (reversed)
├── capabilities/       (entity/rel_mask/cap_mask)
├── inheritance/        (entity/entity/entity)
├── inheritance_rev/    (reversed)
└── labels/             (entity/rel_mask/label, entity/cap_mask/label)
```

### Key Concepts

**Relationship**: `john/0x02/slack → epoch`
Entity "john" has relationship bits `0x02` with entity "slack". The system doesn't know john is a user or slack is an app—that's business knowledge.

**Per-Entity Capability Semantics**: The same relationship bit means different things to different entities:
```
slack/0x02/cap_mask → 0x0F    (editor in slack: read, write, delete, admin)
github/0x02/cap_mask → 0x03   (editor in github: read, write only)
```

**Inheritance**: `john/sales/mary → epoch`
John inherits whatever relationship mary has with sales.

**Policies**: Conditional relationships where code evaluates context and outputs a rel_mask:
```
john/policy/slack → epoch
// Policy code returns 0x02 during work hours, 0x01 otherwise
```

### Access Evaluation

Three to four lookups, left to right:

```
Can entity1 perform action on entity2?

Step 1: entity1/*/entity2
→ get all existing rel_masks (direct relationships)
→ includes static (rel_mask) and dynamic (policy)

Step 1b: If policy exists (entity1/policy/entity2)
→ execute policy code with context
→ policy returns rel_mask (or 0x00)

Step 2: entity1/entity2/*
→ if inheritance exists, get entity3
→ do step 1 for entity3 (inherited relationships)

Step 3: entity2/rel_mask/cap_mask
→ for each rel_mask from steps 1, 1b, and 2
→ if rel_mask matches, get capability bits
→ evaluate requested action against capability bits
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
  john/0x02/sales → epoch
  sales/0x02/john → epoch
```

Enables O(log N) queries from either direction:
- "What can john access?" → scan `john/*/*`
- "Who can access sales?" → scan `sales/*/*`

## Usage Example

```javascript
const capbit = require('./capbit.node');

// Initialize
capbit.init('./data/capbit.mdb');

// Define capability semantics for a resource
const READ = 0x01;
const WRITE = 0x02;
const DELETE = 0x04;
const ADMIN = 0x08;

const EDITOR = 0x02;   // relationship mask
const VIEWER = 0x01;   // relationship mask

// "editor" relationship on "project42" grants read+write
capbit.setCapability('project42', EDITOR, READ | WRITE);

// "viewer" relationship on "project42" grants read only
capbit.setCapability('project42', VIEWER, READ);

// John is an editor on project42
capbit.setRelationship('john', EDITOR, 'project42');

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
│   └── lib.rs          # Rust library with NAPI bindings
├── Cargo.toml          # Rust package config
├── build.rs            # NAPI build script
├── package.json        # Node.js package config
├── index.js            # Node.js entry point (generated)
├── capbit.node         # Native module (generated)
└── data/               # LMDB data directory
```

## Design Principles

1. **Type Agnostic**: No types in paths; business layer defines meaning
2. **Linear Scaling**: Bitmasks, not named roles
3. **O(log N) Access**: LMDB B-tree lookups
4. **O(1) Evaluation**: Bitmask AND operation
5. **Per-Entity Semantics**: Each entity defines its own capability mappings
6. **Conditional Access**: Policies output rel_masks based on context
7. **Inheritance**: Path reference, not graph traversal
8. **Deterministic**: Epochs order all operations
9. **ACID**: Transactional forward/reverse writes
10. **Bidirectional**: Query from either entity's perspective
