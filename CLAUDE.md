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
cargo build                                # Build library
cargo build --release                      # Build optimized
cargo test                                 # Run all 190 tests
cargo test -- --nocapture                  # Run with output
cargo test demo_simulation -- --nocapture  # Interactive demo
cargo run --bin capbit-server              # Run REST API server (demo at localhost:3000)
```

## Architecture

### File Structure

```
capbit/
├── src/
│   ├── lib.rs              # Public API re-exports
│   ├── core.rs             # Core database operations
│   ├── caps.rs             # SystemCap constants
│   ├── bootstrap.rs        # Genesis/root creation
│   ├── protected.rs        # Protected mutation API
│   └── bin/
│       └── server.rs       # REST API server
├── demo/
│   └── index.html          # Interactive web demo
├── tests/
│   ├── integration.rs          # v1 compatibility tests
│   ├── attack_vectors.rs       # Security tests (9)
│   ├── attack_vectors_extended.rs  # Advanced security (15)
│   ├── permission_boundaries.rs    # Capability edge cases (16)
│   ├── revocation.rs           # Permission removal (11)
│   ├── authorized_operations.rs    # Client abilities (17)
│   ├── input_validation.rs     # Edge cases (18)
│   ├── inheritance_advanced.rs # Complex inheritance (12)
│   ├── batch_operations.rs     # Batch API (13)
│   ├── query_operations.rs     # Query completeness (15)
│   ├── type_system.rs          # Type lifecycle (19)
│   ├── protected_api.rs        # v2 API tests (23)
│   ├── simulation.rs           # Organization scenarios (2)
│   ├── benchmarks.rs           # Performance tests (7)
│   └── demo_verbose.rs         # Interactive demo (1)
├── Cargo.toml
├── README.md               # User documentation
├── GUIDE.md                # Visual guide with diagrams
├── SIMULATION.md           # Full simulation spec
└── TEST_PLAN.md            # Comprehensive test plan
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
Permission Check Flow (check_access):

1. Direct grants: subject/*/object → get rel_types
2. Type-level grants: subject/*/_type:{type} → get rel_types (for typed entities)
3. Inheritance: subject/object/* → get sources, recurse
4. Capability lookup: object/rel_type → cap_mask
5. Combine: OR all masks together
6. Evaluate: (effective & required) == required

Note: check_access includes type-level permissions when querying instances.
E.g., querying user:root on team:engineering includes root's grants on _type:team.
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
| Security Attacks | 24 | Attack vectors, privilege escalation |
| Permission Boundaries | 16 | Capability edge cases |
| Revocation | 11 | Permission removal, cascade |
| Authorized Operations | 17 | Client abilities (happy path) |
| Input Validation | 18 | Edge cases, special chars |
| Inheritance | 12 | Diamond, wide, deep patterns |
| Batch Operations | 13 | WriteBatch, atomic ops |
| Query Operations | 15 | Query completeness |
| Type System | 19 | Type lifecycle, permissions |
| Protected API | 23 | v2 API authorization |
| Integration | 9 | End-to-end |
| Simulation | 2 | Organization scenarios |
| Benchmarks | 7 | Performance |
| Doc-tests | 3 | Example verification |
| **Total** | **190** | |

## Design Principles

1. **Typed Entities**: All entities are `type:id` format
2. **Protected by Default**: v2 API requires authorization
3. **Type-Level Permissions**: Control entity creation at type level
4. **Bounded Delegation**: Inherited caps never exceed delegator's
5. **Single Bootstrap**: Genesis runs exactly once
6. **Fail Secure**: Missing permission = denied
