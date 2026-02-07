# Capbit v2: Modal Authorization Architecture

## Overview

Upgrade capbit from flat bitmask authorization to a **modally-qualified three-tuple architecture**. Every relationship, delegation, and permission gains a modal operator (□ necessary, ◇ possible, ¬ deny) that qualifies the *strength* of the authorization — unifying MAC, DAC, and explicit deny in a single model while preserving capbit's core principle: authorization as atomized data.

## Current → New Mapping

### Current Architecture (v0.5)

```
SUBJECTS:   (subject, object, role) → 1         // flat grant
INHERITS:   (subject, object, role) → parent     // single-parent delegation
OBJECTS:    (object, role) → mask                // permission bitmask
```

Three concerns, three partitions, but no modality. Every grant is equally strong. No deny. No conditional access. Inheritance is a parent pointer, not a qualified delegation.

### New Architecture (v2)

```
Tuple 1 — RELATIONS:    (subject, object, context, modal) → 1
Tuple 2 — DELEGATIONS:  (subject, object, context, modal, target) → 1
Tuple 3 — PERMISSIONS:  (object, context, modal) → mask
```

Same three concerns, same atomized tuples, now with modal qualification at every level.

### Field Mapping

| v0.5 | v2 | Notes |
|------|-----|-------|
| `role` (u64) | `context` (u64) | Rename. Same type. Context is the broader concept — a role is one kind of context. |
| *(absent)* | `modal` (u8) | New field. □=0, ◇=1, ¬=2. |
| `parent` (u64 value) | `target` (u64 in key) | Delegation target moves from value to key. Enables multiple delegations per (subject, object, context). |
| `mask` (u64 value) | `mask` (u64 value) | Unchanged. Capbit keeps bitmask efficiency. |

## Modal Encoding

```rust
// Modal operators
pub const NECESSARY: u8 = 0;  // □ — structural, mandatory, MAC
pub const POSSIBLE: u8  = 1;  // ◇ — discretionary, conditional, DAC
pub const DENY: u8      = 2;  // ¬ — explicit prohibition

// Extended modals (future)
pub const QUORUM: u8    = 3;  // ◇≥k — threshold/multi-party (k stored in value)
```

u8 keeps keys compact. The modal byte sits between the context and any trailing fields, maintaining prefix-scan efficiency on (subject, object) and (object, context).

## Key Encoding

### Tuple 1 — RELATIONS

```
Key:   [subject:8][object:8][context:8][modal:1]  = 25 bytes
Value: [1:8]                                       = 8 bytes (marker)
```

Prefix scan on `[subject:8][object:8]` returns all contexts and modalities for a subject+object pair — same scan pattern as current SUBJECTS.

### Tuple 2 — DELEGATIONS

```
Key:   [subject:8][object:8][context:8][modal:1][target:8]  = 33 bytes
Value: [1:8]                                                 = 8 bytes (marker)
```

Replaces INHERITS. Key differences from v0.5:
- **modal in key**: each delegation edge is modally qualified
- **target in key, not value**: enables multiple delegations from one subject on the same (object, context) — critical for ◇≥k quorum semantics
- No more single-parent restriction

### Tuple 3 — PERMISSIONS

```
Key:   [object:8][context:8][modal:1]  = 17 bytes
Value: [mask:8]                        = 8 bytes
```

Replaces OBJECTS. Each (object, context) can have up to three entries — one per modality:

```
(doc:100, EDITOR, □) → READ|WRITE|COMMENT       // editors necessarily can rwc
(doc:100, EDITOR, ◇) → DELETE                    // editors possibly can delete
(doc:100, EDITOR, ¬) → ADMIN                     // editors necessarily cannot admin
```

Same bitmask efficiency. Different masks per modality.

## Reverse Indexes

```
RELATIONS_REV:      (object, subject, context, modal) → 1          // "who has access to this object?"
DELEGATIONS_REV:    (target, object, context, modal, source) → 1   // "who delegated to me?"
DELEGATIONS_BY_OBJ: (object, context, modal, source, target) → 1  // "all delegations on this object"
```

Same pattern as current SUBJECTS_REV, INHERITS_BY_OBJ, INHERITS_BY_PARENT — reverse indexes for efficient queries in both directions.

