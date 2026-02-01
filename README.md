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

// Create named entities
let alice = create_entity("alice")?;
let bob = create_entity("bob")?;
let doc = create_entity("quarterly-report")?;

// Grant permissions
grant(alice, doc, READ | WRITE)?;
grant(bob, doc, READ)?;

// Check access
if check(alice, doc, WRITE)? {
    println!("{} can write", get_label(alice)?.unwrap());
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

### Bootstrap & Protected API

System permissions are scoped to a special `_system` object. All system-level privileges (ADMIN, GRANT, etc.) are checked against this entity, preventing users from granting themselves global powers on arbitrary objects.

```rust
// Initialize the system (creates _system and _root_user entities)
let (system, root_user) = bootstrap()?;

// root_user has all bits on _system
assert!(check(root_user, system, u64::MAX)?);

// Protected operations check actor's permissions on _system
protected_grant(actor, subject, object, mask)?;   // Requires GRANT on _system
protected_revoke(actor, subject, object)?;        // Requires GRANT on _system
protected_set_role(actor, object, role, mask)?;   // Requires ADMIN on _system
protected_set_inherit(actor, obj, child, parent)?; // Requires ADMIN on _system
protected_remove_inherit(actor, object, child)?;  // Requires ADMIN on _system
protected_list_for_object(actor, object)?;        // Requires VIEW on _system

// Query bootstrap state
is_bootstrapped()?;           // true if bootstrap() was called
get_system()?;                // Returns _system entity ID
get_root_user()?;             // Returns _root_user entity ID
```

**User freedom:** Users can use any bit on their own objects. The system only enforces permissions when you use `protected_*` functions, and only checks against `_system`:

```rust
const MY_CUSTOM: u64 = 1 << 50;
grant(alice, my_doc, ADMIN | MY_CUSTOM)?;  // No system permission needed
// ADMIN bit here is just data—system doesn't intercept it
```

## Constants

These constants define system capabilities when checked against `_system`:

| Bit | Constant | Meaning on `_system` |
|-----|----------|----------------------|
| 0 | `READ` | — |
| 1 | `WRITE` | — |
| 2 | `DELETE` | — |
| 3 | `CREATE` | — |
| 4 | `GRANT` | Can use `protected_grant`, `protected_revoke` |
| 5 | `EXECUTE` | — |
| 6–61 | — | — |
| 62 | `VIEW` | Can use `protected_list_for_object` |
| 63 | `ADMIN` | Can use `protected_set_role`, `protected_set_inherit` |

**On your own objects, all 64 bits are free to use however you want.** The system only checks bits against `_system`. On any other object, you can reuse `READ`, `ADMIN`, or any bit for your own meanings.

## Performance

```
Single check:         2-3 us
Batch grants:         200-300K/sec
Inheritance depth 10: ~17 us
Concurrent reads:     2.1M/sec (8 threads)
```

## Testing

```bash
cargo test -- --test-threads=1
```

Tests must run single-threaded (LMDB limitation).

## License

CNCOSL - See LICENSE
