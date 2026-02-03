# CLAUDE.md

Project context for Claude Code.

## What is Capbit?

Authorization as atomized data. Minimal capability-based access control with u64 IDs and bitmask permissions, backed by fjall (LSM-tree storage).

## Core Framing

|  | Relationships | Semantics |
|---|---|---|
| **ReBAC** | Stored | Computed (code) |
| **Zanzibar** | Atomized | Data (schema) |
| **Capbit** | Atomized | Atomized data |

**Zanzibar's insight**: Semantics belong in data, not code. Stored as schema manifest.

**Capbit's refinement**: Semantics should be atomized data, not schema blob.

## Data Structure

```
SUBJECTS: (subject, object) → role           // relationship tuple
OBJECTS:  (object, role) → mask              // semantic tuple
INHERITS: (subject, object, role) → parent   // role-specific inheritance
```

Three independent tuples. Each queryable on its own.

## Constants

```rust
// Reserved IDs
pub const _SYSTEM: u64 = 1;
pub const _ROOT: u64 = 2;

// Reserved role IDs
pub const _OWNER: u64 = 1;
pub const _ADMIN: u64 = 2;
pub const _EDITOR: u64 = 3;
pub const _VIEWER: u64 = 4;

// Aggregate masks
pub const ALL_BITS: u64 = 0x3FFFFF;    // 22 bits - full access
pub const ADMIN_BITS: u64 = 0x3FCFCF;  // admin operations
pub const EDITOR_BITS: u64 = 0x000366; // read + update
pub const VIEWER_BITS: u64 = 0x000318; // read only
```

## Permission Resolution

```rust
fn get_mask(subject, object) -> u64 {
    let mut mask = 0;
    let mut current = subject;
    for _ in 0..10 {  // max 10 hops
        if let Some(role) = SUBJECTS.get(current, object) {
            mask |= OBJECTS.get(object, role);
            match INHERITS.get(current, object, role) {  // role-specific
                Some(parent) => current = parent,
                None => break,
            }
        } else {
            break;
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
// Initialize
init("data_path")?;

// Bootstrap - creates _SYSTEM object with default roles, grants _ROOT owner
bootstrap() -> Result<(u64, u64)>  // returns (_SYSTEM, _ROOT)

// SUBJECTS table (grants)
grant(actor, subject, object, role)?;      // requires _GRANT bit
revoke(actor, subject, object)?;           // requires _REVOKE bit
get_subject(actor, subject, object)?;      // requires _GET_GRANT bit
check_subject(subject, object, role)?;     // no actor required

// OBJECTS table (role definitions)
create(actor, object, role, mask)?;        // requires _CREATE_ROLE | _CREATE_MASK
update(actor, object, role, mask)?;        // requires _UPDATE_ROLE | _UPDATE_MASK
delete(actor, object, role)?;              // requires _DELETE_ROLE | _DELETE_MASK
get_object(actor, object, role)?;          // requires _GET_ROLE | _GET_MASK
check_object(actor, object, role)?;        // requires _CHECK_ROLE | _CHECK_MASK

// INHERITS table (role-specific inheritance)
inherit(actor, subject, object, role, parent)?;    // requires _SET_INHERIT
remove_inherit(actor, subject, object, role)?;     // requires _REMOVE_INHERIT
get_inherit(actor, subject, object, role)?;        // requires _GET_INHERIT
check_inherit(actor, subject, object, role)?;      // requires _CHECK_INHERIT

// Resolution (no actor required)
check(subject, object, required) -> Result<bool>
get_mask(subject, object) -> Result<u64>

// Utility
clear()?;  // wipe all data
```

## Web UI

Run the test UI with:

```bash
cargo run --features ui --bin ui
```

Opens at http://localhost:3000 - provides forms for all API operations.

## Testing

```bash
cargo test
```

## License

PolyForm Noncommercial 1.0.0 - no commercial use without separate license.
