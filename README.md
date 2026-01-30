# Capbit

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: Non-Commercial](https://img.shields.io/badge/License-Non--Commercial-red.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-47%20passing-brightgreen.svg)](#testing)

**High-performance access control library for Rust** with typed entities, protected mutations, and bitmask capabilities.

```
Can user:alice edit team:engineering?

  ┌─────────────┐     check_access     ┌─────────────────┐
  │ user:alice  │ ──────────────────►  │ team:engineering│
  │  role: lead │                      │  lead = 0x0030  │
  └─────────────┘                      └─────────────────┘
                         │
                         ▼
                   ✓ ALLOWED (0x0030)
```

---

## Features

| Feature | Description |
|---------|-------------|
| **O(1) Evaluation** | Bitmask AND operations for instant permission checks |
| **O(log N) Lookup** | LMDB B-tree storage for fast lookups |
| **Typed Entities** | `user:alice`, `team:sales`, `app:backend` |
| **Protected Mutations** | All writes require authorization |
| **Per-Entity Semantics** | Each entity defines what roles mean to it |
| **Delegation** | Pass permissions to others (bounded by your own) |
| **47 Security Tests** | Battle-tested against privilege escalation attacks |

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

### 3. Define Roles

```rust
use capbit::{protected, SystemCap};

// Define what "lead" means on this team
protected::set_capability(
    "user:admin",           // who's setting this
    "team:engineering",     // on which entity
    "lead",                 // role name
    SystemCap::GRANT_WRITE | SystemCap::GRANT_READ
)?;

// Define "member" (read-only)
protected::set_capability(
    "user:admin",
    "team:engineering",
    "member",
    SystemCap::GRANT_READ
)?;
```

### 4. Assign Roles

```rust
// Make Bob the team lead
protected::set_grant("user:admin", "user:bob", "lead", "team:engineering")?;

// Bob can now add members (he has GRANT_WRITE)
protected::set_grant("user:bob", "user:alice", "member", "team:engineering")?;
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

### Entity Types

Everything is a typed entity in `type:id` format:

```
user:alice        - A person
team:engineering  - A group
app:backend       - An application
resource:doc123   - Something to protect
_type:user        - Meta-entity for type-level permissions
```

### Permission Model

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

### Capability Bits

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

### Composite Capabilities

```rust
use capbit::SystemCap;

SystemCap::ENTITY_ADMIN   // Full entity management (0x1ffc)
SystemCap::GRANT_ADMIN    // Full grant control (0x0070)
SystemCap::CAP_ADMIN      // Full capability control (0x0380)
SystemCap::DELEGATE_ADMIN // Full delegation control (0x1c00)
SystemCap::ALL            // Everything (0x1fff)
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

## Testing

Run all 47 tests:

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
| Security | 9 | Attack vectors (escalation, spoofing, replay) |
| Bootstrap | 6 | System initialization |
| Entities | 4 | Create, delete, validation |
| Grants | 3 | Relationship management |
| Capabilities | 2 | Role definitions |
| Delegations | 3 | Inheritance system |
| Access | 5 | Permission evaluation |
| Integration | 9 | End-to-end scenarios |
| Simulation | 2 | Full organization scenarios |
| Doc-tests | 3 | Example code verification |

---

## Security

Capbit is designed to prevent common access control vulnerabilities:

| Attack | Protection |
|--------|------------|
| Privilege Escalation | All mutations require capability check |
| Bootstrap Replay | `bootstrap()` only runs once |
| Entity Spoofing | Entities must be created through protected API |
| Delegation Amplification | Inherited caps bounded by delegator's caps |
| Circular Delegation | Depth-limited traversal with cycle detection |
| Scope Confusion | Type-level and entity-level permissions separated |

---

## Examples

### Organization Hierarchy

```rust
// Create structure
protected::create_entity("user:root", "team", "engineering")?;
protected::create_entity("user:root", "user", "alice")?;
protected::create_entity("user:root", "user", "bob")?;

// Define roles
protected::set_capability("user:root", "team:engineering", "lead",
    SystemCap::GRANT_WRITE | SystemCap::GRANT_READ)?;
protected::set_capability("user:root", "team:engineering", "member",
    SystemCap::GRANT_READ)?;

// Assign
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

| Operation | Complexity |
|-----------|------------|
| Key lookup | O(log N) |
| Prefix scan | O(log N + K) |
| Bitmask check | O(1) |
| Access check | O(log N) |

LMDB provides:
- Memory-mapped I/O for fast reads
- ACID transactions
- Zero-copy reads
- Multi-reader concurrency

---

## Documentation

- [**GUIDE.md**](GUIDE.md) - User-friendly guide with diagrams
- [**CLAUDE.md**](CLAUDE.md) - Technical architecture details
- [**SIMULATION.md**](SIMULATION.md) - Full organization simulation

---

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all 47 tests pass: `cargo test`
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