## Resolution Algorithm

### Current (v0.5)

```rust
fn get_mask(sub, obj) -> u64 {
    mask = 0
    walk subject inheritance chain (max 10 hops)
    for each hop: mask |= OBJECTS.get(obj, role)
    return mask  // flat OR accumulation
}
```

### New (v2)

```rust
fn get_mask(sub: u64, obj: u64) -> Result<ModalMask> {
    let mut necessary: u64 = 0;   // □ bits — guaranteed
    let mut possible: u64 = 0;    // ◇ bits — conditional
    let mut denied: u64 = 0;      // ¬ bits — prohibited

    // Step 1: Collect all direct relations
    for (context, modal) in relations.prefix(sub, obj) {

        // Step 2: Get permission mask for this (object, context, modal)
        // and compose with relationship modality
        for perm_modal in [NECESSARY, POSSIBLE, DENY] {
            if let Some(mask) = permissions.get(obj, context, perm_modal) {
                let effective = compose(modal, perm_modal);
                match effective {
                    NECESSARY => necessary |= mask,
                    POSSIBLE  => possible |= mask,
                    DENY      => denied |= mask,
                }
            }
        }
    }

    // Step 3: Follow delegation chains
    let mut visited = HashSet::new();
    let mut stack: Vec<(u64, u8, u8)> = vec![];  // (source, depth, chain_modal)

    // Seed stack from delegations targeting this subject
    for (source, context, modal) in delegations_rev.prefix(sub, obj) {
        stack.push((source, 1, modal));
    }

    while let Some((cur, depth, chain_modal)) = stack.pop() {
        if depth > max_depth || visited.contains(&cur) { continue; }
        visited.insert(cur);

        // Get source's relation modality
        for (context, rel_modal) in relations.prefix(cur, obj) {
            let composed = compose(compose(rel_modal, chain_modal), perm_modal);
            // ... accumulate into necessary/possible/denied
        }

        // Follow further delegations
        for (source, del_modal) in delegations_rev.prefix(cur, obj) {
            let new_chain = compose(chain_modal, del_modal);
            if new_chain != DENY {
                stack.push((source, depth + 1, new_chain));
            }
        }
    }

    // Step 4: Apply deny override
    necessary &= !denied;
    possible &= !denied;

    Ok(ModalMask { necessary, possible, denied })
}
```

### Modal Composition

```rust
fn compose(a: u8, b: u8) -> u8 {
    if a == DENY || b == DENY { return DENY; }
    if a == NECESSARY && b == NECESSARY { return NECESSARY; }
    POSSIBLE  // anything else degrades to possible
}
```

The lattice: `□ > ◇ > ¬`. Composition takes the minimum. Deny absorbs everything.

### Return Type

```rust
pub struct ModalMask {
    pub necessary: u64,  // □ — guaranteed permissions
    pub possible: u64,   // ◇ — conditional permissions
    pub denied: u64,     // ¬ — prohibited permissions
}

impl ModalMask {
    /// Check if a permission is necessarily granted
    pub fn check_necessary(&self, req: u64) -> bool {
        self.necessary & req == req && self.denied & req == 0
    }

    /// Check if a permission is at least possible
    pub fn check_possible(&self, req: u64) -> bool {
        (self.necessary | self.possible) & req == req && self.denied & req == 0
    }

    /// Check if a permission is denied
    pub fn is_denied(&self, req: u64) -> bool {
        self.denied & req != 0
    }

    /// Flat check (v0.5 compat): necessary OR possible, minus denied
    pub fn effective(&self) -> u64 {
        (self.necessary | self.possible) & !self.denied
    }
}
```

## API Changes

### v0.5 → v2 Migration

