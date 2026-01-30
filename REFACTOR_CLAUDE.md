# CLAUDE.md (Refactored)

This file provides guidance to Claude Code when working with this repository.

## Project Overview

Capbit is a pure Rust library for high-performance access control with:
- **Typed entities**: All entities have format `type:id` (e.g., `user:john`, `team:sales`)
- **Entity types as mutation control**: Types themselves are scopes for permission checks
- **String relations**: Human-readable connectors ("editor", "lead", "member")
- **Bitmask capabilities**: O(1) permission evaluation
- **Genesis bootstrap**: System starts with root user who delegates authority

## Commands

```bash
cargo build            # Build library
cargo build --release  # Build optimized
cargo test             # Run all tests
cargo doc --open       # Generate and view documentation
```

## Core Concepts

### Terminology

| Term | Description | Example |
|------|-------------|---------|
| **Entity** | Anything in the system with typed ID | `user:alice`, `team:hr`, `app:slack` |
| **Type** | Category of entities, controls creation | `user`, `team`, `app`, `resource` |
| **Seeker** | Entity requesting access | `user:alice` wants to access something |
| **Scope** | Entity being accessed | `team:sales` is being accessed |
| **Relation** | Named connection between entities | "lead", "member", "owner", "viewer" |
| **Capability** | Bitmask defining permissions | `0x0030` = GRANT_WRITE \| GRANT_READ |
| **Delegate** | Entity whose permissions are inherited | `team:hr` delegates to `user:alice` |

### Entity ID Format

All entities use `type:identifier` format:

```
user:john              # User named john
user:auth0|abc123      # User from external provider
team:engineering       # Team entity
app:backend-api        # Application entity
resource:doc-123       # Resource entity
_type:user             # Meta-entity: the user type itself
```

### Types Control Mutation

Entity types are themselves entities that control who can create/delete entities of that type:

```
_type:_type    → controls creation of new types (meta)
_type:user     → controls creation of user entities
_type:team     → controls creation of team entities
_type:app      → controls creation of app entities
```

To create `user:frank`, requester must have `ENTITY_CREATE` capability on `_type:user`.

## Architecture

**Stack:** Pure Rust + LMDB (via heed)

### Database Schema

```
LMDB Environment
│
│  # Registry (enforces uniqueness)
├── types/                 type_name → TypeMeta { creator, epoch }
├── entities/              type:id → EntityMeta { creator, epoch }
│
│  # Grants (who has what relation to whom)
├── grants/                seeker/relation/scope → epoch
├── grants_rev/            scope/relation/seeker → epoch
│
│  # Capabilities (what relations mean on each scope)
├── capabilities/          scope/relation → capability (u64)
│
│  # Policies (conditional access)
├── policies/              policy_id → policy_json
├── seeker_policies/       seeker → policy_id (pre-flight)
├── grant_policies/        scope/relation → policy_id
│
│  # Delegations (inheritance)
├── delegations/           seeker/scope/delegate → epoch
├── delegations_by_del/    delegate/scope/seeker → epoch
├── delegations_by_scope/  scope/delegate/seeker → epoch
│
│  # Labels & System
├── cap_labels/            scope/cap_bit → label
├── audit_log/             epoch → audit_entry_json
└── meta/                  key → value
```

### System Capabilities

```rust
pub mod SystemCap {
    // Type management (on _type:_type)
    pub const TYPE_CREATE: u64     = 0x0001;
    pub const TYPE_DELETE: u64     = 0x0002;

    // Entity management (on _type:{type})
    pub const ENTITY_CREATE: u64   = 0x0004;
    pub const ENTITY_DELETE: u64   = 0x0008;

    // Grants
    pub const GRANT_READ: u64      = 0x0010;
    pub const GRANT_WRITE: u64     = 0x0020;
    pub const GRANT_DELETE: u64    = 0x0040;

    // Capabilities
    pub const CAP_READ: u64        = 0x0080;
    pub const CAP_WRITE: u64       = 0x0100;
    pub const CAP_DELETE: u64      = 0x0200;

    // Delegations
    pub const DELEGATE_READ: u64   = 0x0400;
    pub const DELEGATE_WRITE: u64  = 0x0800;
    pub const DELEGATE_DELETE: u64 = 0x1000;

    // Policies
    pub const POLICY_READ: u64     = 0x2000;
    pub const POLICY_WRITE: u64    = 0x4000;
    pub const POLICY_DELETE: u64   = 0x8000;

    // System
    pub const AUDIT_READ: u64      = 0x10000;
    pub const SYSTEM_ADMIN: u64    = 0x20000;
}
```

### Access Evaluation

```
Can seeker perform action on scope?

1. Pre-flight: Check seeker_policies[seeker]
   → If policy fails, DENY immediately

2. Direct grants: grants[seeker/*/scope]
   → Collect all relations seeker has on scope

3. Delegations: delegations[seeker/scope/*]
   → For each delegate, recurse to step 2
   → Inherit delegate's relations on scope

4. Capabilities: capabilities[scope/relation]
   → For each relation found, get capability bits
   → OR all capabilities together

5. Policy check: grant_policies[scope/relation]
   → Filter out relations where policy fails

6. Return effective capability mask
```

