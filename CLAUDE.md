# CLAUDE.md

Technical guidance for Claude Code when working with this repository.

> **Version:** v3.2 (~770 lines total)

## Project Overview

Capbit is a minimal Rust library for capability-based access control:
- **u64 IDs**: Subjects and objects are simple u64 numbers
- **Bitmask capabilities**: 64-bit masks with named constants (READ, WRITE, DELETE, etc.)
- **O(1) evaluation**: Permission check is a single AND operation
- **Roles**: Scoped per-object role definitions for indirection
- **Inheritance**: Dynamic parent lookup with cycle prevention
- **Entities**: CRUD for users/resources with auto-increment IDs
- **Labels**: Human-readable names for IDs
- **Protected mutations**: Admin-controlled operations

## Commands

```bash
cargo build                    # Build library
cargo build --features server  # Build with server
cargo test                     # Run all 24 tests
cargo run --features server    # Run server (port 3000)
```

## Architecture

### Files

| File | Lines | Purpose |
|------|-------|---------|
| `src/lib.rs` | 219 | Core library with `Dbs` struct, cycle prevention |
| `src/bin/server.rs` | 100 | REST API: 14 endpoints |
| `demo/index.html` | 146 | Collapsible UI with nav, emojis |
| `tests/integration.rs` | 170 | 24 integration tests |
| `tests/benchmarks.rs` | 133 | Performance benchmarks |

### Database (5 tables via `Dbs` struct)

```rust
struct Dbs {
    caps: Db,   // [subject:8][object:8] -> role/mask
    rev: Db,    // [object:8][subject:8] -> role/mask (reverse index)
    roles: Db,  // [object:8][role:8] -> mask (role definitions)
    inh: Db,    // [object:8][child:8] -> parent (inheritance)
    meta: Db,   // string keys -> values (labels, bootstrap, next_id)
}
```

### Core API

```rust
// Initialization
pub fn init(path: &str) -> Result<()>
pub fn clear_all() -> Result<()>

// Entity CRUD
pub fn create_entity(name: &str) -> Result<u64>      // auto-increment ID
pub fn rename_entity(id: u64, name: &str) -> Result<()>
pub fn delete_entity(id: u64) -> Result<bool>

// Core operations
pub fn grant(subject: u64, object: u64, role: u64) -> Result<()>
pub fn revoke(subject: u64, object: u64) -> Result<bool>
pub fn check(subject: u64, object: u64, required: u64) -> Result<bool>
pub fn get_mask(subject: u64, object: u64) -> Result<u64>
pub fn batch_grant(grants: &[(u64, u64, u64)]) -> Result<()>
pub fn batch_revoke(revokes: &[(u64, u64)]) -> Result<usize>

// Roles (scoped per object)
pub fn set_role(object: u64, role: u64, mask: u64) -> Result<()>
pub fn get_role(object: u64, role: u64) -> Result<u64>

// Inheritance (with cycle prevention)
pub fn set_inherit(object: u64, child: u64, parent: u64) -> Result<()>
pub fn remove_inherit(object: u64, child: u64) -> Result<bool>
pub fn get_inherit(object: u64, child: u64) -> Result<Option<u64>>

// Queries
pub fn list_for_subject(subject: u64) -> Result<Vec<(u64, u64)>>
pub fn list_for_object(object: u64) -> Result<Vec<(u64, u64)>>

// Protected (require actor authorization)
pub fn protected_grant(actor, subject, object, role) -> Result<()>
pub fn protected_revoke(actor, subject, object) -> Result<bool>
pub fn protected_set_role(actor, object, role, mask) -> Result<()>
pub fn protected_set_inherit(actor, object, child, parent) -> Result<()>

// Bootstrap
pub fn bootstrap(root_id: u64) -> Result<()>
pub fn is_bootstrapped() -> Result<bool>
pub fn get_root() -> Result<Option<u64>>

// Labels
pub fn set_label(id: u64, name: &str) -> Result<()>
pub fn get_label(id: u64) -> Result<Option<String>>
pub fn get_id_by_label(name: &str) -> Result<Option<u64>>
pub fn list_labels() -> Result<Vec<(u64, String)>>

// Capability helpers
pub fn caps_to_names(mask: u64) -> Vec<&'static str>
pub fn names_to_caps(names: &[&str]) -> u64
```

