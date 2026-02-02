# Unified Planner Refactor

## Problem Statement
The codebase has grown to ~970 LOC across 10 modules with reads and writes split:
- Reads: `db::read()` creates NEW transaction per call
- Writes: Go through planner OR direct `transact()`
- No transaction reuse, duplicated code, unnecessary complexity

## Current Architecture

```
check() → get_mask() → read() → NEW RoTxn → resolve()
grant() → require() → check() → NEW RoTxn   (read to verify permission)
       → planner.submit()                    (write batched separately)
```

### Current Files & LOC:
| File | LOC | Purpose |
|------|-----|---------|
| lib.rs | 49 | Exports only |
| db.rs | 181 | BiPair, Dbs, init, read() |
| planner.rs | 258 | Write batching |
| tx.rs | 154 | Tx struct, transact() |
| read.rs | 104 | Read operations |
| write.rs | 78 | Protected write API |
| entity.rs | 25 | Entity wrappers |
| bootstrap.rs | 47 | Bootstrap |
| error.rs | 22 | Error types |
| constants.rs | 53 | Constants |
| **Total** | **971** | |

## Proposed Architecture

**Unified Planner** handles ALL operations with shared transaction pool:

```
┌─────────────────────────────────────────┐
│              PLANNER                     │
│  ┌─────────┐    ┌──────────────────┐    │
│  │ RoTxn   │    │  Write Batch     │    │
│  │ (cached)│    │  (grants, roles) │    │
│  └─────────┘    └──────────────────┘    │
│       ↑              ↑                  │
│       │              │                  │
│    reads          writes                │
└───────┴──────────────┴──────────────────┘
        ↑              ↑
   check()         grant()
   get_mask()      revoke()
   get_label()     set_role()
```

### New Structure:
| File | Est. LOC | Purpose |
|------|----------|---------|
| lib.rs | 40 | Exports |
| db.rs | 100 | BiPair, Dbs, init (no read()) |
| planner.rs | 350 | **Unified**: reads + writes + API |
| error.rs | 22 | Unchanged |
| constants.rs | 53 | Unchanged |
| **Total** | **~565** | **-400 LOC** |

## Implementation Plan

### Step 1: Extend Planner with Read Transaction Cache
Add to planner.rs:
```rust
struct Planner {
    tx: Sender<Op>,
    read_txn: Mutex<Option<RoTxn<'static>>>,  // Cached read transaction
    // ...
}
```

### Step 2: Add Read Operations to Planner
```rust
impl Planner {
    fn read<T>(&self, f: impl FnOnce(&Dbs, &RoTxn) -> Result<T>) -> Result<T> {
        let dbs = dbs()?;
        let txn = self.get_or_create_read_txn()?;
        f(dbs, &txn)
    }

    fn check(&self, subject: u64, object: u64, required: u64) -> Result<bool>;
    fn get_mask(&self, subject: u64, object: u64) -> Result<u64>;
    // ... all read ops
}
```

### Step 3: Move All Operations into Planner
Merge from read.rs, write.rs, entity.rs, bootstrap.rs into planner.rs:
- `check()`, `get_mask()`, `get_role()`, etc.
- `grant()`, `revoke()`, `set_role()`, etc.
- `create_entity()`, `set_label()`, etc.
- `bootstrap()`, `is_bootstrapped()`

### Step 4: Simplify Public API
In lib.rs, all operations delegate to planner:
```rust
pub fn check(s: u64, o: u64, r: u64) -> Result<bool> {
    planner()?.check(s, o, r)
}
```

### Step 5: Remove Redundant Files
Delete:
- read.rs (merged into planner)
- write.rs (merged into planner)
- entity.rs (merged into planner)
- bootstrap.rs (merged into planner)
- tx.rs (Tx struct moved into planner, transact() becomes internal)

### Step 6: Simplify db.rs
Remove:
- `read()` function (planner handles this)
- `planner()` function (planner is self-contained)

Keep only:
- `key()`, BiPair, Dbs
- `init()`, `clear_all()`, `test_lock()`
- Global statics

## Key Design Decisions

1. **Read transaction caching**: Direct access via `Mutex<Option<RoTxn>>`, not through planner thread
   - Reads bypass planner thread entirely for best performance
   - Mutex only held briefly during txn access
   - Transaction refreshed when stale (after write flush)
2. **Write batching unchanged**: Same adaptive batching logic, same thread
3. **transact() becomes internal**: Only planner uses it for flush
4. **Permission checks inside planner**: No separate require() functions
5. **Parallelism**: Multiple readers can proceed concurrently with writer thread

## Migration Path

1. Add read support to planner (non-breaking)
2. Move functions one by one, updating tests
3. Remove empty files
4. Update exports in lib.rs

## Verification

Run full test suite after each step:
```bash
cargo test --release --test benchmarks -- --ignored
cargo test --release --test lib_test
```

Check benchmarks for:
- Check throughput should improve (transaction reuse)
- Grant throughput should stay same or improve
- No correctness regressions
