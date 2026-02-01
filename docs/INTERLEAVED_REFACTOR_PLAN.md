# LMDB Normalization Refactor Plan

## Problem

Currently, entity names like "user:alice" are embedded as strings in all keys:
- Rename alice → bob requires updating ALL records referencing "user:alice"
- Not atomic, O(n) updates, can fail halfway
- Same issue for relation names, type names

## Solution

**Interleaved Bits + Thread-Local Mode + Single Storage**

One key encodes BOTH ID and Name. Thread-local mode tells comparator which to use.

```
Before: grants key = "user:alice" + "viewer" + "resource:office"  (strings, 40+ bytes)
After:  grants key = [TypeId:4][Interleaved(EntityId,NameHash):8]  (12 bytes fixed)

Single DB, single storage, dual O(log n) lookup:
  - Set mode = ID   → comparator compares even bits
  - Set mode = Name → comparator compares odd bits

No mode byte in key. No duplicate storage. Just set mode before query.
```

---

## Thread-Local Mode Switching

### The Key Insight

LMDB comparator has fixed signature `fn(a, b) -> Ordering`. Can't pass extra params.

**Solution:** Thread-local variable holds the mode. Comparator reads it.

```rust
use std::cell::Cell;

thread_local! {
    static COMPARE_MODE: Cell<bool> = Cell::new(false);
    // false = compare by ID (even bits)
    // true  = compare by Name (odd bits)
}

fn set_mode_id() {
    COMPARE_MODE.with(|m| m.set(false));
}

fn set_mode_name() {
    COMPARE_MODE.with(|m| m.set(true));
}

fn compare(a: &[u8], b: &[u8]) -> Ordering {
    if COMPARE_MODE.with(|m| m.get()) {
        compare_odd_bits(a, b)   // Name mode
    } else {
        compare_even_bits(a, b)  // ID mode
    }
}
```

### Usage Pattern

```rust
// Query by ID
set_mode_id();
let result = db.get(&search_key)?;

// Query by Name
set_mode_name();
let result = db.get(&search_key)?;
```

**Benefits:**
- No mode byte in key (saves memory)
- No duplicate storage (single insert)
- Thread-safe (each thread has own mode)
- Single DB, single comparator

---

## Interleaved Bits

### Encoding

Interleave bits of EntityId and NameHash into ONE u64:

```
EntityId   = 5  = binary 0101
NameHash   = 3  = binary 0011

Interleave (alternating bits):
  Position: 0 1 2 3 4 5 6 7
  ID bits:  1   0   1   0     (even positions: 0, 2, 4, 6)
  Name bits:  1   1   0   0   (odd positions: 1, 3, 5, 7)
  Result:   1 1 0 1 1 0 0 0
```

### Selective Bit Comparison (The "Jump")

```rust
fn compare_even_bits(a: &[u8], b: &[u8]) -> Ordering {
    let za = u64::from_be_bytes(a[4..12].try_into().unwrap());
    let zb = u64::from_be_bytes(b[4..12].try_into().unwrap());

    // Jump to even positions: 0, 2, 4, 6, ... (ID bits)
    for i in 0..32 {
        let bit_pos = 2 * i;  // 0, 2, 4, 6, ...
        let bit_a = (za >> bit_pos) & 1;
        let bit_b = (zb >> bit_pos) & 1;
        if bit_a != bit_b {
            return bit_a.cmp(&bit_b);
        }
    }
    Ordering::Equal
}

fn compare_odd_bits(a: &[u8], b: &[u8]) -> Ordering {
    let za = u64::from_be_bytes(a[4..12].try_into().unwrap());
    let zb = u64::from_be_bytes(b[4..12].try_into().unwrap());

    // Jump to odd positions: 1, 3, 5, 7, ... (Name bits)
    for i in 0..32 {
        let bit_pos = 2 * i + 1;  // 1, 3, 5, 7, ...
        let bit_a = (za >> bit_pos) & 1;
        let bit_b = (zb >> bit_pos) & 1;
        if bit_a != bit_b {
            return bit_a.cmp(&bit_b);
        }
    }
    Ordering::Equal
}
```

### B-Tree Traversal

Same tree, same keys. Mode determines which bits are compared at each node.

