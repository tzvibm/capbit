# CLAUDE.md

Project context for Claude Code.

## What is Capbit?

Minimal capability-based access control in ~340 lines of Rust. Uses 64-bit bitmasks instead of boolean relations.

## Core Concepts

### Permission Model
- **Subject** and **Object** are both `u64` IDs (no type distinction)
- **Mask**: 64-bit permission bitmask stored per (subject, object) pair
- **Role**: Named mask defined per object (`set_role(object, role_id, mask)`)
- **Inheritance**: Subject can inherit permissions from parent (`set_inherit(object, child, parent)`)

### Permission Checks

GRANT/VIEW check the **target object**, ADMIN checks **_system**:

```rust
let (system, root) = bootstrap()?;  // Creates _system and _root_user

// GRANT on target object:
grant(actor, subject, object, mask)?;       // Requires GRANT on object
revoke(actor, subject, object)?;            // Requires GRANT on object
list_for_object(actor, object)?;            // Requires VIEW on object

// ADMIN on _system (system-level operations):
set_role(actor, object, role, mask)?;       // Requires ADMIN on _system
set_inherit(actor, obj, child, parent)?;    // Requires ADMIN on _system
remove_inherit(actor, obj, child)?;         // Requires ADMIN on _system
```

**Key insight**: To grant access to document X, you need GRANT on X. To configure roles/inheritance (system-level), you need ADMIN on _system.

### API Design

- **Public API** - All write operations require `actor` parameter, check against `_system`
- **Internal** - `transact(|tx| ...)` bypasses protection, used for bootstrap/testing

## File Structure

```
src/lib.rs      # Everything (~340 lines)
tests/lib_test.rs   # Integration tests
tests/benchmarks.rs # Performance tests
```

## Key Functions

```rust
// Init
init(path)?;                              // Uses default options
init_with_options(path, Options { ... })?; // Custom buffer/flush settings

// Bootstrap
bootstrap() -> Result<(u64, u64)>  // Returns (system_id, root_user_id)
get_system() -> Result<u64>
get_root_user() -> Result<Option<u64>>
is_bootstrapped() -> Result<bool>

// Write operations (require actor with permission on _system)
// Writes are buffered and auto-batched by the engine's writer thread
grant(actor, subject, object, mask)      // Requires GRANT
revoke(actor, subject, object)           // Requires GRANT
set_role(actor, object, role, mask)      // Requires ADMIN
set_inherit(actor, object, child, parent) // Requires ADMIN
remove_inherit(actor, object, child)     // Requires ADMIN
list_for_object(actor, object)           // Requires VIEW

// Explicit durability
sync()?;  // Block until all pending writes are committed

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

| Bit | Constant | Checked on | Used by |
|-----|----------|------------|---------|
| 0 | `READ` | — | app-defined |
| 1 | `WRITE` | — | app-defined |
| 2 | `DELETE` | — | app-defined |
| 3 | `CREATE` | — | app-defined |
| 4 | `GRANT` | target object | `grant`, `revoke`, `grant_set` |
| 5 | `EXECUTE` | — | app-defined |
| 6–61 | — | — | app-defined |
| 62 | `VIEW` | target object | `list_for_object` |
| 63 | `ADMIN` | _system | `set_role`, `set_inherit`, `remove_inherit` |

**GRANT/VIEW** checked per-object. **ADMIN** checked on _system (global).

## Auto-Batching (Performance)

LMDB write transactions are expensive (~1-2ms each due to fsync). The engine automatically batches writes for you:

### How It Works
- On `init()`, a global writer thread is spawned
- All public write APIs (`grant`, `revoke`, `set_role`, etc.) send operations to this thread
- The writer thread batches operations and commits them efficiently
- Call `sync()` when you need durability guarantee

### Configuration
```rust
// Default: buffer=1000, interval=0 (optimal ~600K/s, ~1.2x overhead vs transact)
init(path)?;

// Custom settings
init_with_options(path, Options {
    buffer_capacity: 500,      // Flush after 500 ops
    flush_interval_ms: 50,     // Or every 50ms, whichever first
})?;
```

### Usage Pattern
```rust
// Fast: operations are buffered
for i in 0..10000 {
    grant(root, i, 100, READ)?;
}

// When you need durability:
sync()?;  // Blocks until all pending ops are committed

// Now safe to read
assert!(check(9999, 100, READ)?);
```

### Low-Level Batching with `transact()`
For atomic multi-op transactions or when you need immediate commits:
```rust
transact(|tx| {
    tx.grant(1, 100, READ)?;
    tx.grant(2, 100, WRITE)?;
    tx.set_role(100, 1000, READ | WRITE)?;
    Ok(())
})?;  // Committed immediately
```

### Performance Comparison
| Approach | 10K grants | Notes |
|----------|-----------|-------|
| Individual `grant()` without sync | ~instant | Buffered, not yet committed |
| Individual `grant()` + `sync()` | depends on batch size | Engine batches automatically |
| `transact()` | ~500K/s | Single transaction, immediate commit |

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
