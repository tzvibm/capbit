# Capbit API Plan

## Data Model

```
OBJECTS:  (object, role) → mask              # role definitions
SUBJECTS: (subject, object) → role           # grants
INHERITS: (subject, object, role) → parent   # role-specific inheritance
```

## Constants (public)

```rust
// Reserved IDs
pub const _SYSTEM: u64 = 1;
pub const _ROOT: u64 = 2;

// Reserved role IDs
pub const _OWNER: u64 = 1;
pub const _ADMIN: u64 = 2;
pub const _EDITOR: u64 = 3;
pub const _VIEWER: u64 = 4;

// Aggregate masks (derived from granular bits)
// VIEWER = get/check bits only
pub const VIEWER_BITS: u64 = _get_role | _check_role | _get_mask | _check_mask |
                              _get_object | _check_object | _get_grant | _check_grant |
                              _get_inherit | _check_inherit;

// EDITOR = viewer + update
pub const EDITOR_BITS: u64 = VIEWER_BITS | _update_role | _update_mask;

// ADMIN = editor + create/delete roles, grant/revoke, inherit ops
pub const ADMIN_BITS: u64 = EDITOR_BITS | _create_role | _create_mask | _delete_role |
                             _delete_mask | _grant | _revoke | _set_inherit | _remove_inherit;

// OWNER = admin + create/delete objects (all 22 bits)
pub const ALL_BITS: u64 = ADMIN_BITS | _create_object | _delete_object;  // = 0x3FFFFF
```

### Granular Bits (internal, not exported)

```
# Role operations
_create_role     = 1 << 0
_update_role     = 1 << 1
_delete_role     = 1 << 2
_get_role        = 1 << 3
_check_role      = 1 << 4

# Mask operations
_create_mask     = 1 << 5
_update_mask     = 1 << 6
_delete_mask     = 1 << 7
_get_mask        = 1 << 8
_check_mask      = 1 << 9

# Object operations
_create_object   = 1 << 10
_delete_object   = 1 << 11
_get_object      = 1 << 12
_check_object    = 1 << 13

# Grant operations
_grant           = 1 << 14
_revoke          = 1 << 15
_get_grant       = 1 << 16
_check_grant     = 1 << 17

# Inherit operations
_set_inherit     = 1 << 18
_remove_inherit  = 1 << 19
_get_inherit     = 1 << 20
_check_inherit   = 1 << 21
```

These are internal - system functions check the right bit, users just use aggregate roles.

## Bootstrap

```rust
pub fn bootstrap() -> Result<(u64, u64)> {
    // role→mask mappings on _system
    set(OBJECTS, key(1, 1), ALL_BITS);     // _owner
    set(OBJECTS, key(1, 2), ADMIN_BITS);   // _admin
    set(OBJECTS, key(1, 3), EDITOR_BITS);  // _editor
    set(OBJECTS, key(1, 4), VIEWER_BITS);  // _viewer
    // _root gets _owner on _system
    set(SUBJECTS, key(2, 1), 1);
    Ok((1, 2))  // (_system, _root)
}
```

## API Functions

### OBJECTS table (role→mask mappings)

```
create(actor, object, role, mask)        # _create_role bit on object
delete(actor, object, role)              # _delete_role bit on object
update(actor, object, role, mask)        # _update_role bit on object
get_object(actor, object, role) -> mask  # _get_role bit on object
check_object(actor, object, role) -> bool# _check_role bit on object
```

### SUBJECTS table (grants)

```
grant(actor, subject, object, role)      # _grant bit on object
revoke(actor, subject, object)           # _revoke bit on object
get_subject(actor, subject, object) -> role    # _get_grant bit on object
check_subject(subject, object, role) -> bool   # _check_grant bit on object (no actor)
```

### INHERITS table (role-specific inheritance)

```
inherit(actor, subject, object, role, parent)  # _set_inherit bit on object
remove_inherit(actor, subject, object, role)   # _remove_inherit bit on object
get_inherit(actor, subject, object, role) -> parent  # _get_inherit bit on object
check_inherit(actor, subject, object, role) -> bool  # _check_inherit bit on object
```

### Resolution (no permission check)

```
get_mask(subject, object) -> u64         # resolve full mask with inheritance
check(subject, object, required) -> bool # check bits against resolved mask
```

### Bootstrap

```
bootstrap() -> (_system, _root)          # direct DB, no checks
```

## Resolution

```
get_mask(subject, object) -> u64:
    mask = 0
    current = subject
    loop max 10:
        role = SUBJECTS[current, object]
        if role:
            mask |= OBJECTS[object, role] or role
        parent = INHERITS[current, object, role]  # role-specific inheritance
        if parent:
            current = parent
        else:
            break
    return mask

check(subject, object, required) -> bool:
    return (get_mask(subject, object) & required) == required
```

## Workflow

```
1. bootstrap()
   → OBJECTS[1, 1] = ALL_BITS     # _system, _owner
   → OBJECTS[1, 2] = ADMIN_BITS   # _system, _admin
   → OBJECTS[1, 3] = EDITOR_BITS  # _system, _editor
   → OBJECTS[1, 4] = VIEWER_BITS  # _system, _viewer
   → SUBJECTS[2, 1] = 1           # _root has _owner on _system
   → returns (1, 2)

2. grant(_root, alice, _system, _admin)
   → check: get_mask(_root, _system) & GRANT_BIT? yes
   → SUBJECTS[alice, _system] = _admin

3. alice can now grant others on _system
   → get_mask(alice, _system) = ADMIN_BITS (has GRANT_BIT)
```

## Summary

- 3 tables: OBJECTS, SUBJECTS, INHERITS
- ~10 public constants
- 16 functions:
  - OBJECTS: create, delete, update, get_object, check_object (5)
  - SUBJECTS: grant, revoke, get_subject, check_subject (4)
  - INHERITS: inherit, remove_inherit, get_inherit, check_inherit (4)
  - Resolution: get_mask, check (2)
  - Bootstrap: bootstrap (1)
- Permission = bitmask check
- Roles map to masks via OBJECTS table
