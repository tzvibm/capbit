# Capbit

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: Non-Commercial](https://img.shields.io/badge/License-Non--Commercial-red.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-192%20passing-brightgreen.svg)](#testing)

**High-performance access control library for Rust** with typed entities, protected mutations, and bitmask capabilities.

```
Can user:alice access resource:office?

  ┌─────────────┐     check_access     ┌─────────────────┐
  │ user:alice  │ ──────────────────►  │ resource:office │
  │role: employee│                     │ employee = 0x07 │
  └─────────────┘                      └─────────────────┘
                         │
                         ▼
                   ✓ ALLOWED (0x07)

  (bit0=enter, bit1=printer, bit2=fax)
```

---

## Features

| Feature | Description |
|---------|-------------|
| **O(1) Bitmask Eval** | Final permission check is a single AND operation |
| **O(log N) Lookup** | LMDB B-tree storage for fast lookups |
| **O(k × log N) Total** | Full check: k relations × log N lookup each |
| **Typed Entities** | `user:alice`, `team:sales`, `app:backend` |
| **Protected Mutations** | All writes require authorization |
| **Per-Entity Semantics** | Each entity defines what relation names mean (via capabilities) |
| **Delegation** | Pass permissions to others (bounded by your own) |
| **Two-Layer Capability Model** | System capabilities for type-level, org-defined for entity-level |
| **192 Tests** | Comprehensive security, permission, and integration tests |
| **REST API Demo** | Interactive web demo with full CRUD operations |

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
capbit = "0.2"
```

---

## Quick Start

### 1. Bootstrap the System

```rust
use capbit::{init, bootstrap};

fn main() -> capbit::Result<()> {
    // Initialize database
    init("./data/capbit.mdb")?;

    // Bootstrap creates root user with full admin powers
    // This only works once - prevents attackers from re-bootstrapping
    bootstrap("admin")?;

    Ok(())
}
```

### 2. Create Your Organization

```rust
use capbit::protected;

// Root creates teams
protected::create_entity("user:admin", "team", "engineering")?;
protected::create_entity("user:admin", "team", "sales")?;

// Root creates users
protected::create_entity("user:admin", "user", "alice")?;
protected::create_entity("user:admin", "user", "bob")?;
```

### 3. Define Capabilities

Capbit has a **two-layer capability model**:

1. **System Capabilities (SystemCap)** - Used on `_type:*` scopes only. Control who can create entities, define grants, etc.
2. **Org-Defined Capabilities** - Arbitrary bitmasks where YOUR organization defines the meaning per entity.

**Key concepts:**
- **Entities** = Things (`user:alice`, `resource:office`, `team:sales`)
- **Capabilities** = Define what relation names MEAN on an entity (maps relation name → bitmask)
- **Grants** = Business rules that assign relations to seekers (these ARE the role assignments!)

```rust
use capbit::protected;

// ═══════════════════════════════════════════════════════════
// CAPABILITIES: Define what relation names mean on this entity
// The BITS are primitives: bit0=enter, bit1=print, bit2=fax, etc.
// ═══════════════════════════════════════════════════════════

// On resource:office, define what each relation name means:
protected::set_capability("user:admin", "resource:office", "visitor",  0x01)?;  // bit0 only
protected::set_capability("user:admin", "resource:office", "employee", 0x07)?;  // bits 0-2
protected::set_capability("user:admin", "resource:office", "manager",  0x0F)?;  // bits 0-3
protected::set_capability("user:admin", "resource:office", "full-access", 0x3F)?;  // all bits
```

**Key insight**: The BITS within bitmasks are the primitives (atomic actions). Capabilities map relation NAMES to combinations of those bits. The same bitmask value (e.g., 0x01) means different things on different entities.

### 4. Create Grants (Business Rules)

Grants ARE the role assignments. They assign relations (with their capability bitmasks) to seekers.

```rust
// Grant Bob the "employee" relation on the office
// This is the business rule: "Bob has employee access to the office"
protected::set_grant("user:admin", "user:bob", "employee", "resource:office")?;

// Bob now has capabilities 0x07 (bits 0-2) on resource:office
// because "employee" was defined as 0x07 in step 3

// Alice can grant others (if she has a relation with GRANT_WRITE bit)
protected::set_grant("user:alice", "user:charlie", "visitor", "resource:office")?;
```

### 5. Check Permissions

```rust
use capbit::{check_access, has_capability, SystemCap};

// Get all of Bob's permissions on the team
let caps = check_access("user:bob", "team:engineering", None)?;
println!("Bob has: 0x{:04x}", caps);  // 0x0030

// Check specific permission
if has_capability("user:bob", "team:engineering", SystemCap::GRANT_WRITE)? {
    println!("Bob can add team members!");
}
```

---

## Architecture

### Core Concepts

```
┌─────────────────────────────────────────────────────────────────────┐
│                         CAPBIT MODEL                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   ENTITIES = Things                                                  │
│   ─────────────────                                                  │
│   user:alice, team:sales, resource:office, app:api                  │
│                                                                      │
│   CAPABILITIES = What relation names MEAN on an entity              │
│   ──────────────────────────────────────────────────                │
│   On resource:office: "visitor"=0x01, "employee"=0x07               │
│   The BITS are primitives (bit0=enter, bit1=print, etc.)            │
│                                                                      │
│   GRANTS = Business rules assigning relations to seekers            │
│   ──────────────────────────────────────────────────                │
│   "user:alice has 'employee' on resource:office"                    │
│   Grants ARE the role assignments!                                   │
│                                                                      │
│   DELEGATIONS = Inherited grants                                     │
│   ─────────────────────────────                                      │
│   "user:bob inherits from user:alice on resource:office"            │
│   Bob gets Alice's capabilities (bounded by what Alice has)         │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Entity Types

Everything is a typed entity in `type:id` format:

```
user:alice        - A person
team:engineering  - A group
app:backend       - An application
resource:doc123   - Something to protect
_type:user        - Meta-entity for type-level permissions
```

### Permission Check Flow

```
                    ┌──────────────────────────────────────┐
                    │           Permission Check           │
                    └──────────────────────────────────────┘
                                      │
                    ┌─────────────────┴─────────────────┐
                    ▼                                   ▼
            ┌──────────────┐                   ┌──────────────┐
            │    Direct    │                   │  Inherited   │
            │    Grants    │                   │ (Delegation) │
            └──────────────┘                   └──────────────┘
                    │                                   │
                    └─────────────────┬─────────────────┘
                                      ▼
                            ┌──────────────────┐
                            │ Look up capability│
                            │ for each relation │
                            └──────────────────┘
                                      │
                                      ▼
                            ┌──────────────────┐
                            │   OR together    │
                            │  all cap masks   │
                            └──────────────────┘
                                      │
                                      ▼
                            ┌──────────────────┐
                            │ Final capability │
                            │     bitmask      │
                            └──────────────────┘
```

### Two-Layer Capability Model

```
┌─────────────────────────────────────────────────────────────────────┐
│                    CAPBIT CAPABILITY MODEL                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│   LAYER 1: System Capabilities (SystemCap)                          │
│   ─────────────────────────────────────────                         │
│   Scope: _type:* only (e.g., _type:user, _type:team)                │
│   Purpose: Control who can create entities, define grants, etc.     │
│   Protected: Only root and delegatees can modify                    │
│                                                                     │
│   LAYER 2: Org-Defined Capabilities                                 │
│   ─────────────────────────────────────                             │
│   Scope: Any non-_type entity (resource:office, team:sales, etc.)   │
│   Purpose: Whatever YOUR organization decides                       │
│   Flexible: 64 bits, you define the meaning per entity              │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### System Capabilities (Layer 1)

**Only meaningful on `_type:*` scopes.** Used by the protected API to authorize system operations.

| Capability | Hex | Description |
|------------|-----|-------------|
| `TYPE_CREATE` | 0x0001 | Create new entity types |
| `TYPE_DELETE` | 0x0002 | Delete entity types |
| `ENTITY_CREATE` | 0x0004 | Create entities of a type |
| `ENTITY_DELETE` | 0x0008 | Delete entities |
| `GRANT_READ` | 0x0010 | View relationships |
| `GRANT_WRITE` | 0x0020 | Create relationships |
| `GRANT_DELETE` | 0x0040 | Remove relationships |
| `CAP_READ` | 0x0080 | View capability definitions |
| `CAP_WRITE` | 0x0100 | Define capabilities |
| `CAP_DELETE` | 0x0200 | Remove capability definitions |
| `DELEGATE_READ` | 0x0400 | View delegations |
| `DELEGATE_WRITE` | 0x0800 | Create delegations |
| `DELEGATE_DELETE` | 0x1000 | Remove delegations |

**Composites:**
```rust
SystemCap::ENTITY_ADMIN   // 0x1ffc - Full entity management
SystemCap::GRANT_ADMIN    // 0x0070 - Full grant control
SystemCap::TYPE_ADMIN     // 0x1fff - Everything
```

### Org-Defined Capabilities (Layer 2)

**You define what each bit means for YOUR entities:**

```rust
// For resource:office - you decide:
// bit0 = enter building
// bit1 = use printer
// bit2 = use fax

// For resource:database - completely different meanings:
// bit0 = read
// bit1 = write
// bit2 = delete
// bit3 = admin

// The bitmask 0x01 means "enter" on office, but "read" on database!
```

### Scope Isolation (Security)

**Having SystemCap values on your own entity does NOT grant system powers:**

```rust
// Alice has 0x1fff (all bits) on resource:alice-doc
// This does NOT let her create users!
// Why? The protected API checks capabilities on _type:user, not on resource:alice-doc

protected::create_entity("user:alice", "user", "hacked")  // DENIED!
// Alice has 0x0000 on _type:user, so she can't create users
```

---

## API Reference

### Initialization

```rust
init(db_path: &str) -> Result<()>      // Initialize database
bootstrap(root_id: &str) -> Result<u64> // Create root user (once only)
is_bootstrapped() -> Result<bool>       // Check if system is bootstrapped
get_root_entity() -> Result<Option<String>> // Get root entity ID
```

### Protected API (v2)

All mutations require authorization:

```rust
use capbit::protected;

// Entities
protected::create_entity(actor, entity_type, id) -> Result<u64>
protected::delete_entity(actor, entity_id) -> Result<bool>

// Grants (relationships)
protected::set_grant(actor, seeker, relation, scope) -> Result<u64>
protected::delete_grant(actor, seeker, relation, scope) -> Result<bool>

// Capabilities
protected::set_capability(actor, scope, relation, cap_mask) -> Result<u64>

// Delegations
protected::set_delegation(actor, seeker, scope, delegate) -> Result<u64>
protected::delete_delegation(actor, seeker, scope, delegate) -> Result<bool>

// Types
protected::create_type(actor, type_name) -> Result<u64>
```

### Access Checks (Read-only)

```rust
check_access(subject, object, max_depth) -> Result<u64>
has_capability(subject, object, required_cap) -> Result<bool>
entity_exists(entity_id) -> Result<bool>
type_exists(type_name) -> Result<bool>
```

### Query Operations

```rust
list_accessible(subject) -> Result<Vec<(String, String)>>  // What can X access?
list_subjects(object) -> Result<Vec<(String, String)>>     // Who can access X?
get_relationships(subject, object) -> Result<Vec<String>>  // Get relation types
```

### Legacy API (v1 Compatibility)

Unprotected operations for simple use cases:

```rust
set_relationship(subject, rel_type, object) -> Result<u64>
delete_relationship(subject, rel_type, object) -> Result<bool>
set_capability(entity, rel_type, cap_mask) -> Result<u64>
set_inheritance(subject, object, source) -> Result<u64>
```

---

## Delegation

Delegation allows passing permissions without copying them:

```rust
// Alice has edit access to doc
protected::set_grant("user:admin", "user:alice", "editor", "resource:doc")?;

// Alice can delegate (needs DELEGATE_WRITE)
protected::set_capability("user:admin", "resource:doc", "owner", SystemCap::DELEGATE_WRITE)?;
protected::set_grant("user:admin", "user:alice", "owner", "resource:doc")?;

// Alice delegates to Bob
protected::set_delegation("user:alice", "user:bob", "resource:doc", "user:alice")?;

// Bob now inherits Alice's permissions (bounded by what Alice has)
let bob_caps = check_access("user:bob", "resource:doc", None)?;
```

**Security**: Bob can never have more permissions than Alice. If Alice loses access, Bob loses it too.

---

## Storage

LMDB databases with optimized indexes:

| Database | Key Pattern | Purpose |
|----------|-------------|---------|
| `relationships` | subject/rel_type/object | Forward lookup |
| `relationships_rev` | object/rel_type/subject | Reverse lookup |
| `capabilities` | entity/rel_type | Capability definitions |
| `inheritance` | subject/object/source | Delegation chains |
| `inheritance_by_source` | source/object/subject | "Who inherits from X?" |
| `inheritance_by_object` | object/source/subject | Audit queries |
| `types` | type_name | Type registry |
| `entities` | entity_id | Entity registry |
| `meta` | key | System metadata |

---

## Demo

Capbit includes a REST API server with an interactive web demo.

### Running the Demo

```bash
# Start the server (serves demo at http://localhost:3000)
cargo run --bin capbit-server

# Or build release version
cargo build --release --bin capbit-server
./target/release/capbit-server
```

### Demo Features

- **Bootstrap** - Initialize the system with a root user
- **Entity Management** - Create users, teams, apps, resources
- **Capability Definitions** - Define what relation names mean with 16-bit selector
- **Grant Management** - Create grants (business rules assigning relations)
- **Access Checks** - Test permission queries with dynamic dropdown
- **Templates** - Pre-built scenarios showing entities, capabilities, and grants:
  - **Startup Template**: Office access control (enter, printer, fax, safe, server room)
  - **Org Hierarchy Template**: Team-based permissions

**Demo UI Features:**
- **Bit Selector**: Click individual bits (0-15) to define capabilities visually
- **Dynamic Dropdowns**: Entity/grant/capability dropdowns populated from database
- **Filtered View**: `_type:*` system entities hidden from normal UI (system-level)
- **Copy Logs**: One-click copy of operation logs
- **Auto-Reset**: Templates detect bootstrapped state and offer reset

### REST API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/bootstrap` | Initialize system with root user |
| GET | `/status` | Get system status |
| POST | `/entity` | Create entity |
| GET | `/entities` | List all entities |
| POST | `/capability` | Define capability |
| GET | `/capabilities` | List capabilities |
| POST | `/grant` | Create grant |
| GET | `/grants` | List grants |
| POST | `/check` | Check access permissions |
| POST | `/reset` | Reset database (dev only) |

---

## Testing

Run all 192 tests:

```bash
cargo test
```

Run with output:

```bash
cargo test -- --nocapture
```

Run the interactive demo:

```bash
cargo test demo_simulation -- --nocapture
```

### Test Coverage

| Category | Tests | Description |
|----------|-------|-------------|
| Security Attacks | 26 | Attack vectors, privilege escalation, scope isolation |
| Permission Boundaries | 16 | Capability edge cases, exact matching |
| Revocation | 11 | Permission removal, cascade effects |
| Authorized Operations | 17 | Client abilities (happy path verification) |
| Input Validation | 18 | Edge cases, special characters, limits |
| Inheritance | 12 | Diamond patterns, wide/deep inheritance |
| Batch Operations | 13 | Atomic batch processing, WriteBatch API |
| Query Operations | 15 | list_accessible, list_subjects completeness |
| Type System | 19 | Type lifecycle, custom types, permissions |
| Protected API | 23 | v2 API authorization |
| Integration | 9 | End-to-end scenarios |
| Simulation | 2 | Full organization scenarios |
| Benchmarks | 7 | Performance verification |
| Doc-tests | 3 | Example code verification |
| **Total** | **192** | |

---

## Security

Capbit is designed to prevent common access control vulnerabilities:

| Attack | Protection |
|--------|------------|
| Privilege Escalation | All mutations require capability check on correct scope |
| Bootstrap Replay | `bootstrap()` only runs once |
| Entity Spoofing | Entities must be created through protected API |
| Delegation Amplification | Inherited caps bounded by delegator's caps |
| Circular Delegation | Depth-limited traversal with cycle detection |
| Scope Confusion | Type-level (`_type:*`) and entity-level permissions are isolated |
| Fake SystemCap | Having SystemCap values on org entities grants NO system powers |

### Scope Isolation Security Model

The protected API checks capabilities on the **correct scope**, not just any scope:

```rust
// ATTACK: Alice gives herself all SystemCap bits on her own resource
protected::set_capability("user:root", "resource:alice-doc", "owner", 0x1FFF)?;
protected::set_grant("user:root", "user:alice", "owner", "resource:alice-doc")?;

// Alice now has 0x1FFF (all bits) on resource:alice-doc
// Can she create users? NO!

protected::create_entity("user:alice", "user", "hacked")  // DENIED!
// Why? create_entity checks capabilities on _type:user, not resource:alice-doc
// Alice has 0x0000 on _type:user

// This is proven by tests:
// - attack_fake_systemcap_bitmask
// - verify_root_grants_protected
```

---

## Examples

### Organization Hierarchy

```rust
// Create entities (things)
protected::create_entity("user:root", "team", "engineering")?;
protected::create_entity("user:root", "user", "alice")?;
protected::create_entity("user:root", "user", "bob")?;

// Define capabilities (what relation names mean on this entity)
protected::set_capability("user:root", "team:engineering", "lead",
    SystemCap::GRANT_WRITE | SystemCap::GRANT_READ)?;
protected::set_capability("user:root", "team:engineering", "member",
    SystemCap::GRANT_READ)?;

// Create grants (business rules - these ARE the role assignments!)
protected::set_grant("user:root", "user:alice", "lead", "team:engineering")?;
protected::set_grant("user:alice", "user:bob", "member", "team:engineering")?;
```

### Resource Access Control

```rust
// Create resource
protected::create_entity("user:root", "resource", "secret-doc")?;

// Define access levels
protected::set_capability("user:root", "resource:secret-doc", "viewer", 0x01)?;
protected::set_capability("user:root", "resource:secret-doc", "editor", 0x03)?;
protected::set_capability("user:root", "resource:secret-doc", "admin", 0x0F)?;

// Grant access
protected::set_grant("user:root", "user:alice", "editor", "resource:secret-doc")?;

// Check
assert!(has_capability("user:alice", "resource:secret-doc", 0x02)?); // Can write
assert!(!has_capability("user:alice", "resource:secret-doc", 0x08)?); // Can't admin
```

### Delegated Administration

```rust
// HR team manages users
protected::set_grant("user:root", "team:hr", "admin", "_type:user")?;

// Alice (HR lead) inherits team permissions
protected::set_capability("user:root", "_type:user", "delegator", SystemCap::DELEGATE_WRITE)?;
protected::set_grant("user:root", "user:root", "delegator", "_type:user")?;
protected::set_delegation("user:root", "user:alice", "_type:user", "team:hr")?;

// Now Alice can create users
protected::create_entity("user:alice", "user", "new-hire")?;
```

---

## Performance

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Key lookup | O(log N) | LMDB B-tree |
| Prefix scan | O(log N + K) | K = matching keys |
| Bitmask check | O(1) | Single AND operation |
| Full access check | O(k × log N) | k = user's relations on target |
| With inheritance | O(d × k × log N) | d = delegation depth |

**Measured performance** (ARM64/Android):
- Single permission check: 7-8 μs
- With 3-level inheritance: ~25 μs

Run benchmarks: `cargo test benchmark_ -- --nocapture`

LMDB provides:
- Memory-mapped I/O for fast reads
- ACID transactions
- Zero-copy reads
- Multi-reader concurrency

---

## Documentation

- [**docs/GUIDE.md**](docs/GUIDE.md) - User-friendly guide with diagrams
- [**CLAUDE.md**](CLAUDE.md) - Technical architecture details (for Claude Code)
- [**docs/SIMULATION.md**](docs/SIMULATION.md) - Full organization simulation
- [**docs/V3_ROADMAP.md**](docs/V3_ROADMAP.md) - Future features roadmap
- [**docs/TEST_PLAN.md**](docs/TEST_PLAN.md) - Comprehensive test plan

---

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all 192 tests pass: `cargo test`
5. Submit a pull request

---

## License

**Non-Commercial Open Source License** - see [LICENSE](LICENSE) for details.

**Allowed:**
- Personal use
- Educational use
- Academic research
- Non-profit organizations
- Open source projects (with attribution)

**Not Allowed (without commercial license):**
- Corporate/business use
- Commercial products or services

For commercial licensing, contact: https://github.com/tzvibm

---

## Changelog

### v2.2.0
- **Clarified conceptual model**
  - Entities = things (user:alice, resource:office)
  - Capabilities = what relation names mean on an entity (bitmasks)
  - Grants = business rules assigning relations to seekers (role assignments!)
- **Two-layer capability model**
  - System capabilities (SystemCap) for `_type:*` scopes only
  - Org-defined capabilities with flexible 64-bit bitmasks
- **Scope isolation security model** with tests proving SystemCap values on org entities don't grant system powers
- **Enhanced demo UI**
  - 16-bit clickable selector for defining capabilities
  - Dynamic dropdowns populated from database
  - Filtered view hiding `_type:*` system entities
  - Delegation support with inherited grants display
  - Templates demonstrating entities, capabilities, grants, and delegations
- **New security tests**: `attack_fake_systemcap_bitmask`, `verify_root_grants_protected`
- **192 tests** total (up from 190)

### v2.1.0
- REST API server with interactive web demo
- Type-level permissions now included in `check_access` queries
- Comprehensive test suite: 190 tests (up from 47)
  - Security attack vectors
  - Permission boundary testing
  - Revocation and cascade effects
  - Client ability verification
  - Input validation edge cases
  - Advanced inheritance patterns
  - Batch operation tests
  - Query completeness tests
  - Type system tests

### v2.0.0
- Protected mutations requiring authorization
- Typed entities (`type:id` format)
- Bootstrap/genesis system
- Delegation with bounded inheritance
- 47 security and functionality tests

### v1.0.0
- Initial release
- Basic relationships and capabilities
- LMDB storage
- Inheritance system