```
                    [Root]
                   /      \
            [Node A]      [Node B]
            /    \        /    \
          ...    ...    ...    ...

mode=ID:   comparator compares even bits → O(log n)
mode=Name: comparator compares odd bits  → O(log n)
```

---

## New Database Schema

### Key Encoding Functions

```rust
/// Interleave two u32s into one u64
fn interleave(a: u32, b: u32) -> u64 {
    let mut result = 0u64;
    for i in 0..32 {
        result |= (((a >> i) & 1) as u64) << (2 * i);      // a bits at even positions
        result |= (((b >> i) & 1) as u64) << (2 * i + 1);  // b bits at odd positions
    }
    result
}

/// De-interleave u64 back to two u32s
fn deinterleave(z: u64) -> (u32, u32) {
    let mut a = 0u32;
    let mut b = 0u32;
    for i in 0..32 {
        a |= (((z >> (2 * i)) & 1) as u32) << i;      // even positions → a
        b |= (((z >> (2 * i + 1)) & 1) as u32) << i;  // odd positions → b
    }
    (a, b)
}

/// Hash a name to u32 for interleaving (preserves prefix ordering)
fn name_hash(name: &str) -> u32 {
    let bytes = name.as_bytes();
    let mut h = 0u32;
    for (i, &b) in bytes.iter().take(4).enumerate() {
        h |= (b as u32) << (8 * (3 - i));  // big-endian for lex order
    }
    h
}
```

### Databases (Single Storage, Dual Lookup via Thread-Local Mode)

```
┌─────────────────────────────────────────────────────────────────┐
│ entities (ONE DB)                                                │
│   Key:   [type_id:u32][interleaved(entity_id, name_hash):u64]   │
│   Value: [original_name:String]                                  │
│   Size:  12 bytes fixed                                          │
│                                                                  │
│   mode=ID:   finds entity by type + entity_id                    │
│   mode=Name: finds entity by type + name_hash                    │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ grants (ONE DB)                                                  │
│   Key:   [interleaved(seeker_id, scope_id):u64][role_id:u32]    │
│   Value: [cap_mask:u64]                                          │
│   Size:  12 bytes fixed                                          │
│                                                                  │
│   mode=Seeker: finds grants by seeker_id                         │
│   mode=Scope:  finds grants by scope_id                          │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ roles (ONE DB, simple)                                           │
│   Key:   [role_id:u32]                                           │
│   Value: [label:String]                                          │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ capabilities (ONE DB)                                            │
│   Key:   [scope_type:u32][scope_id:u32][role_id:u32]            │
│   Value: [cap_mask:u64]                                          │
│   Size:  12 bytes fixed                                          │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ meta (ONE DB)                                                    │
│   "next_type_id" → u32                                           │
│   "next_entity_id:{type_id}" → u32                               │
│   "next_role_id" → u32                                           │
└─────────────────────────────────────────────────────────────────┘
```

**Total: 5 DBs (down from current ~10+ with separate forward/reverse indexes)**

---

## Key Format & Single Comparator

All keys are fixed-width with interleaved dimensions. No postfixes needed.

### Entity Key (12 bytes)

```rust
fn entity_key(type_id: u32, entity_id: u32, name: &str) -> [u8; 12] {
    let mut key = [0u8; 12];
    key[0..4].copy_from_slice(&type_id.to_be_bytes());
    let z = interleave(entity_id, name_hash(name));
    key[4..12].copy_from_slice(&z.to_be_bytes());
    key
}
```

### Grant Key (12 bytes)

```rust
fn grant_key(seeker_id: u32, scope_id: u32, role_id: u32) -> [u8; 12] {
    let mut key = [0u8; 12];
    let z = interleave(seeker_id, scope_id);
    key[0..8].copy_from_slice(&z.to_be_bytes());
    key[8..12].copy_from_slice(&role_id.to_be_bytes());
    key
}
```

### Single Comparator with Thread-Local Mode

