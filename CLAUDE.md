# CLAUDE.md

Project context for Claude Code.

## What is Capbit?

Minimal capability-based access control in ~280 lines of Rust. Uses 64-bit bitmasks instead of boolean relations.

## Core Concepts

### Permission Model
- **Subject** and **Object** are both `u64` IDs (no type distinction)
- **Mask**: 64-bit permission bitmask stored per (subject, object) pair
- **Role**: Named mask defined per object (`set_role(object, role_id, mask)`)
- **Inheritance**: Subject can inherit permissions from parent (`set_inherit(object, child, parent)`)

### The `_system` Object

All system-level permission checks happen against a special `_system` entity:

```rust
let (system, root_user) = bootstrap()?;  // Creates _system and _root_user

// All write operations check actor's permissions on _system:
grant(actor, subject, object, mask)?;       // Requires GRANT on _system
revoke(actor, subject, object)?;            // Requires GRANT on _system
set_role(actor, object, role, mask)?;       // Requires ADMIN on _system
set_inherit(actor, obj, child, parent)?;    // Requires ADMIN on _system
list_for_object(actor, object)?;            // Requires VIEW on _system
```

**Key insight**: Permission bits only have system meaning when checked against `_system`. On user objects, all 64 bits are free to use however you want.

### API Design

- **Public API** - All write operations require `actor` parameter, check against `_system`
- **Internal** - `transact(|tx| ...)` bypasses protection, used for bootstrap/testing

## File Structure

```
src/lib.rs      # Everything (~280 lines)
tests/lib_test.rs   # Integration tests (68 tests)
tests/benchmarks.rs # Performance tests
```

## Key Functions

```rust
// Bootstrap
bootstrap() -> Result<(u64, u64)>  // Returns (system_id, root_user_id)
get_system() -> Result<u64>
get_root_user() -> Result<Option<u64>>
is_bootstrapped() -> Result<bool>

// Write operations (require actor with permission on _system)
grant(actor, subject, object, mask)      // Requires GRANT
revoke(actor, subject, object)           // Requires GRANT
set_role(actor, object, role, mask)      // Requires ADMIN
set_inherit(actor, object, child, parent) // Requires ADMIN
remove_inherit(actor, object, child)     // Requires ADMIN
list_for_object(actor, object)           // Requires VIEW

// Read operations (no actor needed)
check(subject, object, required) -> bool
get_mask(subject, object) -> u64
get_role(object, role) -> u64
get_inherit(object, child) -> Option<u64>
list_for_subject(subject) -> Vec<(u64, u64)>

// Internal (bypasses protection, use for bootstrap/testing)
transact(|tx| { tx.grant(...); tx.set_role(...); Ok(()) })

// Entities (no protection)
create_entity(name) -> u64
get_label(id) -> Option<String>
get_id_by_label(name) -> Option<u64>
```

## Constants

| Bit | Constant | System capability (on `_system`) |
|-----|----------|----------------------------------|
| 0 | `READ` | — |
| 1 | `WRITE` | — |
| 2 | `DELETE` | — |
| 3 | `CREATE` | — |
| 4 | `GRANT` | `grant`, `revoke`, `batch_grant`, `batch_revoke` |
| 5 | `EXECUTE` | — |
| 6–61 | — | — |
| 62 | `VIEW` | `list_for_object` |
| 63 | `ADMIN` | `set_role`, `set_inherit`, `remove_inherit` |

**On user objects, all 64 bits are free to use.** System only checks against `_system`.

## Testing

```bash
cargo test -- --test-threads=1  # Single-threaded (LMDB requirement)
```

## Storage

Uses LMDB (heed crate). Databases:
- `caps` / `rev`: Bidirectional (subject,object) -> mask
- `roles`: (object,role) -> mask
- `inh`: (object,child) -> parent
- `meta`: Bootstrap state (boot, system, root_user)
- `labels` / `names`: Entity ID <-> name mapping
