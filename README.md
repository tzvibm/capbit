# Capbit

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: Non-Commercial](https://img.shields.io/badge/License-Non--Commercial-red.svg)](LICENSE)

**Minimal capability-based access control for Rust** — 199 lines.

Most ACL libraries are bloated with ORMs, policy languages, and framework dependencies. Capbit is just the math: subjects, objects, and 64-bit capability masks. O(1) permission checks via bitwise AND. LMDB for persistence. No dependencies beyond that.

**Everything is relative.** Subjects and objects are just u64 IDs — they can represent users, files, APIs, devices, anything. Roles are defined per-object, so "admin" on document A can mean something different than "admin" on document B. Capability masks are relative to each subject-object pair. You define the semantics.

This is a library, not a framework. Embed it directly in your application.

## Usage

```rust
use capbit::{init, grant, check, READ, WRITE, DELETE};

fn main() -> capbit::Result<()> {
    init("./data/capbit.mdb")?;

    grant(42, 100, READ | WRITE)?;  // user 42 can read/write doc 100

    assert!(check(42, 100, READ)?);
    assert!(!check(42, 100, DELETE)?);

    Ok(())
}
```

## API

```rust
// Core
grant(subject: u64, object: u64, mask: u64) -> Result<()>
revoke(subject: u64, object: u64) -> Result<bool>
check(subject: u64, object: u64, required: u64) -> Result<bool>
get_mask(subject: u64, object: u64) -> Result<u64>

// Batch
batch_grant(grants: &[(u64, u64, u64)]) -> Result<()>
batch_revoke(revokes: &[(u64, u64)]) -> Result<usize>

// Roles (indirection layer)
set_role(object: u64, role: u64, mask: u64) -> Result<()>
get_role(object: u64, role: u64) -> Result<u64>

// Inheritance (with cycle prevention)
set_inherit(object: u64, child: u64, parent: u64) -> Result<()>
remove_inherit(object: u64, child: u64) -> Result<bool>

// Queries
list_for_subject(subject: u64) -> Result<Vec<(u64, u64)>>
list_for_object(object: u64) -> Result<Vec<(u64, u64)>>

// Labels (optional, for human-readable names)
set_label(id: u64, name: &str) -> Result<()>
get_label(id: u64) -> Result<Option<String>>
get_id_by_label(name: &str) -> Result<Option<u64>>
```

## Constants

```rust
pub const READ: u64 = 1;
pub const WRITE: u64 = 1 << 1;
pub const DELETE: u64 = 1 << 2;
pub const CREATE: u64 = 1 << 3;
pub const GRANT: u64 = 1 << 4;
pub const EXECUTE: u64 = 1 << 5;
pub const VIEW: u64 = 1 << 62;
pub const ADMIN: u64 = 1 << 63;
```

## Design

- **u64 IDs** — no string parsing, direct memory ops
- **64-bit masks** — 64 capability bits per grant
- **O(1) check** — single AND operation
- **O(log N) lookup** — LMDB B-tree
- **BiPair pattern** — forward/reverse indexes stay in sync
- **Cycle prevention** — inheritance validated on write

## Testing

```bash
cargo test -- --test-threads=1
```

## License

Non-Commercial Open Source — see [LICENSE](LICENSE).

For commercial licensing: https://github.com/tzvibm