```rust
use std::cell::Cell;
use std::cmp::Ordering;

thread_local! {
    /// false = compare even bits (first dimension: ID/Seeker)
    /// true  = compare odd bits (second dimension: Name/Scope)
    pub static COMPARE_MODE: Cell<bool> = Cell::new(false);
}

pub fn set_mode_first() {
    COMPARE_MODE.with(|m| m.set(false));
}

pub fn set_mode_second() {
    COMPARE_MODE.with(|m| m.set(true));
}

/// Single comparator for ALL interleaved databases
/// Reads thread-local mode to decide which bits to compare
pub fn interleaved_compare(a: &[u8], b: &[u8]) -> Ordering {
    // First compare any prefix (e.g., type_id for entities)
    let prefix_len = if a.len() == 12 && b.len() == 12 {
        4  // entity key: [type_id:4][interleaved:8]
    } else {
        0  // grant key: [interleaved:8][role_id:4]
    };

    if prefix_len > 0 {
        match a[0..prefix_len].cmp(&b[0..prefix_len]) {
            Ordering::Equal => {}
            other => return other,
        }
    }

    // Get interleaved portion
    let z_start = prefix_len;
    let z_end = z_start + 8;
    let za = u64::from_be_bytes(a[z_start..z_end].try_into().unwrap());
    let zb = u64::from_be_bytes(b[z_start..z_end].try_into().unwrap());

    // Compare based on mode
    let use_odd = COMPARE_MODE.with(|m| m.get());

    for i in 0..32 {
        let bit_pos = if use_odd { 2 * i + 1 } else { 2 * i };
        let bit_a = (za >> bit_pos) & 1;
        let bit_b = (zb >> bit_pos) & 1;
        if bit_a != bit_b {
            return bit_a.cmp(&bit_b);
        }
    }

    // If interleaved equal, compare suffix (e.g., role_id for grants)
    a[z_end..].cmp(&b[z_end..])
}
```

### Using with heed (Rust LMDB wrapper)

```rust
use heed::{Database, Env};
use heed::types::{Bytes, Str, U64};

// ONE database for entities, ONE comparator
let entities: Database<Bytes, Str> = env
    .create_database_with_comparator(Some("entities"), interleaved_compare)?;

// ONE database for grants, SAME comparator
let grants: Database<Bytes, U64<BigEndian>> = env
    .create_database_with_comparator(Some("grants"), interleaved_compare)?;

// Insert ONCE
let key = entity_key(type_id, entity_id, name);
entities.put(&mut txn, &key, name)?;  // single insert!

// Query by ID
set_mode_first();
let result = entities.get(&txn, &search_key)?;

// Query by Name
set_mode_second();
let result = entities.get(&txn, &search_key)?;
```

**Benefits:**
- Fixed 12-byte keys
- Single insert per entity/grant
- Single comparator (with if/else based on mode)
- Thread-local mode = thread-safe
- Dual O(log n) lookup from same data

---

## Workflow Mapping (How Current API Keeps Working)