### Capability Constants

```rust
pub const READ: u64 = 1;          // 0x01
pub const WRITE: u64 = 1 << 1;    // 0x02
pub const DELETE: u64 = 1 << 2;   // 0x04
pub const CREATE: u64 = 1 << 3;   // 0x08
pub const GRANT: u64 = 1 << 4;    // 0x10
pub const EXECUTE: u64 = 1 << 5;  // 0x20
pub const VIEW: u64 = 1 << 62;    // Protected list access
pub const ADMIN: u64 = 1 << 63;   // Full control
```

### Server Endpoints

| Method | Endpoint | Body/Query | Response |
|--------|----------|------------|----------|
| GET | /status | - | `{ bootstrapped, root }` |
| POST | /bootstrap | `{ root_id }` | `{ ok, data: root_id }` |
| POST | /entity | `{ name }` | `{ ok, data: id }` |
| POST | /entity/rename | `{ id, name }` | `{ ok }` |
| POST | /entity/delete | `{ id }` | `{ ok }` |
| POST | /grant | `{ actor, subject, object, mask }` | `{ ok }` |
| POST | /revoke | `{ actor, subject, object }` | `{ ok }` |
| POST | /role | `{ actor, object, role_id, mask }` | `{ ok }` |
| POST | /inherit | `{ actor, object, child, parent }` | `{ ok }` |
| GET | /check | `?subject=&object=&required=` | `{ allowed, mask }` |
| GET | /list | `?subject=` or `?object=` | `[{ id, mask, label }]` |
| POST | /label | `{ id, name }` | `{ ok }` |
| GET | /labels | - | `[{ id, mask, label }]` |
| POST | /reset | - | `{ ok }` |

### Validation

```rust
// Prevents self-reference and circular inheritance
fn no_cycle(d: &Dbs, tx: &RoTxn, obj: u64, from: u64, to: u64) -> Result<()>
```

| Case | Example | Error |
|------|---------|-------|
| Self-inherit | `set_inherit(doc, alice, alice)` | "Cannot reference self" |
| Direct cycle | A→B, B→A | "Circular reference" |
| Chain cycle | A→B→C→A | "Circular reference" |

## Quick Start

```rust
use capbit::{init, bootstrap, create_entity, grant, check, READ, WRITE};

init("/tmp/capbit.mdb")?;
bootstrap(1)?;

// Create entities (auto-increment IDs)
let alice = create_entity("alice")?;   // returns 1
let bob = create_entity("bob")?;       // returns 2
let doc = create_entity("report.pdf")?; // returns 3

// Grant access
grant(bob, doc, READ | WRITE)?;

// Check access
assert!(check(bob, doc, READ)?);
assert!(!check(bob, doc, DELETE)?);
```

## Demo UI Features

- 🧭 **Sticky nav**: Jump to any section
- 📂 **Collapsible**: Click headers to expand/collapse
- 😀 **Emojis**: Visual icons for sections
- 🔄 **Auto-collapse**: Setup hides after init
- ✅ **Validation**: Helpful error messages

## Design Principles

1. **u64 IDs**: No string parsing, direct memory operations
2. **Named Dbs struct**: `d.caps`, `d.roles` instead of `d.0`, `d.2`
3. **Cycle prevention**: `no_cycle()` validates inheritance
4. **Entity CRUD**: `create_entity()` with auto-increment
5. **Roles as indirection**: Change one role = update millions of users
6. **Dynamic inheritance**: Parent's current access checked at query time
7. **Batch operations**: LMDB single-writer optimized
8. **Fail secure**: Missing permission = denied
