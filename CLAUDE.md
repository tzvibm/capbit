# CLAUDE.md

Technical guidance for Claude Code when working with this repository.

## Project Overview

Capbit is a Rust library for high-performance access control with:
- **Entities**: Things/resources in format `type:id` (e.g., `user:john`, `resource:office`)
- **Capabilities**: For org entities, ROLES that bundle primitive actions (e.g., `employee`=0x07)
- **Grants**: Assign ONE role → user gets SET of actions
- **Delegations**: Inherited grants (bounded by delegator's capabilities)
- **Protected mutations**: All writes require authorization
- **Bitmask evaluation**: O(1) permission check via single AND operation

**Key distinction:** System grants on `_type:*` are one-to-one (each SystemCap = one action). Org grants assign roles (sets of actions).

## Commands

```bash
cargo build                                # Build library
cargo build --release                      # Build optimized
cargo test                                 # Run all 192 tests
cargo test -- --nocapture                  # Run with output
cargo test demo_simulation -- --nocapture  # Interactive demo
```

### Running the Server

**IMPORTANT for Claude CLI:** Use the management script to run the server:

```bash
./scripts/server.sh start    # Start server (background, port 3000)
./scripts/server.sh stop     # Stop server
./scripts/server.sh status   # Check if running
./scripts/server.sh logs     # View server logs
PORT=3001 ./scripts/server.sh start  # Use different port
```

Do NOT use `cargo run` directly - it won't work properly in Claude CLI's environment.
The script handles process detachment and logging correctly for Termux.

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
├── docs/
│   ├── GUIDE.md            # Visual guide with diagrams
│   ├── SIMULATION.md       # Full simulation spec
│   ├── TEST_PLAN.md        # Comprehensive test plan
│   ├── V3_ROADMAP.md       # Future features roadmap
│   └── COMPARISON.md       # Comparison with other systems
├── tests/
│   ├── integration.rs          # v1 compatibility tests
│   ├── attack_vectors.rs       # Security tests
│   ├── attack_vectors_extended.rs  # Advanced security
│   ├── permission_boundaries.rs    # Capability edge cases
│   ├── revocation.rs           # Permission removal
│   ├── authorized_operations.rs    # Client abilities
│   ├── input_validation.rs     # Edge cases
│   ├── inheritance_advanced.rs # Complex inheritance
│   ├── batch_operations.rs     # Batch API
│   ├── query_operations.rs     # Query completeness
│   ├── type_system.rs          # Type lifecycle
│   ├── protected_api.rs        # v2 API tests
│   ├── simulation.rs           # Organization scenarios
│   ├── benchmarks.rs           # Performance tests
│   └── demo_verbose.rs         # Interactive demo
├── Cargo.toml
├── README.md               # User documentation
└── CLAUDE.md               # Technical guidance for Claude Code
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

### Core Model

```
ENTITIES = Things/resources (user:alice, resource:office, team:sales)
CAPABILITIES = For org entities: ROLES (sets of primitive actions)
               For _type:*: SystemCap bits (one-to-one)
GRANTS = Assign ONE role → get SET of actions
DELEGATIONS = Inherited grants (bounded by delegator)

Example: Roles on resource:office
  "visitor"  = 0x01 (enter only)
  "employee" = 0x07 (enter + print + fax)
  "owner"    = 0x3F (all actions)

Grant bob "employee" → bob gets 0x07 (one grant, multiple actions)
```

### Permission Check Flow (check_access)

```
1. Find grants: subject/*/object → get relation names
2. Type-level grants: subject/*/_type:{type} → get relation names
3. Inheritance: subject/object/* → get sources, recurse for inherited grants
4. Capability lookup: For each relation, get cap_mask from object's capabilities
5. Combine: OR all cap_masks together
6. Evaluate: (effective & required) == required

Note: check_access includes type-level grants when querying instances.
E.g., querying user:root on team:engineering includes root's grants on _type:team.
```

### Two-Layer Capability Model

**Layer 1: System Capabilities (SystemCap)** - Only meaningful on `_type:*` scopes
**Layer 2: Org-Defined Capabilities** - Arbitrary bitmasks, org defines meaning per entity

### SystemCap Bits (Layer 1 - for `_type:*` scopes only)

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

### Org-Defined Capabilities (Layer 2)

On non-`_type:*` entities, capabilities are ROLES (sets of primitive actions):
```
resource:office (primitives: enter=0x01, print=0x02, fax=0x04, safe=0x08):
  "visitor"  = 0x01  (enter only)
  "employee" = 0x07  (enter + print + fax)
  "manager"  = 0x0F  (+ safe)
  "owner"    = 0x3F  (all)

app:api-gateway (primitives: read=0x01, write=0x02, delete=0x04, bulk=0x08):
  "basic"      = 0x03  (read + write)
  "pro"        = 0x1F  (+ delete + bulk + webhooks)
  "enterprise" = 0xFF  (all)
```

### Scope Isolation Security

Having SystemCap values on your entity does NOT grant system powers:
```rust
// Alice has 0x1fff on resource:doc - can she create users? NO!
// create_entity checks capabilities on _type:user, not resource:doc
```

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
| Security Attacks | 26 | Attack vectors, privilege escalation, scope isolation |
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
| **Total** | **192** | |

## Design Principles

1. **Entities are Things**: `type:id` format (user:alice, resource:office)
2. **Capabilities are Actions**: Single bits (enter=0x01, print=0x02, fax=0x04)
3. **Grants are Sets of Actions**: Assign capabilities to users, accumulate via OR
4. **Protected by Default**: v2 API requires authorization
5. **Type-Level Permissions**: Control entity creation at `_type:*` level
6. **Bounded Delegation**: Inherited grants never exceed delegator's
7. **Single Bootstrap**: Genesis runs exactly once
8. **Fail Secure**: Missing permission = denied
