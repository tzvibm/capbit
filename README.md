# Capbit

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: Non-Commercial](https://img.shields.io/badge/License-Non--Commercial-red.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-100%20passing-brightgreen.svg)](#testing)

**Minimal capability-based access control for Rust** - ~770 lines total.

```
Can user 42 read document 100?

  ┌─────────┐    check(42, 100, READ)    ┌─────────┐
  │ User 42 │ ────────────────────────►  │ Doc 100 │
  │ mask: 7 │                            │         │
  └─────────┘                            └─────────┘
                        │
                        ▼
                  ✓ ALLOWED
            (7 & 1 == 1)
```

---

## Features

| Feature | Description |
|---------|-------------|
| **O(1) Check** | Permission check is a single AND operation |
| **O(log N) Lookup** | LMDB B-tree storage |
| **u64 IDs** | Simple numeric identifiers, no parsing |
| **64-bit Masks** | 64 capability bits per grant |
| **Roles** | Per-object role definitions for indirection |
| **Inheritance** | Dynamic parent lookup with cycle prevention |
| **Protected API** | Admin-controlled mutations |
| **100 Tests** | Comprehensive integration + benchmark tests |

---

## Quick Start

```rust
use capbit::{init, bootstrap, create_entity, grant, check, READ, WRITE};

fn main() -> capbit::Result<()> {
    init("./data/capbit.mdb")?;
    bootstrap(1)?;  // User 1 is root

    // Create entities (auto-increment IDs)
    let alice = create_entity("alice")?;  // returns 1
    let bob = create_entity("bob")?;      // returns 2
    let doc = create_entity("report")?;   // returns 3

    // Grant access
    grant(bob, doc, READ | WRITE)?;

    // Check access
    assert!(check(bob, doc, READ)?);
    assert!(!check(bob, doc, DELETE)?);

    Ok(())
}
```

---

## API

### Initialization

```rust
init(path: &str) -> Result<()>           // Initialize database
bootstrap(root: u64) -> Result<()>       // Create root (once only)
is_bootstrapped() -> Result<bool>
get_root() -> Result<Option<u64>>
```

### Core Operations

```rust
grant(subject: u64, object: u64, mask: u64) -> Result<()>
revoke(subject: u64, object: u64) -> Result<bool>
check(subject: u64, object: u64, required: u64) -> Result<bool>
get_mask(subject: u64, object: u64) -> Result<u64>
```

### Batch Operations

```rust
batch_grant(grants: &[(u64, u64, u64)]) -> Result<()>
batch_revoke(revokes: &[(u64, u64)]) -> Result<usize>
```

### Roles

```rust
set_role(object: u64, role: u64, mask: u64) -> Result<()>
get_role(object: u64, role: u64) -> Result<u64>
```

### Inheritance

```rust
set_inherit(object: u64, child: u64, parent: u64) -> Result<()>
remove_inherit(object: u64, child: u64) -> Result<bool>
get_inherit(object: u64, child: u64) -> Result<Option<u64>>
```

### Queries

```rust
list_for_subject(subject: u64) -> Result<Vec<(u64, u64)>>
list_for_object(object: u64) -> Result<Vec<(u64, u64)>>
```

### Protected Operations

```rust
protected_grant(actor, subject, object, mask) -> Result<()>
protected_revoke(actor, subject, object) -> Result<bool>
protected_set_role(actor, object, role, mask) -> Result<()>
protected_set_inherit(actor, object, child, parent) -> Result<()>
```

### Entity CRUD

```rust
create_entity(name: &str) -> Result<u64>   // Auto-increment ID
rename_entity(id: u64, name: &str) -> Result<()>
delete_entity(id: u64) -> Result<bool>
```

### Labels

```rust
set_label(id: u64, name: &str) -> Result<()>
get_label(id: u64) -> Result<Option<String>>
get_id_by_label(name: &str) -> Result<Option<u64>>
list_labels() -> Result<Vec<(u64, String)>>
```

### Capability Helpers

```rust
caps_to_names(mask: u64) -> Vec<&'static str>
names_to_caps(names: &[&str]) -> u64
```

---

## Capability Constants

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

---

## Architecture

### Files

| File | Lines | Purpose |
|------|-------|---------|
| `src/lib.rs` | 220 | Core library |
| `src/bin/server.rs` | 100 | REST API (14 endpoints) |
| `demo/index.html` | 150 | Interactive web UI |
| `tests/integration.rs` | 430 | 89 integration tests |
| `tests/benchmarks.rs` | 135 | 11 benchmark tests |
| **Total** | **~770** | |

### Database (LMDB)

```rust
struct Dbs {
    caps: Db,   // [subject:8][object:8] -> mask
    rev: Db,    // [object:8][subject:8] -> mask (reverse index)
    roles: Db,  // [object:8][role:8] -> mask
    inh: Db,    // [object:8][child:8] -> parent
    meta: Db,   // string keys -> values
}
```

---

## Server

```bash
cargo run --features server
```

### Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/status` | System status |
| POST | `/bootstrap` | Initialize root |
| POST | `/entity` | Create entity |
| POST | `/entity/rename` | Rename entity |
| POST | `/entity/delete` | Delete entity |
| POST | `/grant` | Grant access |
| POST | `/revoke` | Revoke access |
| POST | `/role` | Set role |
| POST | `/inherit` | Set inheritance |
| GET | `/check` | Check permissions |
| GET | `/list` | List grants |
| POST | `/label` | Set label |
| GET | `/labels` | List labels |
| POST | `/reset` | Reset database |

---

## Testing

```bash
cargo test -- --test-threads=1
```

### Test Coverage

| Category | Tests |
|----------|-------|
| Core operations | 9 |
| Capability bits | 15+ |
| Edge case IDs | 64 |
| Isolation | 3 |
| List operations | 5 |
| Batch operations | 7 |
| Roles | 8 |
| Inheritance | 11 |
| Cycle prevention | 5 |
| Bootstrap | 6 |
| Protected ops | 10 |
| Labels | 7 |
| Entity CRUD | 5 |
| Cap helpers | 8 |
| Constants | 2 |
| Stress/scale | 4 |
| Benchmarks | 11 |
| **Total** | **~100** |

---

## Design Principles

1. **u64 IDs** - No string parsing, direct memory operations
2. **Single file library** - Everything in `lib.rs`
3. **Cycle prevention** - `no_cycle()` validates inheritance
4. **Pre-merged masks** - Grants accumulate via OR
5. **Dynamic inheritance** - Parent's access checked at query time
6. **Fail secure** - Missing permission = denied

---

## License

**Non-Commercial Open Source License** - see [LICENSE](LICENSE).

For commercial licensing: https://github.com/tzvibm
