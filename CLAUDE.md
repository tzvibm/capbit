# CLAUDE.md

Technical guidance for Claude Code when working with this repository.

## Project Overview

Capbit is a Rust library for high-performance access control with:
- **Typed entities**: Format `type:id` (e.g., `user:john`, `team:sales`)
- **Protected mutations**: All writes require authorization
- **Bitmask capabilities**: O(1) permission evaluation
- **Delegation**: Bounded inheritance chains

## Commands

```bash
cargo build            # Build library
cargo build --release  # Build optimized
cargo test             # Run all 47 tests
cargo test -- --nocapture  # Run with output
cargo test demo_simulation -- --nocapture  # Interactive demo
```

## Architecture

### File Structure

```
capbit/
├── src/
│   ├── lib.rs          # Public API re-exports
│   ├── core.rs         # Core database operations
│   ├── caps.rs         # SystemCap constants
│   ├── bootstrap.rs    # Genesis/root creation
│   └── protected.rs    # Protected mutation API
├── tests/
│   ├── integration.rs      # v1 compatibility tests
│   ├── attack_vectors.rs   # Security tests
│   ├── protected_api.rs    # v2 API tests
│   ├── simulation.rs       # Organization scenarios
│   └── demo_verbose.rs     # Interactive demo
├── Cargo.toml
├── README.md           # User documentation
├── GUIDE.md            # Visual guide with diagrams
└── SIMULATION.md       # Full simulation spec
```

### Core Modules

| Module | Purpose |
|--------|---------|
| `core.rs` | LMDB operations, transactions, access checks |
| `caps.rs` | SystemCap bitmask constants |
| `bootstrap.rs` | Genesis: create root user, core types |
| `protected.rs` | Authorization-checked mutations |

### Entity Types

```
user:alice        - Person
team:engineering  - Group
app:backend       - Application
resource:doc123   - Protected resource
_type:user        - Meta-entity for type-level permissions
```

### Storage (LMDB)

```
LMDB
├── relationships/           (subject/rel_type/object → epoch)
├── relationships_rev/       (object/rel_type/subject → epoch)
├── capabilities/            (entity/rel_type → cap_mask)
├── inheritance/             (subject/object/source → epoch)
├── inheritance_by_source/   (source/object/subject → epoch)
├── inheritance_by_object/   (object/source/subject → epoch)
├── cap_labels/              (entity/cap_bit → label)
├── types/                   (type_name → epoch)
├── entities/                (entity_id → epoch)
└── meta/                    (key → value)
```

### Permission Model

```
Permission Check Flow:

1. Direct grants: subject/*/object → get rel_types
2. Inheritance: subject/object/* → get sources, recurse
3. Capability lookup: object/rel_type → cap_mask
4. Combine: OR all masks together
5. Evaluate: (effective & required) == required
```

### SystemCap Bits

| Capability | Hex | Purpose |
|------------|-----|---------|
| TYPE_CREATE | 0x0001 | Create types |
| TYPE_DELETE | 0x0002 | Delete types |
| ENTITY_CREATE | 0x0004 | Create entities |
| ENTITY_DELETE | 0x0008 | Delete entities |
| GRANT_READ | 0x0010 | View relationships |
| GRANT_WRITE | 0x0020 | Create relationships |
| GRANT_DELETE | 0x0040 | Remove relationships |
| CAP_READ | 0x0080 | View capabilities |
| CAP_WRITE | 0x0100 | Define capabilities |
| CAP_DELETE | 0x0200 | Remove capabilities |
| DELEGATE_READ | 0x0400 | View delegations |
| DELEGATE_WRITE | 0x0800 | Create delegations |
| DELEGATE_DELETE | 0x1000 | Remove delegations |

Composites:
- `ENTITY_ADMIN` = 0x1ffc (full entity management)
- `GRANT_ADMIN` = 0x0070 (full grant control)
- `TYPE_ADMIN` = 0x1fff (everything)

### Protected API Pattern

All mutations in `protected.rs` follow:

```rust
pub fn set_grant(actor: &str, seeker: &str, relation: &str, scope: &str) -> Result<u64> {
    // 1. Check actor has required capability on scope (or _type:*)
    check_permission(actor, scope, SystemCap::GRANT_WRITE)?;

    // 2. Execute within transaction
    with_write_txn_pub(|txn, dbs| {
        // 3. Validate scope exists
        // 4. Perform operation
        _set_relationship_in(txn, dbs, seeker, relation, scope)
    })
}
```

### Bootstrap Sequence

```rust
bootstrap("root"):
  1. Create meta-type "_type"
  2. Create core types: user, team, app, resource
  3. Create type entities: _type:_type, _type:user, etc.
  4. Define admin capability on each type
  5. Create user:root entity
  6. Grant root "admin" on all type entities
  7. Mark system as bootstrapped
```

## Complexity

| Operation | Complexity |
|-----------|------------|
| Key lookup | O(log N) |
| Prefix scan | O(log N + K) |
| Bitmask check | O(1) |
| Access check | O(log N) |

## Test Categories

| Category | Count | Purpose |
|----------|-------|---------|
| Security | 9 | Attack vectors |
| Bootstrap | 6 | Initialization |
| Entities | 4 | CRUD operations |
| Grants | 3 | Relationships |
| Capabilities | 2 | Role definitions |
| Delegations | 3 | Inheritance |
| Access | 5 | Permission checks |
| Integration | 9 | End-to-end |
| Simulation | 2 | Organization scenarios |
| Doc-tests | 3 | Example verification |
| **Total** | **47** | |

## Design Principles

1. **Typed Entities**: All entities are `type:id` format
2. **Protected by Default**: v2 API requires authorization
3. **Type-Level Permissions**: Control entity creation at type level
4. **Bounded Delegation**: Inherited caps never exceed delegator's
5. **Single Bootstrap**: Genesis runs exactly once
6. **Fail Secure**: Missing permission = denied
