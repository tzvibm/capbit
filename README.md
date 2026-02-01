# Capbit

Minimal capability-based access control. ~280 lines of Rust.

## Features

- **u64 IDs** — Subjects and objects are simple integers
- **64-bit masks** — 2^64 permission combinations per role
- **O(1) checks** — Single bitmask AND operation
- **Unlimited roles** — Any u64 as role ID, defined per object
- **Inheritance** — DAG with cycle prevention
- **Instant updates** — No cache invalidation
- **Zero ops** — Embedded LMDB, single file

## Usage

```rust
use capbit::*;

// Initialize
init("/path/to/db")?;

// Grant permissions
grant(alice, doc, READ | WRITE)?;

// Check access
if check(alice, doc, READ)? {
    // allowed
}

// Revoke
revoke(alice, doc)?;
```

## API

### Core Operations

```rust
grant(subject, object, mask)?;      // Add permissions (OR with existing)
grant_set(subject, object, mask)?;  // Set exact permissions
revoke(subject, object)?;           // Remove all permissions
check(subject, object, required)?;  // Check if (mask & required) == required
get_mask(subject, object)?;         // Get current permission mask
```

### Roles (Indirection)

```rust
set_role(object, role_id, mask)?;   // Define role -> mask mapping
grant(subject, object, role_id)?;   // Assign role (resolved at check time)
get_role(object, role_id)?;         // Get role's mask
```

### Inheritance

```rust
set_inherit(object, child, parent)?;    // Child inherits parent's permissions
remove_inherit(object, child)?;         // Remove inheritance link
get_inherit(object, child)?;            // Get parent ID
```

### Entities (Optional CRUD)

```rust
create_entity(name)?;               // Auto-increment ID with label
rename_entity(id, new_name)?;
delete_entity(id)?;
set_label(id, name)?;               // Label any ID
get_label(id)?;
get_id_by_label(name)?;
```

### Batch Operations

```rust
transact(|tx| {
    tx.grant(a, b, READ)?;
    tx.grant(c, d, WRITE)?;
    tx.set_role(d, EDITOR, READ | WRITE)?;
    tx.create_entity("alice")?;
    Ok(())
})?;

batch_grant(&[(a, b, READ), (c, d, WRITE)])?;
batch_revoke(&[(a, b), (c, d)])?;
```

### Query

```rust
list_for_subject(subject)?;     // Vec<(object, mask)>
list_for_object(object)?;       // Vec<(subject, mask)>
count_for_subject(subject)?;    // Count without iteration
count_for_object(object)?;
list_labels()?;                 // All labeled entities
```

### Protected Operations

Require actor authorization (ADMIN or sufficient permissions):

```rust
protected_grant(actor, subject, object, mask)?;
protected_revoke(actor, subject, object)?;
protected_set_role(actor, object, role, mask)?;
protected_set_inherit(actor, object, child, parent)?;
protected_list_for_object(actor, object)?;  // Requires VIEW
```

### Bootstrap

```rust
bootstrap(root_id)?;    // First-time setup, grants root ADMIN on itself
is_bootstrapped()?;
get_root()?;
```

## Permission Constants

```rust
pub const READ: u64    = 1;
pub const WRITE: u64   = 1 << 1;
pub const DELETE: u64  = 1 << 2;
pub const CREATE: u64  = 1 << 3;
pub const GRANT: u64   = 1 << 4;
pub const EXECUTE: u64 = 1 << 5;
pub const VIEW: u64    = 1 << 62;
pub const ADMIN: u64   = 1 << 63;

// Use remaining 56 bits for custom permissions
```

## Performance

On mobile ARM64 (8 cores):

```
Single check:           2-3 us
Batch grants:           200-300K/sec
Inheritance depth 10:   ~17 us
Concurrent reads:       2.1M checks/sec
```

## License

CNCOSL - See LICENSE