| v0.5 | v2 | Change |
|------|-----|--------|
| `grant(actor, sub, obj, role)` | `relate(actor, sub, obj, ctx, modal)` | Added modal parameter |
| `revoke(actor, sub, obj, role)` | `unrelate(actor, sub, obj, ctx, modal)` | Added modal parameter |
| `inherit(actor, sub, obj, role, parent)` | `delegate(actor, sub, obj, ctx, modal, target)` | Renamed, parent→target, added modal |
| `remove_inherit(actor, sub, obj, role)` | `undelegate(actor, sub, obj, ctx, modal, target)` | Target now required (multiple delegations possible) |
| `create(actor, obj, role, mask)` | `set_permission(actor, obj, ctx, modal, mask)` | Added modal, idempotent (no create/update split) |
| `delete(actor, obj, role)` | `remove_permission(actor, obj, ctx, modal)` | Added modal |
| `check(sub, obj, req)` | `check(sub, obj, req)` | Same signature, returns effective (compat) |
| `get_mask(sub, obj)` | `get_modal_mask(sub, obj)` | Returns ModalMask with □/◇/¬ |
| — | `deny(actor, sub, obj, ctx)` | New: explicit deny relation |
| — | `check_necessary(sub, obj, req)` | New: strict □-only check |

### Backward Compatibility

```rust
// v0.5 compat wrappers
pub fn grant(actor: u64, sub: u64, obj: u64, role: u64) -> Result<()> {
    self.relate(actor, sub, obj, role, NECESSARY)
}

pub fn check(sub: u64, obj: u64, req: u64) -> Result<bool> {
    let mm = self.get_modal_mask(sub, obj)?;
    Ok(mm.effective() & req == req)
}

pub fn get_mask(sub: u64, obj: u64) -> Result<u64> {
    Ok(self.get_modal_mask(sub, obj)?.effective())
}
```

All existing v0.5 behavior works unchanged through compat wrappers. Every v0.5 grant is implicitly □ (necessary).

## Partition Layout (v2)

```
RELATIONS:          (subject, object, context, modal) → 1
RELATIONS_REV:      (object, subject, context, modal) → 1
DELEGATIONS:        (subject, object, context, modal, target) → 1
DELEGATIONS_REV:    (target, object, context, modal, source) → 1
DELEGATIONS_BY_OBJ: (object, context, modal, source, target) → 1
PERMISSIONS:        (object, context, modal) → mask
CONFIG:             key_bytes → u64
```

7 partitions (up from 6). Same fjall/LSM-tree backend.

## Key Builder Changes

```rust
// New key builders for modal keys
#[inline]
fn key3m(a: u64, b: u64, c: u64, m: u8) -> [u8; 25] {
    let mut x = [0u8; 25];
    x[..8].copy_from_slice(&a.to_be_bytes());
    x[8..16].copy_from_slice(&b.to_be_bytes());
    x[16..24].copy_from_slice(&c.to_be_bytes());
    x[24] = m;
    x
}

#[inline]
fn key3m1(a: u64, b: u64, c: u64, m: u8, d: u64) -> [u8; 33] {
    let mut x = [0u8; 33];
    x[..8].copy_from_slice(&a.to_be_bytes());
    x[8..16].copy_from_slice(&b.to_be_bytes());
    x[16..24].copy_from_slice(&c.to_be_bytes());
    x[24] = m;
    x[25..33].copy_from_slice(&d.to_be_bytes());
    x
}

#[inline]
fn key2m(a: u64, b: u64, m: u8) -> [u8; 17] {
    let mut x = [0u8; 17];
    x[..8].copy_from_slice(&a.to_be_bytes());
    x[8..16].copy_from_slice(&b.to_be_bytes());
    x[16] = m;
    x
}
```

## Permission Bits Update

```rust
// Existing bits remain unchanged (0-21)

// New bits for v2 operations
const _SET_RELATION: u64     = 1 << 14;    // was _GRANT
const _REMOVE_RELATION: u64  = 1 << 15;    // was _REVOKE
const _SET_DELEGATION: u64   = 1 << 18;    // was _SET_INHERIT
const _REMOVE_DELEGATION: u64 = 1 << 19;   // was _REMOVE_INHERIT
const _SET_DENY: u64         = 1 << 22;    // new
const _REMOVE_DENY: u64     = 1 << 23;     // new

// Compat aliases
const _GRANT: u64  = _SET_RELATION;
const _REVOKE: u64 = _REMOVE_RELATION;
```

## What This Gains Over v0.5