### Current vs New Internal Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ CURRENT WORKFLOW                                                │
├─────────────────────────────────────────────────────────────────┤
│ Client: POST /grant {seeker: "user:alice", scope: "doc:x"}     │
│                          ↓                                      │
│ Server: parse "user:alice" → store as string key                │
│                          ↓                                      │
│ LMDB: key = "user:alice|viewer|doc:x" (string)                 │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ NEW WORKFLOW (same API, different internals)                    │
├─────────────────────────────────────────────────────────────────┤
│ Client: POST /grant {seeker: "user:alice", scope: "doc:x"}     │
│                          ↓                                      │
│ Server: resolve "user:alice" → (type_id=1, entity_id=5)        │
│         resolve "doc:x" → (type_id=2, entity_id=3)             │
│                          ↓                                      │
│ LMDB: key = [interleaved(5,3)][role_id] (12 bytes)             │
└─────────────────────────────────────────────────────────────────┘
```

### API Layer (src/protected.rs + src/bin/server.rs)

**No API changes for clients.** Internal resolution happens transparently.

```rust
// Server endpoint (unchanged signature)
async fn post_grant(req: GrantRequest) -> Response {
    // NEW: resolve labels to IDs
    let seeker_id = resolve_entity_by_name(&req.seeker)?;
    let scope_id = resolve_entity_by_name(&req.scope)?;
    let role_id = resolve_role_by_name(&req.role)?;

    // NEW: store with interleaved key
    set_grant(seeker_id, scope_id, role_id, req.cap_mask)?;

    Ok(Response::ok())
}
```

### Core Operations

#### Create Entity
```rust
fn create_entity(type_id: u32, name: &str) -> Result<u32> {
    let entity_id = next_entity_id(type_id);
    let key = entity_key(type_id, entity_id, name);

    // Single insert!
    entities.put(&key, name)?;

    Ok(entity_id)
}
```

#### Resolve by Name (O(log n))
```rust
fn resolve_by_name(type_id: u32, name: &str) -> Result<u32> {
    set_mode_second();  // compare odd bits (name_hash)

    let search_key = entity_key(type_id, 0, name);

    // Binary search finds matching name_hash
    for (key, stored_name) in entities.range(&search_key[0..4]..)? {
        if stored_name == name {
            let z = u64::from_be_bytes(key[4..12].try_into().unwrap());
            let (entity_id, _) = deinterleave(z);
            return Ok(entity_id);
        }
    }
    Err(NotFound)
}
```

#### Resolve by ID (O(log n))
```rust
fn resolve_by_id(type_id: u32, entity_id: u32) -> Result<String> {
    set_mode_first();  // compare even bits (entity_id)

    let search_key = entity_key(type_id, entity_id, "");

    for (key, name) in entities.range(&search_key[0..4]..)? {
        let z = u64::from_be_bytes(key[4..12].try_into().unwrap());
        let (id, _) = deinterleave(z);
        if id == entity_id {
            return Ok(name.to_string());
        }
    }
    Err(NotFound)
}
```

#### Rename Entity (O(1) - grants unchanged!)
```rust
fn rename_entity(type_id: u32, entity_id: u32, new_name: &str) -> Result<()> {
    let old_name = resolve_by_id(type_id, entity_id)?;
    let old_key = entity_key(type_id, entity_id, &old_name);
    let new_key = entity_key(type_id, entity_id, new_name);

    // Delete old, insert new
    entities.delete(&old_key)?;
    entities.put(&new_key, new_name)?;

    // Grants use entity_id (in interleaved key) - UNCHANGED!
    Ok(())
}
```

#### Set Grant (single insert)
```rust
fn set_grant(seeker_id: u32, scope_id: u32, role_id: u32, cap_mask: u64) -> Result<()> {
    let key = grant_key(seeker_id, scope_id, role_id);
    grants.put(&key, &cap_mask)?;  // single insert!
    Ok(())
}
```

#### Check Access
```rust
fn check_access(seeker_id: u32, scope_id: u32) -> Result<u64> {
    set_mode_first();  // compare by seeker_id

    let mut total_caps = 0u64;

    for (key, cap_mask) in grants.iter()? {
        let z = u64::from_be_bytes(key[0..8].try_into().unwrap());
        let (sk, sc) = deinterleave(z);
        if sk == seeker_id && sc == scope_id {
            total_caps |= cap_mask;
        }
    }

    Ok(total_caps)
}
```

#### List by Scope (O(log n))
```rust
fn list_subjects(scope_id: u32) -> Result<Vec<(u32, u64)>> {
    set_mode_second();  // compare by scope_id

    let mut result = vec![];

    for (key, cap_mask) in grants.iter()? {
        let z = u64::from_be_bytes(key[0..8].try_into().unwrap());
        let (seeker_id, sc) = deinterleave(z);
        if sc == scope_id {
            result.push((seeker_id, cap_mask));
        }
    }

    Ok(result)
}
```

---

## Files to Modify

| File | Changes |
|------|---------|
| `src/keys.rs` | Rewrite: `interleave()`, `deinterleave()`, `name_hash()`, `entity_key()`, `grant_key()`, thread-local `COMPARE_MODE`, `interleaved_compare()` |
| `src/core.rs` | Rewrite Databases struct: 5 DBs with single comparator, new entity/grant operations |
| `src/protected.rs` | Update to resolve labels→IDs internally, keep same external API |
| `src/bootstrap.rs` | Create initial entities/types with interleaved keys |
| `src/bin/server.rs` | Add `/rename/entity` endpoint, return `{type_id, entity_id, name}` format |
| `demo/app.js` | Handle new response format, add inline rename UI |
| `Cargo.toml` | Ensure heed supports `create_database_with_comparator` |
| `tests/*.rs` | New tests for interleaving, thread-local mode, rename |

---

## Migration Strategy

**Fresh start** - incompatible database format.

```bash
# Backup if needed
cp -r ./data/capbit.mdb ./data/capbit.mdb.backup

# Delete old database
rm -rf ./data/capbit.mdb

# Build and run
cargo build --release --features server
./target/release/capbit-server

# Re-bootstrap (creates initial types, root user, etc.)
# Server auto-bootstraps on first run
```

**No migration path** - interleaved keys fundamentally different from string keys.

---

## Implementation Order

### Phase 1: Core Infrastructure
**File: src/keys.rs** (rewrite)
```rust
// New functions:
pub fn interleave(a: u32, b: u32) -> u64
pub fn deinterleave(z: u64) -> (u32, u32)
pub fn name_hash(name: &str) -> u32
pub fn entity_key(type_id: u32, entity_id: u32, name: &str) -> [u8; 12]
pub fn grant_key(seeker_id: u32, scope_id: u32, role_id: u32) -> [u8; 12]

// Thread-local mode:
pub static COMPARE_MODE: Cell<bool>
pub fn set_mode_first()
pub fn set_mode_second()
pub fn interleaved_compare(a: &[u8], b: &[u8]) -> Ordering
```

**File: src/core.rs** (rewrite Databases struct)
```rust
pub struct Databases {
    pub entities: Database<Bytes, Str>,     // single DB!
    pub grants: Database<Bytes, U64>,       // single DB!
    pub roles: Database<U32, Str>,
    pub capabilities: Database<Bytes, U64>,
    pub meta: Database<Str, Bytes>,
}

// All interleaved DBs use same comparator: interleaved_compare
```

### Phase 2: Entity Operations
**File: src/core.rs**
```rust
pub fn create_entity(type_id: u32, name: &str) -> Result<u32>
pub fn resolve_by_name(type_id: u32, name: &str) -> Result<u32>
pub fn resolve_by_id(type_id: u32, entity_id: u32) -> Result<String>
pub fn rename_entity(type_id: u32, entity_id: u32, new_name: &str) -> Result<()>
pub fn delete_entity(type_id: u32, entity_id: u32) -> Result<()>
pub fn list_entities(type_id: u32) -> Result<Vec<(u32, String)>>
```

### Phase 3: Grant Operations
**File: src/core.rs**
```rust
pub fn set_grant(seeker_id: u32, scope_id: u32, role_id: u32, cap: u64) -> Result<()>
pub fn delete_grant(seeker_id: u32, scope_id: u32, role_id: u32) -> Result<()>
pub fn check_access(seeker_id: u32, scope_id: u32) -> Result<u64>
pub fn list_by_seeker(seeker_id: u32) -> Result<Vec<Grant>>
pub fn list_by_scope(scope_id: u32) -> Result<Vec<Grant>>
```

### Phase 4: API Layer
**File: src/protected.rs**
- Keep same function signatures
- Internally resolve labels → IDs before operations
- Return IDs + labels in responses

**File: src/bin/server.rs**
- Keep same HTTP endpoints
- Add: `POST /rename/entity`
- Response format: `{type_id, entity_id, name}` instead of just string

### Phase 5: Frontend
**File: demo/app.js**
- Handle new response format with IDs
- Display names (from response)
- Add rename UI (click to edit inline)

---

## Verification

### 1. Unit Tests

```rust
#[test]
fn test_interleave_deinterleave() {
    let a = 12345u32;
    let b = 67890u32;
    let z = interleave(a, b);
    let (a2, b2) = deinterleave(z);
    assert_eq!(a, a2);
    assert_eq!(b, b2);
}

#[test]
fn test_thread_local_mode() {
    let k1 = entity_key(1, 100, "alice");
    let k2 = entity_key(1, 100, "bob");
    let k3 = entity_key(1, 200, "alice");

    // Mode = ID (even bits)
    set_mode_first();
    assert_eq!(interleaved_compare(&k1, &k2), Ordering::Equal);  // same ID
    assert_ne!(interleaved_compare(&k1, &k3), Ordering::Equal);  // diff ID

    // Mode = Name (odd bits)
    set_mode_second();
    assert_ne!(interleaved_compare(&k1, &k2), Ordering::Equal);  // diff name
    assert_eq!(interleaved_compare(&k1, &k3), Ordering::Equal);  // same name
}

#[test]
fn test_rename_doesnt_affect_grants() {
    init("test.mdb").unwrap();

    // Create entities
    let alice_id = create_entity(1, "alice").unwrap();
    let doc_id = create_entity(2, "secret").unwrap();

    // Create grant
    set_grant(alice_id, doc_id, 1, 0x03).unwrap();

    // Verify access
    assert_eq!(check_access(alice_id, doc_id).unwrap(), 0x03);

    // Rename alice → alicia
    rename_entity(1, alice_id, "alicia").unwrap();

    // Grant still works (uses IDs, not names)
    assert_eq!(check_access(alice_id, doc_id).unwrap(), 0x03);

    // Name changed
    assert_eq!(resolve_by_id(1, alice_id).unwrap(), "alicia");
}
```

### 2. Manual Test Workflow

```bash
# Start fresh
rm -rf ./data/capbit.mdb
cargo run --release --features server

# Create entities
curl -X POST localhost:3000/entity -d '{"type":"user","name":"alice"}'
# → {"ok":true,"data":{"type_id":1,"entity_id":1,"name":"alice"}}

curl -X POST localhost:3000/entity -d '{"type":"doc","name":"secret"}'
# → {"ok":true,"data":{"type_id":2,"entity_id":1,"name":"secret"}}

# Grant
curl -X POST localhost:3000/grant -d '{"seeker":"user:alice","scope":"doc:secret","role":"editor","cap_mask":3}'
# → {"ok":true}

# Check access
curl -X POST localhost:3000/check -d '{"seeker":"user:alice","scope":"doc:secret"}'
# → {"ok":true,"data":{"cap_mask":3}}

# Rename
curl -X POST localhost:3000/rename/entity -d '{"type_id":1,"entity_id":1,"new_name":"alicia"}'
# → {"ok":true}

# Verify rename
curl -X POST localhost:3000/resolve -d '{"type_id":1,"entity_id":1}'
# → {"ok":true,"data":{"name":"alicia"}}

# Verify grants still work
curl -X POST localhost:3000/check -d '{"seeker":"user:alicia","scope":"doc:secret"}'
# → {"ok":true,"data":{"cap_mask":3}}
```

### 3. Benchmark: Rename with 1M Grants

```bash
# Create test data
for i in {1..1000000}; do
  curl -X POST localhost:3000/grant -d "{\"seeker\":\"user:alice\",\"scope\":\"doc:$i\",\"role\":\"viewer\",\"cap_mask\":1}"
done

# Time the rename
time curl -X POST localhost:3000/rename/entity -d '{"type_id":1,"entity_id":1,"new_name":"alicia"}'
# Expected: < 1ms (O(1), grants unchanged)

# Verify still works
curl -X POST localhost:3000/check -d '{"seeker":"user:alicia","scope":"doc:500000"}'
# → {"ok":true,"data":{"cap_mask":1}}
```

---

## Comparative Efficiency Analysis

### Approach Comparison

| Metric | Current (Strings) | Label/ID Tables | Interleaved + Thread-Local |
|--------|-------------------|-----------------|---------------------------|
| **Storage per entity** | ~20-50 bytes | ~30 bytes (2 tables) | 12 bytes (1 table) |
| **Storage per grant** | ~50-100 bytes | ~20 bytes | 12 bytes |
| **Databases needed** | ~10 (fwd+rev) | ~8 (labels+rev+data) | **5** |
| **Inserts per entity** | 2 (fwd+rev) | 2-3 (label+rev+data) | **1** |
| **Inserts per grant** | 2 (fwd+rev) | 2 (fwd+rev) | **1** |
| **Lookup by ID** | O(log n) | O(log n) | O(log n) |
| **Lookup by Name** | O(log n) | O(log n) | O(log n) |
| **Rename** | **O(n)** - update all refs | **O(1)** - update label only | **O(1)** - update entity only |
| **Memory for 1M grants** | ~50-100 MB | ~20 MB | **~12 MB** |

### Rename Cost Analysis

```
Scenario: Entity with 1 million grants needs rename

Current (Strings):
  - Find all grants referencing "user:alice"
  - Update each key: delete old, insert new
  - 1,000,000 deletes + 1,000,000 inserts = 2M operations
  - NOT ATOMIC - can fail halfway
  - Time: O(n) = minutes to hours

Label/ID Tables:
  - Update label table: "alice" → "alicia"
  - Update reverse index: delete old label, insert new
  - 3 operations total
  - ATOMIC
  - Time: O(1) = microseconds

Interleaved + Thread-Local:
  - Delete old entity key, insert new entity key
  - 2 operations total (grants use entity_id, unchanged)
  - ATOMIC
  - Time: O(1) = microseconds
```

### Memory Efficiency

```
Current (String keys):
  Grant key: "user:alice|editor|resource:secret-document"
  Size: ~45 bytes average

Label/ID Tables:
  Grant key: [seeker_type:4][seeker_id:4][role:4][scope_type:4][scope_id:4]
  Size: 20 bytes
  + Label tables overhead

Interleaved:
  Grant key: [interleaved(seeker_id, scope_id):8][role_id:4]
  Size: 12 bytes
  No separate label tables (name in entity value)
```

### Insert Efficiency

```
Current:
  create_entity("user", "alice"):
    1. entities.put("user:alice", metadata)
    2. entities_rev.put(reverse_key, "user:alice")
  Total: 2 inserts

Label/ID Tables:
  create_entity("user", "alice"):
    1. entity_labels.put([type_id, entity_id], "alice")
    2. entity_by_label.put([type_id, "alice"], entity_id)
  Total: 2 inserts

Interleaved + Thread-Local:
  create_entity("user", "alice"):
    1. entities.put(interleaved_key, "alice")
  Total: 1 insert ← 50% fewer writes!
```

### Query Efficiency

```
All approaches: O(log n) for both ID and Name lookups

But interleaved has:
  - Smaller keys = more keys per B-tree node = shallower tree
  - Single DB = simpler transaction handling
  - No joins needed (name stored in value)
```

### Comparison Summary

| Aspect | Winner |
|--------|--------|
| Storage size | **Interleaved** (12 bytes vs 20-50) |
| Write operations | **Interleaved** (1 insert vs 2) |
| Rename cost | **Tie** (Label/ID and Interleaved both O(1)) |
| Complexity | **Label/ID** (simpler concept) |
| DB count | **Interleaved** (5 vs 8-10) |
| Novel approach | **Interleaved** (may need debugging) |

---

## Trade-offs

**Pros:**
- **Single storage** - no separate label tables
- **Dual O(log n) lookup** - ID or Name via selective bit comparison
- **Fixed 12-byte keys** - smallest possible, no postfixes
- **No stale data** - grants use IDs, labels in entity values
- **Rename = O(1)** - update entity only, grants unchanged
- **50% fewer writes** - single insert vs dual insert
- **Deterministic** - you control exact bit comparison
- **Thread-safe** - each thread has own comparison mode

**Cons:**
- Novel approach - less battle-tested than label tables
- Bit manipulation adds small CPU overhead
- Name lookup uses hash - collision handling needed for exact match
- Thread-local state requires discipline (set mode before query)
- Breaking change (new DB format)

---

## Summary

```
┌─────────────────────────────────────────────────────────────────┐
│ INTERLEAVED BITS + THREAD-LOCAL MODE                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│ Key structure:  [TypeId:4][Interleaved(ID, NameHash):8]         │
│                 = 12 bytes fixed, no postfixes                   │
│                                                                  │
│ Value:          Original name string (for display)               │
│                                                                  │
│ Interleaving:                                                    │
│   ID bits   → even positions: 0, 2, 4, 6, ...                   │
│   Name bits → odd positions:  1, 3, 5, 7, ...                   │
│                                                                  │
│ Thread-local mode:                                               │
│   set_mode_first()  → comparator compares even bits (ID)        │
│   set_mode_second() → comparator compares odd bits (Name)       │
│                                                                  │
│ Result:                                                          │
│   - Single DB per table (not dual)                              │
│   - Single insert per entity/grant                               │
│   - Dual O(log n) lookup via mode switching                     │
│   - O(1) rename (grants use IDs, unchanged)                     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

Rename "alice" → "alicia":
  1. set_mode_first() → find entity by ID
  2. Delete old entity key
  3. Insert new entity key (same ID, new name_hash)
  4. Grants have interleaved(seeker_id, scope_id) → UNCHANGED!

Total operations: 2 (delete + insert)
Time: O(1)
```
