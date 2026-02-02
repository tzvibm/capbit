# CLAUDE.md

Project context for Claude Code.

## What is Capbit?

Authorization as first-class data.

Zanzibar stored relationships. Capbit decouples them from schema.

## Core Framing

|  | Relationships | Semantics |
|---|---|---|
| **ReBAC** | Stored | Computed |
| **Zanzibar** | Atomic tuple | Coupled to schema |
| **Capbit** | Atomic tuple | Atomic tuple |

- **ReBAC**: Relationships stored, semantic relationships computed (expensive)
- **Zanzibar**: Relationship tuple atomic, but coupled to schema for meaning
- **Capbit**: Three separate atomic tuples - relationship, semantic, inheritance

Anything Zanzibar expresses can be expressed in Capbit. Zanzibar provides schema skeleton - Capbit provides decoupled tuples.

## Data Structure

```
caps:     (subject, object) → role          // relationship tuple
roles:    (object, role) → mask             // semantic tuple (separate!)
inherit:  (object, child) → parent          // inheritance tuple (separate!)
```

All tuples. All atomic. All decoupled.

## Permission Check Flow

```rust
fn check(subject, object, required) -> bool {
    let role = caps.get(subject, object);      // relationship tuple
    let mask = roles.get(object, role);        // semantic tuple
    mask & required == required
}
```

With inheritance:
```rust
fn get_mask(subject, object) -> u64 {
    let mut mask = 0;
    let mut current = subject;
    loop {
        let role = caps.get(current, object);
        mask |= roles.get(object, role);
        match inherit.get(object, current) {
            Some(parent) => current = parent,
            None => break,
        }
    }
    mask
}
```

## API

```rust
// Bootstrap
bootstrap() -> (system_id, root_user_id)

// Relationship tuples
grant(actor, subject, object, role)       // GRANT required on object
revoke(actor, subject, object)            // GRANT required on object

// Semantic tuples
set_role(actor, object, role, mask)       // ADMIN required on _system

// Inheritance tuples
set_inherit(actor, object, child, parent) // ADMIN required on _system

// Read (no actor)
check(subject, object, required) -> bool
get_mask(subject, object) -> u64
get_role(object, role) -> u64

// Internal (bypasses protection)
transact(|tx| { tx.grant(...); Ok(()) })
```

## Testing

```bash
cargo test -- --test-threads=1
```