| Capability | v0.5 | v2 |
|-----------|------|-----|
| Flat grants | Yes | Yes (□) |
| Explicit deny | No | Yes (¬) |
| Conditional/temporary access | No | Yes (◇) |
| MAC/DAC unification | No | Yes (□ vs ◇) |
| Multiple delegations per context | No (single parent) | Yes (target in key) |
| Modal-qualified delegation | No | Yes |
| Three-tier permission response | No (binary allow/deny) | Yes (necessary/possible/denied) |
| Deny-override | No | Yes (¬ absorbs □ and ◇) |
| Bitmask efficiency | Yes | Yes (unchanged) |
| Atomized semantics | Yes | Yes (unchanged) |

## What This Gains Over Fong's ReBAC

| Fong Limitation | v2 Status |
|------|------|
| Bisimulation / counting barrier | Avoided — explicit tuples, not graph formulas |
| Monotonicity constraint | Solved — ¬ is explicit denial, not inference from absence |
| Graph traversal complexity | Solved — join-based evaluation |
| No native deny | Solved — ¬ as first-class modal |
| Exponential formula blowup | Avoided — counting at query layer |
| No MAC/DAC unification | Solved — □ vs ◇ |
| Single-owner assumption | Solved — tuple multiplicity |
| RBAC interoperability | Subsumed — RBAC is context assignment + permission lookup |

## Extended Modals (Future)

The u8 modal field supports up to 256 operator types. Planned extensions:

```rust
pub const QUORUM: u8     = 3;   // ◇≥k — value stores k
pub const TEMPORAL: u8   = 4;   // □_until — value stores expiry timestamp
pub const CONDITIONAL: u8 = 5;  // →{cond} — value stores condition reference
```

These fit in the same key structure. The modal byte selects the operator, and the value field can carry operator-specific data (k for quorum, timestamp for temporal).

## Implementation Order

### Phase 1: Core Modal Tuples
1. Add modal constants and `ModalMask` struct
2. New key builders (`key2m`, `key3m`, `key3m1`)
3. Replace SUBJECTS → RELATIONS partition with modal key
4. Replace OBJECTS → PERMISSIONS partition with modal key
5. Replace INHERITS → DELEGATIONS partition with modal key + target in key
6. Update reverse indexes
7. Backward-compat wrappers for existing API

### Phase 2: Resolution Engine
1. New `get_modal_mask()` with three-bucket accumulation
2. Modal composition function
3. Delegation chain walker with modal propagation
4. Deny-override application
5. `check_necessary()` and `check_possible()` methods
6. Compat `check()` and `get_mask()` using `effective()`

### Phase 3: API Surface
1. `relate()` / `unrelate()` with modal parameter
2. `delegate()` / `undelegate()` with modal + target
3. `deny()` convenience method (relate with ¬)
4. `set_permission()` / `remove_permission()` with modal
5. List queries updated for modal dimension
6. UI update for modal fields

### Phase 4: Extended Modals
1. ◇≥k quorum support
2. Temporal modals (□_until)
3. Compound modals (∧, ∨)

## Migration

### Data Migration (v0.5 → v2)

All v0.5 data is a subset of v2 with `modal = NECESSARY (0)`:

```rust
fn migrate_v05_to_v2(old: &Capbit, new: &CapbitV2) -> Result<()> {
    // SUBJECTS → RELATIONS with modal=□
    for (sub, obj, role) in old.scan_all_subjects()? {
        new.relate_internal(sub, obj, role, NECESSARY)?;
    }

    // OBJECTS → PERMISSIONS with modal=□
    for (obj, role, mask) in old.scan_all_objects()? {
        new.set_permission_internal(obj, role, NECESSARY, mask)?;
    }

    // INHERITS → DELEGATIONS with modal=□
    for (sub, obj, role, parent) in old.scan_all_inherits()? {
        new.delegate_internal(sub, obj, role, NECESSARY, parent)?;
    }

    Ok(())
}
```

Lossless. Every v0.5 fact becomes a v2 fact with □ modality.

## Summary

Capbit v2 adds a single byte (modal operator) to every tuple, gaining:
- Three-valued authorization (necessary / possible / denied)
- MAC + DAC + deny in one model
- Multi-target delegation
- Modal composition through chains
- All of Fong's ReBAC expressiveness without his limitations

While preserving:
- Bitmask efficiency
- Atomized semantics
- O(1) permission lookups
- Prefix-scan query patterns
- Full backward compatibility