### Complexity

| Operation | Complexity |
|-----------|------------|
| Key lookup | O(log N) via B-tree |
| Prefix scan | O(log N + K), K = results |
| Bitmask evaluation | O(1) |
| Access check | O(log N × D), D = delegation depth |

## Bootstrap (Genesis)

System starts empty. Bootstrap creates root user with full authority:

```rust
bootstrap("root") // Creates:

// 1. Types
types/: _type, user, team, app, resource

// 2. Type entities (for permission control)
entities/: _type:_type, _type:user, _type:team, _type:app, _type:resource

// 3. Root user
entities/: user:root

// 4. Capabilities on type entities
capabilities/:
  _type:_type/admin → TYPE_CREATE | TYPE_DELETE
  _type:user/admin  → ENTITY_CREATE | ENTITY_DELETE
  _type:team/admin  → ENTITY_CREATE | ENTITY_DELETE
  ...

// 5. Root gets admin on all types
grants/:
  user:root/admin/_type:_type
  user:root/admin/_type:user
  user:root/admin/_type:team
  ...
```

After bootstrap, ALL mutations require permission checks.

## Protected API

All write operations require a requester parameter:

```rust
// Create entity (checks ENTITY_CREATE on _type:{type})
create_entity(requester, entity_type, id) -> Result<()>

// Set grant (checks GRANT_WRITE on scope)
set_grant(requester, seeker, relation, scope) -> Result<u64>

// Set capability (checks CAP_WRITE on scope)
set_capability(requester, scope, relation, cap_mask) -> Result<u64>

// Set delegation (checks DELEGATE_WRITE on scope)
set_delegation(requester, seeker, scope, delegate) -> Result<u64>
```

## Usage Example

```rust
use capbit::{bootstrap, create_entity, set_grant, set_capability,
             set_delegation, check_access, SystemCap};

fn main() -> capbit::Result<()> {
    capbit::init("./data/capbit.mdb")?;

    // Genesis - only runs once
    bootstrap("root")?;

    // Root creates teams
    create_entity("user:root", "team", "hr")?;
    create_entity("user:root", "team", "engineering")?;

    // Root creates users
    create_entity("user:root", "user", "alice")?;
    create_entity("user:root", "user", "bob")?;

    // Define what "lead" means on team:hr
    set_capability("user:root", "team:hr", "lead",
                   SystemCap::GRANT_WRITE | SystemCap::GRANT_READ)?;

    // Alice is lead of HR
    set_grant("user:root", "user:alice", "lead", "team:hr")?;

    // HR team can manage users
    set_grant("user:root", "team:hr", "admin", "_type:user")?;

    // Alice inherits HR's permissions via delegation
    set_delegation("user:root", "user:alice", "_type:user", "team:hr")?;

    // Now alice can create users!
    let can_create = check_access("user:alice", "_type:user", None)?;
    assert!((can_create & SystemCap::ENTITY_CREATE) != 0);

    create_entity("user:alice", "user", "frank")?;  // Works!

    Ok(())
}
```

## File Structure

```
capbit/
├── src/
│   ├── lib.rs          # Public API re-exports
│   ├── core.rs         # Core database operations
│   ├── bootstrap.rs    # Genesis and bootstrap logic
│   ├── protected.rs    # Protected API layer
│   └── policy.rs       # Policy evaluation
├── tests/
│   └── integration.rs  # Integration tests
├── Cargo.toml          # Rust package config
├── REFACTOR.md         # Refactor specification
├── SIMULATION.md       # Database state simulation
└── README.md           # User documentation
```

## Design Principles

1. **Typed Entities**: `type:id` format guarantees uniqueness
2. **Types as Scopes**: Entity types control mutation permissions
3. **String Relations**: Human-readable ("lead", "member", "owner")
4. **Bitmask Capabilities**: O(1) permission evaluation
5. **Genesis Bootstrap**: System starts with auditable root authority
6. **Protected Mutations**: All writes require authorization
7. **Explicit Delegation**: Inheritance via delegation records
8. **Per-Scope Semantics**: Each scope defines what relations mean
9. **Policy Support**: Conditional access (time, location, etc.)
10. **Bidirectional Queries**: "What can X access?" and "Who can access X?"

## Key Lessons from Simulation

1. **Two-level capability definition**:
   - Type-level: What can you do WITH entities of that type
   - Instance-level: What can you do ON a specific entity

2. **Delegation is explicit**: If team:hr has admin on _type:user, individual users need delegation records to inherit that capability

3. **Creator ownership**: Consider auto-granting "owner" relation to entity creators

4. **Relation definitions are per-scope**: "lead" on team:hr can mean different things than "lead" on team:engineering

5. **Bootstrap creates 6+ entities minimum**: Type entities plus root user

## Open Design Questions

1. Should creators auto-receive "owner" relation on created entities?
2. Should grants validate both seeker and scope exist in registry?
3. Should team membership auto-inherit team capabilities (vs explicit delegation)?
4. Define relations globally or per-scope?
