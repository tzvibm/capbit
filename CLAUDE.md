# CLAUDE.md

Project context for Claude Code.

## What is Capbit?

Schema-free authorization. Role definitions stored as indexed data, not a parsed manifest.

## Core Idea

Zanzibar: Schema defines what roles mean (type-level), tuples store who has roles.
Capbit: Both stored as indexed tuples (instance-level).

## Data Structure

```
caps:     (subject, object) → role_id       // who has what
caps_rev: (object, subject) → role_id       // reverse index
roles:    (object, role_id) → mask          // what roles mean (per object!)
inherit:  (object, child) → parent          // permission inheritance
```

All pointer-based index lookups. Key insight: `roles` is keyed by `(object, role_id)`, so each object defines its own role semantics.

## Permission Check Flow

```rust
fn get_mask(subject, object) -> u64 {
    let mut mask = 0;
    let mut current = subject;
    loop {
        let role_id = caps.get(current, object);
        mask |= roles.get(object, role_id);  // object-specific role lookup
        match inherit.get(object, current) {
            Some(parent) => current = parent,
            None => break,
        }
    }
    mask
}

fn check(subject, object, required) -> bool {
    get_mask(subject, object) & required == required
}
```

## API

```rust
// Bootstrap
bootstrap() -> (system_id, root_user_id)

// Write (require actor with permission on _system)
grant(actor, subject, object, role_id)    // GRANT required
revoke(actor, subject, object)            // GRANT required
set_role(actor, object, role, mask)       // ADMIN required
set_inherit(actor, object, child, parent) // ADMIN required

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
