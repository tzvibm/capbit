# CLAUDE.md

Project context for Claude Code.

## What is Capbit?

Authorization as atomized data.

## Core Framing

|  | Relationships | Semantics |
|---|---|---|
| **ReBAC** | Stored | Computed (code) |
| **Zanzibar** | Atomized | Data (schema) |
| **Capbit** | Atomized | Atomized data |

**Zanzibar's insight**: Semantics belong in data, not code. Stored as schema manifest.

**Capbit's refinement**: Semantics should be atomized data, not schema blob.

Summary:
- **ReBAC**: Relationships stored, semantics computed
- **Zanzibar**: Relationships atomized, semantics in schema (data, not atomized)
- **Capbit**: Both atomized into independent tuples

## Data Structure

```
caps:     (subject, object) → role          // relationship tuple
roles:    (object, role) → mask             // semantic tuple
inherit:  (object, child) → parent          // inheritance tuple
```

Three independent tuples. Each queryable on its own.

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
