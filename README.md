# Capbit

Minimal capability-based access control. ~280 lines of Rust.

## Why Capbit?

Most authorization systems (including Google's Zanzibar) use **boolean relations**: "alice IS a viewer". Each relation is one bit of information. Adding permission types requires schema changes.

Capbit uses **64-bit masks**: each grant carries 2^64 possible permission combinations. Define new permission types at runtime. No schema.

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

grant(alice, doc, READ | WRITE)?;

if check(alice, doc, READ)? {
    // allowed
}
```

## API

### Core

```rust
grant(subject, object, mask)?;      // Add permissions (OR)
grant_set(subject, object, mask)?;  // Set exact permissions
revoke(subject, object)?;           // Remove all
check(subject, object, required)?;  // (mask & required) == required
get_mask(subject, object)?;         // Get current mask
```

### Roles

```rust
set_role(object, role_id, mask)?;   // Define role -> mask
grant(subject, object, role_id)?;   // Assign role
get_role(object, role_id)?;
```

### Inheritance

```rust
set_inherit(object, child, parent)?;
remove_inherit(object, child)?;
get_inherit(object, child)?;
```

### Batch

```rust
transact(|tx| {
    tx.grant(a, b, READ)?;
    tx.set_role(b, EDITOR, READ | WRITE)?;
    tx.create_entity("alice")?;
    Ok(())
})?;
```

### Query

```rust
list_for_subject(subject)?;   // Vec<(object, mask)>
list_for_object(object)?;     // Vec<(subject, mask)>
count_for_subject(subject)?;
count_for_object(object)?;
```

### Entities (optional)

```rust
create_entity(name)?;         // Auto-increment ID
rename_entity(id, name)?;
delete_entity(id)?;
set_label(id, name)?;
get_label(id)?;
get_id_by_label(name)?;
```

## Constants

```rust
pub const READ: u64    = 1;
pub const WRITE: u64   = 1 << 1;
pub const DELETE: u64  = 1 << 2;
pub const CREATE: u64  = 1 << 3;
pub const GRANT: u64   = 1 << 4;
pub const EXECUTE: u64 = 1 << 5;
pub const VIEW: u64    = 1 << 62;
pub const ADMIN: u64   = 1 << 63;
// 56 bits available for custom permissions
```

## Performance

```
Single check:         2-3 us
Batch grants:         200-300K/sec
Inheritance depth 10: ~17 us
Concurrent reads:     2.1M/sec (8 threads)
```

## License

CNCOSL - See LICENSE
