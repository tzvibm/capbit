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
SUBJECTS:           (subject, object, role) → 1        // grant (multi-role per subject+object)
SUBJECTS_REV:       (object, subject, role) → 1        // reverse index
OBJECTS:            (object, role) → mask              // semantic tuple
INHERITS:           (subject, object, role) → parent   // role-specific inheritance
INHERITS_BY_OBJ:    (object, role, parent, subject) → 1   // reverse index
INHERITS_BY_PARENT: (parent, object, role, subject) → 1   // reverse index
```

Six partitions. Reverse indexes enable efficient queries in both directions. A subject can have multiple roles on an object. All multi-partition writes use atomic batches.

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
        for role in SUBJECTS.prefix(current, object) {  // multi-role
            mask |= OBJECTS.get(object, role);
            if let Some(parent) = INHERITS.get(current, object, role) {
                current = parent;
                break;
            }
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

// SUBJECTS table (grants) - subject can have multiple roles on object
grant(actor, subject, object, role)?;      // requires _GRANT bit (atomic)
revoke(actor, subject, object, role)?;     // requires _REVOKE bit (atomic)
check_subject(subject, object, role)?;     // no actor required

// SUBJECTS list queries
list_roles_for(actor, subject, object)?;   // → Vec<role>
list_grants(actor, subject)?;              // → Vec<(object, role)>
list_subjects(actor, object)?;             // → Vec<(subject, role)>

// OBJECTS table (role definitions)
create(actor, object, role, mask)?;        // requires _CREATE_ROLE | _CREATE_MASK
update(actor, object, role, mask)?;        // requires _UPDATE_ROLE | _UPDATE_MASK
delete(actor, object, role)?;              // requires _DELETE_ROLE | _DELETE_MASK
get_object(actor, object, role)?;          // requires _GET_ROLE | _GET_MASK
check_object(actor, object, role)?;        // requires _CHECK_ROLE | _CHECK_MASK
list_roles(actor, object)?;                // → Vec<(role, mask)>

// INHERITS table (role-specific inheritance)
inherit(actor, subject, object, role, parent)?;    // requires _SET_INHERIT (atomic)
remove_inherit(actor, subject, object, role)?;     // requires _REMOVE_INHERIT (atomic)
get_inherit(actor, subject, object, role)?;        // requires _GET_INHERIT
check_inherit(actor, subject, object, role)?;      // requires _CHECK_INHERIT

// INHERITS list queries
list_inherits(actor, subject, object)?;                    // → Vec<(role, parent)>
list_inherits_on_obj(actor, object)?;                      // → Vec<(role, parent, subject)>
list_inherits_on_obj_role(actor, object, role)?;           // → Vec<(parent, subject)>
list_inherits_from_parent(actor, parent)?;                 // → Vec<(object, role, subject)>
list_inherits_from_parent_on_obj(actor, parent, object)?;  // → Vec<(role, subject)>

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
cargo test -- --test-threads=1
```

Note: Tests share database state, so must run single-threaded.

## License

PolyForm Noncommercial 1.0.0 - no commercial use without separate license.
