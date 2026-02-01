# Capbit

Minimal capability-based access control. ~280 lines of Rust.

## Why Capbit?

Most authorization systems (including Google's Zanzibar) use **boolean relations**: "alice IS a viewer". Each relation is one bit of information. Adding permission types requires schema changes.

Capbit uses **64-bit masks**: each grant carries 2^64 possible permission combinations. Define new permission types at runtime. No schema.

**Not just numbers:** While IDs are u64 internally for speed, you can use human-readable labels:

```rust
let alice = create_entity("alice")?;    // returns u64 ID
let doc = create_entity("quarterly-report")?;
grant(alice, doc, READ | WRITE)?;

// Or look up by name
let alice = get_id_by_label("alice")?.unwrap();
```

| | Traditional (Zanzibar-style) | Capbit |
|---|---|---|
| Permissions per grant | 1 (boolean) | 64 bits |
| Roles per object | Fixed by schema | Unlimited (any u64) |
| New permission type | Schema migration | Runtime, instant |
| Permission check | Graph traversal | Single AND operation |
| Entity model | Typed namespaces | Unified u64 IDs |

### Unified Entity Model

Subject and object are the same concept—just u64 IDs. This means inheritance works in any direction:

```rust
// User inherits from group (traditional)
set_inherit(doc, alice, engineering)?;

// Object inherits from object (folder hierarchy)
set_inherit(policy, doc, folder)?;
set_inherit(policy, folder, workspace)?;

// Any entity inherits from any entity
// No artificial type system limits your model
```

### Cheap Restructuring

One inheritance change restructures entire subtrees instantly:

```rust
// Move entire engineering org under new VP
set_inherit(company, engineering, new_vp)?;
// Done. Thousands of users inherit through new_vp.
```

No tuple updates. No cache invalidation. Computed at read time.

### No Derived State

Permissions are stored directly as bitmasks. No "computed" permissions that could be stale. Every check reads current data.

See [COMPARISON.md](COMPARISON.md) for detailed analysis vs Zanzibar.

---

## Usage

```rust
use capbit::*;

init("/path/to/db")?;

// Bootstrap creates _system and _root_user
let (system, root) = bootstrap()?;

// Create named entities
let alice = create_entity("alice")?;
let bob = create_entity("bob")?;
let doc = create_entity("quarterly-report")?;

// Grant permissions (requires actor with GRANT on _system)
grant(root, alice, doc, READ | WRITE)?;
grant(root, bob, doc, READ)?;

// Check access (no actor needed for reads)
if check(alice, doc, WRITE)? {
    println!("{} can write", get_label(alice)?.unwrap());
}

// Delegate: give alice GRANT permission on _system
grant(root, alice, system, GRANT)?;
// Now alice can grant permissions too
grant(alice, bob, doc, WRITE)?;
```

## API

All write operations require an `actor` with appropriate permissions on `_system`.

### Core (require GRANT on `_system`)

```rust
grant(actor, subject, object, mask)?;      // Add permissions (OR)
grant_set(actor, subject, object, mask)?;  // Set exact permissions
revoke(actor, subject, object)?;           // Remove all
```

### Reads (no actor needed)

```rust
check(subject, object, required)?;  // (mask & required) == required
get_mask(subject, object)?;         // Get current mask
list_for_subject(subject)?;         // Vec<(object, mask)>
count_for_subject(subject)?;
count_for_object(object)?;
```

### Roles (require ADMIN on `_system`)

```rust
set_role(actor, object, role_id, mask)?;   // Define role -> mask
get_role(object, role_id)?;                // Read (no actor)
```

### Inheritance (require ADMIN on `_system`)

```rust
set_inherit(actor, object, child, parent)?;
remove_inherit(actor, object, child)?;
get_inherit(object, child)?;               // Read (no actor)
```

### List for object (requires VIEW on `_system`)

```rust
list_for_object(actor, object)?;           // Vec<(subject, mask)>
```

### Batch (require GRANT on `_system`)

```rust
batch_grant(actor, &[(subject, object, mask), ...])?;
batch_revoke(actor, &[(subject, object), ...])?;
```

### Internal batch (bypasses protection)

```rust
transact(|tx| {
    tx.grant(subject, object, READ)?;
    tx.set_role(object, EDITOR, READ | WRITE)?;
    tx.create_entity("alice")?;
    Ok(())
})?;
```

### Entities (no protection)

```rust
create_entity(name)?;         // Auto-increment ID
rename_entity(id, name)?;
delete_entity(id)?;
set_label(id, name)?;
get_label(id)?;
get_id_by_label(name)?;
```

### Bootstrap

```rust
let (system, root_user) = bootstrap()?;  // Creates _system and _root_user
is_bootstrapped()?;                       // true if bootstrap() was called
get_system()?;                            // Returns _system entity ID
get_root_user()?;                         // Returns _root_user entity ID
```

## Constants

These constants define system capabilities when checked against `_system`:

| Bit | Constant | Meaning on `_system` |
|-----|----------|----------------------|
| 0 | `READ` | — |
| 1 | `WRITE` | — |
| 2 | `DELETE` | — |
| 3 | `CREATE` | — |
| 4 | `GRANT` | Can use `grant`, `revoke`, `batch_grant`, `batch_revoke` |
| 5 | `EXECUTE` | — |
| 6–61 | — | — |
| 62 | `VIEW` | Can use `list_for_object` |
| 63 | `ADMIN` | Can use `set_role`, `set_inherit`, `remove_inherit` |

**On your own objects, all 64 bits are free to use however you want.** The system only checks bits against `_system`. On any other object, you can reuse `READ`, `ADMIN`, or any bit for your own meanings.

## Performance

```
Single check:         ~500 ns
Batch grants:         400-650K/sec
Inheritance depth 10: ~5 us
Concurrent reads:     6M/sec (8 threads)
```

## Testing

```bash
cargo test -- --test-threads=1
```

Tests must run single-threaded (LMDB limitation).

## License

CNCOSL - See LICENSE
