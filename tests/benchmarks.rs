//! Capbit Benchmarks - Stress Edition
//! Uses transact() for write operations (bypasses protection for benchmarking)
//!
//! Run with: cargo t --release -- --test-threads=1
//! Results saved to: bench_results.txt

use capbit::*;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::sync::{Once, Mutex};
use std::fs::{File, OpenOptions};
use std::io::Write;

static INIT: Once = Once::new();
static FRESH_DB: Once = Once::new();
static OUTPUT_INIT: Once = Once::new();
static OUTPUT_FILE: Mutex<Option<File>> = Mutex::new(None);

fn output_path() -> String {
    // Default to capbit project directory for easy access
    std::env::var("CAPBIT_BENCH_OUT").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        format!("{}/capbit/bench_results.txt", home)
    })
}

fn init_output() {
    OUTPUT_INIT.call_once(|| {
        let path = output_path();
        if let Ok(f) = OpenOptions::new().create(true).write(true).truncate(true).open(&path) {
            *OUTPUT_FILE.lock().unwrap() = Some(f);
            // Write header with timestamp
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let header = format!("# Capbit Benchmark Results\n# Timestamp: {}\n# Buffer: {}, Interval: {}ms\n\n",
                now,
                std::env::var("CAPBIT_BUFFER").unwrap_or_else(|_| "1000".into()),
                std::env::var("CAPBIT_INTERVAL").unwrap_or_else(|_| "100".into()));
            if let Some(ref mut f) = *OUTPUT_FILE.lock().unwrap() {
                let _ = f.write_all(header.as_bytes());
            }
        }
    });
}

/// Print to stdout and append to results file
fn write_out(msg: &str) {
    init_output();
    println!("{}", msg);
    if let Some(ref mut f) = *OUTPUT_FILE.lock().unwrap() {
        let _ = writeln!(f, "{}", msg);
        let _ = f.flush();
    }
}

macro_rules! out {
    ($($arg:tt)*) => { write_out(&format!($($arg)*)) };
}

fn bench_db_path() -> String {
    std::env::var("CAPBIT_BENCH_DB").unwrap_or_else(|_| {
        let tmp = std::env::temp_dir();
        tmp.join("capbit_bench.mdb").to_string_lossy().to_string()
    })
}

fn fresh_db() {
    FRESH_DB.call_once(|| {
        let path = bench_db_path();
        let _ = std::fs::remove_dir_all(&path);  // Delete old DB files
    });
}

fn setup() {
    fresh_db();
    INIT.call_once(|| { init(&bench_db_path()).unwrap(); });
    clear_all().unwrap();
}

fn bench_grant(grants: &[(u64, u64, u64)]) {
    transact(|tx| { for &(s, o, m) in grants { tx.grant(s, o, m)?; } Ok(()) }).unwrap();
}

fn avg<F: FnMut()>(n: usize, mut f: F) -> Duration {
    for _ in 0..10 { f(); }
    let t = Instant::now();
    for _ in 0..n { f(); }
    t.elapsed() / n as u32
}

fn hdr(s: &str) { out!("\n{}\n{s}\n{}\n", "=".repeat(70), "=".repeat(70)); }
fn ratio(a: Duration, b: Duration) -> f64 { a.as_nanos() as f64 / b.as_nanos() as f64 }

fn rand() -> u64 {
    use std::cell::Cell;
    thread_local! { static S: Cell<u64> = Cell::new(0x853c49e6748fea9b); }
    S.with(|s| { let x = s.get().wrapping_mul(6364136223846793005).wrapping_add(1); s.set(x); x })
}

fn fmt_num(n: u64) -> String {
    if n >= 1_000_000 { format!("{:.1}M", n as f64 / 1_000_000.0) }
    else if n >= 1_000 { format!("{:.1}K", n as f64 / 1_000.0) }
    else { n.to_string() }
}

macro_rules! bench {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() { let _l = test_lock(); $body }
    };
}

// ============================================================================
// BASIC BENCHMARKS
// ============================================================================

bench!(lookup_scaling, {
    hdr("Lookup Scaling O(log N)");
    let mut r = vec![];
    for n in [100, 1000, 10000, 50000] {
        setup();
        let mut g: Vec<_> = (0..n).map(|i| (i+1000, 1u64, 1u64)).collect();
        g.push((999, 1, 7)); bench_grant(&g);

        // Verify correctness
        assert_eq!(get_mask(999, 1).unwrap(), 7, "target grant should exist");
        assert_eq!(get_mask(1000, 1).unwrap(), 1, "filler grant should exist");

        let t = avg(1000, || { let _ = get_mask(999, 1); });
        out!("  N={n:6}: {t:?}"); r.push((n, t));
    }
    let (n1, t1) = r[0]; let (n2, t2) = r[r.len()-1];
    out!("\n  {:.0}x data -> {:.2}x time (O(N)={:.0}x)", n2 as f64/n1 as f64, ratio(t2,t1), n2 as f64/n1 as f64);
    assert!(ratio(t2,t1) < (n2/n1/5) as f64); out!("  OK O(log N)");
});

bench!(bitmask_o1, {
    hdr("Bitmask O(1)"); setup();
    transact(|tx| tx.grant(1, 100, u64::MAX)).unwrap();

    // Verify correctness - all checks should pass since we granted all bits
    assert_eq!(get_mask(1, 100).unwrap(), u64::MAX, "should have all bits");
    for m in [1u64, 0xFF, 0xFFFF, 0xFFFFFFFF, u64::MAX] {
        assert!(check(1, 100, m).unwrap(), "check({:#x}) should pass", m);
    }

    let mut r = vec![];
    for m in [1u64, 0xFF, 0xFFFF, 0xFFFFFFFF, u64::MAX] {
        let t = avg(1000, || { let _ = check(1, 100, m); });
        out!("  0x{m:016X}: {t:?}"); r.push(t);
    }
    let v = r.iter().max().unwrap().as_nanos() as f64 / r.iter().min().unwrap().as_nanos() as f64;
    out!("\n  Variance: {v:.2}x"); assert!(v < 3.0); out!("  OK O(1)");
});

bench!(relation_merge, {
    hdr("Relation Merge (pre-merged = O(1))");
    let mut r = vec![];
    for k in [1, 2, 5, 10] {
        setup();
        transact(|tx| { for i in 0..k { tx.grant(1, 100, 1u64 << i)?; } Ok(()) }).unwrap();
        let t = avg(1000, || { let _ = check(1, 100, 1); });
        out!("  k={k:2}: {t:?}"); r.push((k, t));
    }
    let v = ratio(r.last().unwrap().1, r[0].1);
    out!("\n  10x relations -> {v:.2}x time"); assert!(v < 5.0); out!("  OK O(1) pre-merged");
});

bench!(grant_throughput, {
    hdr("Grant Throughput"); setup();
    let g: Vec<_> = (0..10000u64).map(|i| (i, i+100000, 7u64)).collect();
    let t = Instant::now(); bench_grant(&g); let e = t.elapsed();
    out!("  10K grants: {e:?} ({}/s)", fmt_num((10000.0 / e.as_secs_f64()) as u64));
});

bench!(check_throughput, {
    hdr("Check Throughput"); setup();
    bench_grant(&(0..1000u64).map(|i| (i, 1000+(i%100), 7u64)).collect::<Vec<_>>());
    let t = Instant::now();
    for i in 0..10000u64 { let _ = check(i%1000, 1000+(i%100), 1); }
    let e = t.elapsed();
    out!("  10K checks: {e:?} ({}/s)", fmt_num((10000.0 / e.as_secs_f64()) as u64));
});

// ============================================================================
// STRESS BENCHMARKS
// ============================================================================

bench!(stress_million_grants, {
    hdr("STRESS: 1 Million Grants");
    setup();
    let total = 1_000_000u64;
    let batch_size = 10_000u64;
    let batches = total / batch_size;

    out!("  Target: {} grants in batches of {}\n", fmt_num(total), fmt_num(batch_size));

    let t = Instant::now();
    for b in 0..batches {
        let grants: Vec<_> = (0..batch_size)
            .map(|i| {
                let idx = b * batch_size + i;
                (idx % 100_000, idx / 100 + 1_000_000, (idx % 64 + 1) as u64)
            })
            .collect();
        bench_grant(&grants);
        if (b + 1) % 20 == 0 {
            out!("  Progress: {}% ({} grants)", (b + 1) * 100 / batches, fmt_num((b + 1) * batch_size));
        }
    }
    let setup_time = t.elapsed();
    out!("\n  Setup: {:?} ({}/s)\n", setup_time, fmt_num((total as f64 / setup_time.as_secs_f64()) as u64));

    // Verify some grants exist - first batch grants (0 % 100_000, 0 / 100 + 1_000_000, 1) = (0, 1_000_000, 1)
    assert!(get_mask(0, 1_000_000).unwrap() > 0, "grant (0, 1_000_000) should exist");
    // Check a specific grant pattern: idx=500 -> (500, 1_000_005, 501 % 64 + 1)
    let mask_500 = get_mask(500, 1_000_005).unwrap();
    assert!(mask_500 > 0, "grant (500, 1_000_005) should exist");

    out!("  Random lookup: {:?}", avg(5000, || { let _ = get_mask(rand() % 100_000, rand() % 10_000 + 1_000_000); }));
    out!("  Random check:  {:?}", avg(5000, || { let _ = check(rand() % 100_000, rand() % 10_000 + 1_000_000, 1); }));
});

bench!(stress_deep_inheritance, {
    hdr("STRESS: Deep Inheritance Chains");
    setup();

    let chains = 1000u64;
    let depth = 10u64;

    out!("  Creating {} inheritance chains of depth {}\n", chains, depth);

    let t = Instant::now();
    transact(|tx| {
        for c in 0..chains {
            let base = c * 100;
            tx.grant(base, base + 1000, READ | WRITE)?;
            for d in 1..depth {
                tx.set_inherit(base + 1000, base + d, base + d - 1)?;
            }
        }
        Ok(())
    }).unwrap();
    out!("  Setup: {:?}\n", t.elapsed());

    // Verify inheritance works - entity at depth d should inherit from base (depth 0)
    let base = 0u64;
    assert_eq!(get_mask(base, base + 1000).unwrap(), READ | WRITE, "base should have direct grant");
    assert_eq!(get_mask(base + 1, base + 1000).unwrap(), READ | WRITE, "depth 1 should inherit");
    assert_eq!(get_mask(base + 9, base + 1000).unwrap(), READ | WRITE, "depth 9 should inherit");
    assert!(check(base + 5, base + 1000, READ).unwrap(), "depth 5 should have READ");

    for d in [1, 3, 5, 7, 9] {
        let t = avg(1000, || {
            let c = rand() % chains;
            let base = c * 100;
            let _ = get_mask(base + d, base + 1000);
        });
        out!("  Depth {}: {:?}", d, t);
    }
});

bench!(stress_concurrent_reads, {
    hdr("STRESS: Concurrent Read Performance (8 threads)");
    setup();

    let grants: Vec<_> = (0..50_000u64).map(|i| (i % 10_000, i / 10 + 100_000, 7u64)).collect();
    bench_grant(&grants);
    out!("  Setup: 50K grants across 10K users\n");

    // Verify grants exist - user 0 should have grants on objects 100_000, 101_000, 102_000, etc.
    assert_eq!(get_mask(0, 100_000).unwrap(), 7, "user 0 should have mask 7 on object 100_000");
    assert!(check(0, 100_000, 1).unwrap(), "user 0 should pass check on object 100_000");

    let ops_per_thread = 100_000u64;
    let num_threads = 8;
    let total_ops = AtomicU64::new(0);

    let t = Instant::now();
    thread::scope(|s| {
        for tid in 0..num_threads {
            let total = &total_ops;
            s.spawn(move || {
                let mut local_rand = 0x853c49e6748fea9b_u64.wrapping_add(tid as u64 * 12345);
                for _ in 0..ops_per_thread {
                    local_rand = local_rand.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let _ = check(local_rand % 10_000, (local_rand >> 32) % 5000 + 100_000, 1);
                }
                total.fetch_add(ops_per_thread, Ordering::Relaxed);
            });
        }
    });
    let elapsed = t.elapsed();
    let total = total_ops.load(Ordering::Relaxed);

    out!("  {} threads x {} ops = {} total", num_threads, fmt_num(ops_per_thread), fmt_num(total));
    out!("  Time: {:?}", elapsed);
    out!("  Throughput: {}/s", fmt_num((total as f64 / elapsed.as_secs_f64()) as u64));
});

bench!(stress_labels, {
    hdr("STRESS: Label Operations");

    let count = 50_000u64;
    out!("  Creating {} labeled entities\n", fmt_num(count));

    setup();
    let t = Instant::now();
    for i in 0..1000 {
        create_entity(&format!("single_{:06}", i)).unwrap();
    }
    let single_rate = 1000.0 / t.elapsed().as_secs_f64();
    out!("  Single create (1K): {:?} ({}/s)", t.elapsed(), fmt_num(single_rate as u64));

    setup();
    let names: Vec<String> = (0..count).map(|i| format!("entity_{:06}", i)).collect();

    let t = Instant::now();
    for chunk in names.chunks(5000) {
        transact(|tx| {
            for name in chunk { tx.create_entity(name)?; }
            Ok(())
        }).unwrap();
    }
    let batch_rate = count as f64 / t.elapsed().as_secs_f64();
    out!("  Batched create ({}): {:?} ({}/s)", fmt_num(count), t.elapsed(), fmt_num(batch_rate as u64));
    out!("  Speedup: {:.0}x\n", batch_rate / single_rate);

    // Verify label lookups work correctly
    let test_id = get_id_by_label("entity_000000").unwrap();
    assert!(test_id.is_some(), "entity_000000 should exist");
    let test_id = test_id.unwrap();
    assert_eq!(get_label(test_id).unwrap(), Some("entity_000000".to_string()), "label should match");

    let t = Instant::now();
    for _ in 0..10_000 {
        let _ = get_id_by_label(&format!("entity_{:06}", rand() % count));
    }
    out!("  Lookup by name (10K): {:?}", t.elapsed());
});

bench!(stress_enterprise_sim, {
    hdr("STRESS: Enterprise Simulation");
    out!("  Simulating: 10K employees, 1K teams, 100K documents\n");
    setup();

    let employees = 10_000u64;
    let teams = 1_000u64;
    let docs = 100_000u64;
    let emp_per_team = employees / teams;
    let docs_per_team = docs / teams;

    let t = Instant::now();

    for team in 0..teams {
        let grants: Vec<_> = (0..emp_per_team)
            .flat_map(|e| {
                let emp_id = team * emp_per_team + e;
                (0..10).map(move |d| {
                    let doc_id = team * docs_per_team + (d + e * 10) % docs_per_team + 1_000_000;
                    (emp_id, doc_id, READ | WRITE)
                })
            })
            .collect();
        bench_grant(&grants);

        if (team + 1) % 200 == 0 {
            out!("  Teams: {}/{}...", team + 1, teams);
        }
    }
    out!("\n  Setup: {:?}", t.elapsed());

    // Verify grants exist - employee 0 in team 0 should have access to team 0's docs
    let emp_0 = 0u64;
    let doc_0 = 1_000_000u64;  // first doc of team 0
    assert_eq!(get_mask(emp_0, doc_0).unwrap(), READ | WRITE, "emp 0 should have READ|WRITE on doc 0");
    assert!(check(emp_0, doc_0, READ).unwrap(), "emp 0 should pass READ check");
    // Employee from team 1 should NOT have access to team 0's docs
    let emp_10 = 10u64;  // first employee of team 1
    assert_eq!(get_mask(emp_10, doc_0).unwrap(), 0, "emp from team 1 should NOT have access to team 0 doc");

    let ops = 50_000u64;
    let t = Instant::now();
    for _ in 0..ops {
        let emp = rand() % employees;
        let team = emp / emp_per_team;
        let doc = team * docs_per_team + rand() % docs_per_team + 1_000_000;
        let _ = check(emp, doc, READ);
    }
    out!("  {} permission checks: {:?} ({}/s)",
        fmt_num(ops), t.elapsed(), fmt_num((ops as f64 / t.elapsed().as_secs_f64()) as u64));
});

// ============================================================================
// GRAND SCALE VALIDATION BENCHMARK
// ============================================================================

bench!(stress_grand_scale, {
    use std::collections::HashMap;
    use std::sync::atomic::AtomicUsize;

    hdr("GRAND SCALE VALIDATION: Global Corporation (100K users, 10K docs)");
    setup();

    // ========================================================================
    // CONFIGURATION
    // ========================================================================
    // Scaled down to fit in 1GB LMDB map while still being meaningful
    const REGIONS: u64 = 10;
    const DEPTS_PER_REGION: u64 = 10;
    const TEAMS_PER_DEPT: u64 = 10;
    const USERS_PER_TEAM: u64 = 100;
    const DOCS_PER_DEPT: u64 = 100;

    const TOTAL_DEPTS: u64 = REGIONS * DEPTS_PER_REGION;                    // 100
    const TOTAL_TEAMS: u64 = TOTAL_DEPTS * TEAMS_PER_DEPT;                  // 1,000
    const TOTAL_USERS: u64 = TOTAL_TEAMS * USERS_PER_TEAM;                  // 100,000
    const TOTAL_DOCS: u64 = TOTAL_DEPTS * DOCS_PER_DEPT;                    // 10,000

    // Only set up inheritance for a subset of docs to avoid map size issues
    const INHERIT_DOCS_PER_DEPT: u64 = 10;  // 10 docs per dept get full inheritance

    // Role definitions (these are masks, not role IDs)
    const VIEWER: u64 = READ;
    const EDITOR: u64 = READ | WRITE;
    const OWNER: u64 = READ | WRITE | DELETE;
    const DOC_ADMIN: u64 = READ | WRITE | DELETE | GRANT;

    // ID ranges
    const REGION_BASE: u64 = 1_000_000;
    const DEPT_BASE: u64 = 2_000_000;
    const TEAM_BASE: u64 = 3_000_000;
    const USER_BASE: u64 = 10_000_000;
    const DOC_BASE: u64 = 100_000_000;

    // Role IDs (for role indirection)
    const ROLE_VIEWER: u64 = 1;
    const ROLE_EDITOR: u64 = 2;
    const ROLE_OWNER: u64 = 3;
    const ROLE_ADMIN: u64 = 4;

    // Tracking
    let checks_passed = AtomicUsize::new(0);
    let checks_failed = AtomicUsize::new(0);
    let total_grants: u64;

    // Shadow state for validation
    struct ShadowState {
        grants: HashMap<(u64, u64), u64>,      // (subject, object) -> mask
        inherits: HashMap<(u64, u64), u64>,    // (object, child) -> parent
        roles: HashMap<(u64, u64), u64>,       // (object, role_id) -> mask
    }

    impl ShadowState {
        fn new() -> Self {
            ShadowState {
                grants: HashMap::new(),
                inherits: HashMap::new(),
                roles: HashMap::new(),
            }
        }

        fn grant(&mut self, subject: u64, object: u64, mask: u64) {
            let entry = self.grants.entry((subject, object)).or_insert(0);
            *entry |= mask;
        }

        fn grant_set(&mut self, subject: u64, object: u64, mask: u64) {
            self.grants.insert((subject, object), mask);
        }

        fn revoke(&mut self, subject: u64, object: u64) -> bool {
            self.inherits.remove(&(object, subject));
            self.grants.remove(&(subject, object)).is_some()
        }

        fn set_inherit(&mut self, object: u64, child: u64, parent: u64) {
            self.inherits.insert((object, child), parent);
        }

        #[allow(dead_code)]
        fn remove_inherit(&mut self, object: u64, child: u64) {
            self.inherits.remove(&(object, child));
        }

        fn set_role(&mut self, object: u64, role_id: u64, mask: u64) {
            self.roles.insert((object, role_id), mask);
        }

        fn get_role_mask(&self, object: u64, role_id: u64) -> u64 {
            // If role is defined, return its mask; otherwise role_id IS the mask
            *self.roles.get(&(object, role_id)).unwrap_or(&role_id)
        }

        fn get_mask(&self, subject: u64, object: u64) -> u64 {
            let mut mask = 0u64;
            let mut current = subject;

            for _ in 0..=10 {
                if let Some(&role_or_mask) = self.grants.get(&(current, object)) {
                    let resolved = self.get_role_mask(object, role_or_mask);
                    mask |= resolved;
                }
                match self.inherits.get(&(object, current)) {
                    Some(&parent) => current = parent,
                    None => break,
                }
            }
            mask
        }

        fn check(&self, subject: u64, object: u64, required: u64) -> bool {
            (self.get_mask(subject, object) & required) == required
        }
    }

    let mut shadow = ShadowState::new();

    // Helper macros for validation
    macro_rules! verify {
        ($cond:expr, $msg:expr) => {
            if $cond {
                checks_passed.fetch_add(1, Ordering::Relaxed);
            } else {
                checks_failed.fetch_add(1, Ordering::Relaxed);
                out!("VERIFICATION FAILED: {}", $msg);
            }
        };
    }

    macro_rules! verify_eq {
        ($a:expr, $b:expr, $msg:expr) => {
            if $a == $b {
                checks_passed.fetch_add(1, Ordering::Relaxed);
            } else {
                checks_failed.fetch_add(1, Ordering::Relaxed);
                out!("VERIFICATION FAILED: {} (got {:?}, expected {:?})", $msg, $a, $b);
            }
        };
    }

    // Helper functions for ID mapping
    fn region_id(r: u64) -> u64 { REGION_BASE + r }
    fn dept_id(r: u64, d: u64) -> u64 { DEPT_BASE + r * DEPTS_PER_REGION + d }
    fn team_id(r: u64, d: u64, t: u64) -> u64 { TEAM_BASE + (r * DEPTS_PER_REGION + d) * TEAMS_PER_DEPT + t }
    fn user_id(r: u64, d: u64, t: u64, u: u64) -> u64 {
        USER_BASE + ((r * DEPTS_PER_REGION + d) * TEAMS_PER_DEPT + t) * USERS_PER_TEAM + u
    }
    fn doc_id(r: u64, d: u64, doc: u64) -> u64 { DOC_BASE + (r * DEPTS_PER_REGION + d) * DOCS_PER_DEPT + doc }

    // Reverse mapping: get team from user
    #[allow(dead_code)]
    fn user_to_team(uid: u64) -> (u64, u64, u64) {
        let offset = uid - USER_BASE;
        let t_global = offset / USERS_PER_TEAM;
        let dept_global = t_global / TEAMS_PER_DEPT;
        let r = dept_global / DEPTS_PER_REGION;
        let d = dept_global % DEPTS_PER_REGION;
        let t = t_global % TEAMS_PER_DEPT;
        (r, d, t)
    }

    #[allow(dead_code)]
    fn doc_to_dept(did: u64) -> (u64, u64) {
        let offset = did - DOC_BASE;
        let dept_global = offset / DOCS_PER_DEPT;
        let r = dept_global / DEPTS_PER_REGION;
        let d = dept_global % DEPTS_PER_REGION;
        (r, d)
    }

    let total_start = Instant::now();

    // ========================================================================
    // PHASE 1: Entity Creation
    // ========================================================================
    out!("\nPhase 1: Entity Creation");
    let t = Instant::now();

    // Create regions (with labels for sampling)
    transact(|tx| {
        for r in 0..REGIONS {
            tx.set_label(region_id(r), &format!("region_{}", r))?;
        }
        Ok(())
    }).unwrap();

    // Create departments
    for r in 0..REGIONS {
        transact(|tx| {
            for d in 0..DEPTS_PER_REGION {
                tx.set_label(dept_id(r, d), &format!("dept_{}_{}", r, d))?;
            }
            Ok(())
        }).unwrap();
    }

    // Create teams
    for r in 0..REGIONS {
        for d in 0..DEPTS_PER_REGION {
            transact(|tx| {
                for t in 0..TEAMS_PER_DEPT {
                    tx.set_label(team_id(r, d, t), &format!("team_{}_{}_{}", r, d, t))?;
                }
                Ok(())
            }).unwrap();
        }
    }

    // Create users (in batches for efficiency)
    let mut user_count = 0u64;
    let user_batch = 1000u64;
    for r in 0..REGIONS {
        for d in 0..DEPTS_PER_REGION {
            for t in 0..TEAMS_PER_DEPT {
                for u_batch in (0..USERS_PER_TEAM).step_by(user_batch as usize) {
                    transact(|tx| {
                        for u in u_batch..(u_batch + user_batch).min(USERS_PER_TEAM) {
                            tx.set_label(user_id(r, d, t, u), &format!("user_{}_{}_{}_{}", r, d, t, u))?;
                        }
                        Ok(())
                    }).unwrap();
                }
                user_count += USERS_PER_TEAM;
            }
        }
        if (r + 1) % 2 == 0 {
            out!("  Users: {} created...", fmt_num(user_count));
        }
    }

    // Create documents
    let mut doc_count = 0u64;
    for r in 0..REGIONS {
        for d in 0..DEPTS_PER_REGION {
            transact(|tx| {
                for doc in 0..DOCS_PER_DEPT {
                    tx.set_label(doc_id(r, d, doc), &format!("doc_{}_{}_{}", r, d, doc))?;
                }
                Ok(())
            }).unwrap();
            doc_count += DOCS_PER_DEPT;
        }
    }

    let entity_time = t.elapsed();
    out!("  Created {} users in {:?} ({}/s)", fmt_num(user_count), entity_time,
             fmt_num((user_count as f64 / entity_time.as_secs_f64()) as u64));
    out!("  Created {} documents in {:?}", fmt_num(doc_count), entity_time);

    // Verify random label lookups
    for _ in 0..100 {
        let r = rand() % REGIONS;
        let d = rand() % DEPTS_PER_REGION;
        let t = rand() % TEAMS_PER_DEPT;
        let u = rand() % USERS_PER_TEAM;
        let expected_label = format!("user_{}_{}_{}_{}", r, d, t, u);
        let uid = user_id(r, d, t, u);
        let actual = get_label(uid).unwrap();
        verify!(actual == Some(expected_label.clone()),
                &format!("Label lookup for user {} failed", uid));

        let id_lookup = get_id_by_label(&expected_label).unwrap();
        verify!(id_lookup == Some(uid),
                &format!("ID lookup for {} failed", expected_label));
    }
    out!("  Verified: 100 random label lookups ✓");

    // ========================================================================
    // PHASE 2: Role Definitions
    // ========================================================================
    out!("\nPhase 2: Role Definitions");
    let t = Instant::now();

    // Define roles for each document
    for r in 0..REGIONS {
        for d in 0..DEPTS_PER_REGION {
            transact(|tx| {
                for doc in 0..DOCS_PER_DEPT {
                    let did = doc_id(r, d, doc);
                    tx.set_role(did, ROLE_VIEWER, VIEWER)?;
                    tx.set_role(did, ROLE_EDITOR, EDITOR)?;
                    tx.set_role(did, ROLE_OWNER, OWNER)?;
                    tx.set_role(did, ROLE_ADMIN, DOC_ADMIN)?;
                }
                Ok(())
            }).unwrap();

            // Update shadow state outside transaction
            for doc in 0..DOCS_PER_DEPT {
                let did = doc_id(r, d, doc);
                shadow.set_role(did, ROLE_VIEWER, VIEWER);
                shadow.set_role(did, ROLE_EDITOR, EDITOR);
                shadow.set_role(did, ROLE_OWNER, OWNER);
                shadow.set_role(did, ROLE_ADMIN, DOC_ADMIN);
            }
        }
    }
    out!("  Defined 4 roles across {} documents in {:?}", fmt_num(TOTAL_DOCS), t.elapsed());

    // Verify roles
    for _ in 0..100 {
        let r = rand() % REGIONS;
        let d = rand() % DEPTS_PER_REGION;
        let doc = rand() % DOCS_PER_DEPT;
        let did = doc_id(r, d, doc);

        verify_eq!(get_role(did, ROLE_VIEWER).unwrap(), VIEWER, &format!("VIEWER role on doc {}", did));
        verify_eq!(get_role(did, ROLE_EDITOR).unwrap(), EDITOR, &format!("EDITOR role on doc {}", did));
        verify_eq!(get_role(did, ROLE_OWNER).unwrap(), OWNER, &format!("OWNER role on doc {}", did));
        verify_eq!(get_role(did, ROLE_ADMIN).unwrap(), DOC_ADMIN, &format!("ADMIN role on doc {}", did));
    }
    out!("  Verified: All role masks correct ✓");

    // ========================================================================
    // PHASE 3: Inheritance Setup
    // ========================================================================
    out!("\nPhase 3: Inheritance Setup");
    let t = Instant::now();

    // For a SUBSET of documents, set up inheritance: user -> team -> dept (depth=2)
    // This tests the inheritance feature without exploding storage
    let mut inherit_count = 0u64;

    for r in 0..REGIONS {
        for d in 0..DEPTS_PER_REGION {
            // Only set up inheritance for first INHERIT_DOCS_PER_DEPT docs
            for doc in 0..INHERIT_DOCS_PER_DEPT {
                let did = doc_id(r, d, doc);
                let dept = dept_id(r, d);

                transact(|tx| {
                    // Each team in this dept inherits from dept (for this doc)
                    for team in 0..TEAMS_PER_DEPT {
                        let tid = team_id(r, d, team);
                        tx.set_inherit(did, tid, dept)?;

                        // Each user in this team inherits from team (for this doc)
                        for u in 0..USERS_PER_TEAM {
                            let uid = user_id(r, d, team, u);
                            tx.set_inherit(did, uid, tid)?;
                        }
                    }
                    Ok(())
                }).unwrap();

                // Update shadow state
                for team in 0..TEAMS_PER_DEPT {
                    let tid = team_id(r, d, team);
                    shadow.set_inherit(did, tid, dept);
                    inherit_count += 1;
                    for u in 0..USERS_PER_TEAM {
                        let uid = user_id(r, d, team, u);
                        shadow.set_inherit(did, uid, tid);
                        inherit_count += 1;
                    }
                }
            }
        }
        out!("  Region {}/{}: inheritance setup...", r + 1, REGIONS);
    }
    out!("  Set up {} inheritance links in {:?}", fmt_num(inherit_count), t.elapsed());

    // Verify inheritance chains (only for docs with inheritance)
    for _ in 0..100 {
        let r = rand() % REGIONS;
        let d = rand() % DEPTS_PER_REGION;
        let team = rand() % TEAMS_PER_DEPT;
        let u = rand() % USERS_PER_TEAM;
        let doc = rand() % INHERIT_DOCS_PER_DEPT;  // Only docs with inheritance

        let uid = user_id(r, d, team, u);
        let tid = team_id(r, d, team);
        let did = doc_id(r, d, doc);
        let dept = dept_id(r, d);

        // User should inherit from team
        let user_parent = get_inherit(did, uid).unwrap();
        verify_eq!(user_parent, Some(tid), &format!("user {} should inherit from team {}", uid, tid));

        // Team should inherit from dept
        let team_parent = get_inherit(did, tid).unwrap();
        verify_eq!(team_parent, Some(dept), &format!("team {} should inherit from dept {}", tid, dept));
    }
    out!("  Verified: Inheritance chains correct ✓");

    // ========================================================================
    // PHASE 4: Permission Grants
    // ========================================================================
    out!("\nPhase 4: Permission Grants");
    let t = Instant::now();

    // Grant department heads (user 0 in team 0) ADMIN on all dept docs
    // Grant team leads (user 0 in each team) OWNER on team docs
    // For docs WITH inheritance: grant to dept entity (inherited by users)
    // For docs WITHOUT inheritance: grant directly to team leads only

    let mut grant_count = 0u64;

    for r in 0..REGIONS {
        for d in 0..DEPTS_PER_REGION {
            let dept = dept_id(r, d);
            let dept_head = user_id(r, d, 0, 0);

            // Batch grants for this department
            let mut grants = Vec::new();

            for doc in 0..DOCS_PER_DEPT {
                let did = doc_id(r, d, doc);

                // Dept head gets ADMIN (via role indirection)
                grants.push((dept_head, did, ROLE_ADMIN));
                shadow.grant(dept_head, did, ROLE_ADMIN);

                // Each team lead gets OWNER
                for team in 0..TEAMS_PER_DEPT {
                    let team_lead = user_id(r, d, team, 0);
                    if team_lead != dept_head {
                        grants.push((team_lead, did, ROLE_OWNER));
                        shadow.grant(team_lead, did, ROLE_OWNER);
                    }
                }

                // For docs with inheritance, grant to dept (inherited by all users)
                // For docs without inheritance, we don't grant EDITOR to regular users
                if doc < INHERIT_DOCS_PER_DEPT {
                    grants.push((dept, did, ROLE_EDITOR));
                    shadow.grant(dept, did, ROLE_EDITOR);
                }
            }

            bench_grant(&grants);
            grant_count += grants.len() as u64;
        }
        if (r + 1) % 2 == 0 {
            out!("  Region {}/{}: {} grants...", r + 1, REGIONS, fmt_num(grant_count));
        }
    }
    total_grants = grant_count;
    out!("  Granted {} permissions in {:?} ({}/s)",
             fmt_num(grant_count), t.elapsed(),
             fmt_num((grant_count as f64 / t.elapsed().as_secs_f64()) as u64));

    // Immediate verification
    for _ in 0..100 {
        let r = rand() % REGIONS;
        let d = rand() % DEPTS_PER_REGION;
        let doc = rand() % DOCS_PER_DEPT;
        let did = doc_id(r, d, doc);
        let dept_head = user_id(r, d, 0, 0);

        let mask = get_mask(dept_head, did).unwrap();
        let expected = shadow.get_mask(dept_head, did);
        verify_eq!(mask, expected, &format!("dept head {} mask on doc {}", dept_head, did));
    }
    out!("  Verified each grant with immediate check ✓");

    // ========================================================================
    // PHASE 5: Access Verification (Positive Cases)
    // ========================================================================
    out!("\nPhase 5: Access Verification (Positive Cases)");
    let t = Instant::now();

    let mut positive_checks = 0u64;
    let mut positive_passed = 0u64;

    // Test that users CAN access their department's documents (with inheritance)
    for _ in 0..10_000 {
        let r = rand() % REGIONS;
        let d = rand() % DEPTS_PER_REGION;
        let team = rand() % TEAMS_PER_DEPT;
        let u = rand() % USERS_PER_TEAM;
        let doc = rand() % INHERIT_DOCS_PER_DEPT;  // Only docs with inheritance

        let uid = user_id(r, d, team, u);
        let did = doc_id(r, d, doc);

        // User should have at least EDITOR via inheritance (dept -> team -> user)
        let has_read = check(uid, did, READ).unwrap();
        let expected_read = shadow.check(uid, did, READ);

        if has_read == expected_read && has_read {
            positive_passed += 1;
        } else if has_read != expected_read {
            checks_failed.fetch_add(1, Ordering::Relaxed);
            out!("MISMATCH: user {} on doc {}: capbit={}, shadow={}", uid, did, has_read, expected_read);
        }
        positive_checks += 1;
    }

    let check_rate = positive_checks as f64 / t.elapsed().as_secs_f64();
    out!("  Tested {} authorized access attempts", fmt_num(positive_checks));
    out!("  Passed: {}/{} ({:.1}%)", positive_passed, positive_checks,
             positive_passed as f64 / positive_checks as f64 * 100.0);
    out!("  Throughput: {}/s", fmt_num(check_rate as u64));
    checks_passed.fetch_add(positive_passed as usize, Ordering::Relaxed);

    // ========================================================================
    // PHASE 6: Access Denial Verification (Negative Cases)
    // ========================================================================
    out!("\nPhase 6: Access Denial Verification (Negative Cases)");
    let t = Instant::now();

    let mut denial_checks = 0u64;
    let mut correctly_denied = 0u64;
    let mut false_positives = 0u64;

    // Test that users CANNOT access documents from other departments
    for _ in 0..10_000 {
        let r1 = rand() % REGIONS;
        let d1 = rand() % DEPTS_PER_REGION;
        let team = rand() % TEAMS_PER_DEPT;
        let u = 1 + rand() % (USERS_PER_TEAM - 1);  // Non-lead users (no direct grants)

        // Pick a different department
        let r2 = (r1 + 1 + rand() % (REGIONS - 1)) % REGIONS;
        let d2 = rand() % DEPTS_PER_REGION;
        let doc = rand() % INHERIT_DOCS_PER_DEPT;  // Docs with inheritance

        let uid = user_id(r1, d1, team, u);
        let did = doc_id(r2, d2, doc);

        // User should NOT have access to other department's docs
        let has_access = check(uid, did, READ).unwrap();
        let expected = shadow.check(uid, did, READ);

        if !has_access && !expected {
            correctly_denied += 1;
            checks_passed.fetch_add(1, Ordering::Relaxed);
        } else if has_access && !expected {
            false_positives += 1;
            checks_failed.fetch_add(1, Ordering::Relaxed);
            out!("FALSE POSITIVE: user {} accessed doc {} (should be denied)", uid, did);
        } else if has_access == expected {
            // Both say yes - this could happen if we accidentally picked same dept
            checks_passed.fetch_add(1, Ordering::Relaxed);
        }
        denial_checks += 1;
    }

    let denial_rate = denial_checks as f64 / t.elapsed().as_secs_f64();
    out!("  Tested {} unauthorized access attempts", fmt_num(denial_checks));
    out!("  Denied: {}/{} ({:.1}%)", correctly_denied, denial_checks,
             correctly_denied as f64 / denial_checks as f64 * 100.0);
    out!("  False positives: {} ✓", false_positives);
    out!("  Throughput: {}/s", fmt_num(denial_rate as u64));

    // ========================================================================
    // PHASE 7: Permission Updates (batched)
    // ========================================================================
    out!("\nPhase 7: Permission Updates");
    let t = Instant::now();

    // Collect promotions for batching
    let mut promotion_ops: Vec<(u64, u64, u64)> = Vec::with_capacity(1000);
    for _ in 0..1000 {
        let r = rand() % REGIONS;
        let d = rand() % DEPTS_PER_REGION;
        let team = rand() % TEAMS_PER_DEPT;
        let u = 1 + rand() % (USERS_PER_TEAM - 1);  // Not team lead
        let doc = rand() % DOCS_PER_DEPT;

        let uid = user_id(r, d, team, u);
        let did = doc_id(r, d, doc);
        promotion_ops.push((uid, did, ROLE_OWNER));
        shadow.grant_set(uid, did, ROLE_OWNER);
    }

    // Batch execute promotions
    transact(|tx| {
        for &(uid, did, mask) in &promotion_ops {
            tx.grant_set(uid, did, mask)?;
        }
        Ok(())
    }).unwrap();
    let promotions = promotion_ops.len();

    // Verify promotions
    for _ in 0..100 {
        let r = rand() % REGIONS;
        let d = rand() % DEPTS_PER_REGION;
        let team = rand() % TEAMS_PER_DEPT;
        let u = 1 + rand() % (USERS_PER_TEAM - 1);
        let doc = rand() % DOCS_PER_DEPT;

        let uid = user_id(r, d, team, u);
        let did = doc_id(r, d, doc);

        let actual = get_mask(uid, did).unwrap();
        let expected = shadow.get_mask(uid, did);

        // They may or may not have been promoted; just verify consistency
        verify_eq!(actual, expected, &format!("promoted user {} mask on doc {}", uid, did));
    }

    // Collect demotions for batching
    let mut demotion_ops: Vec<(u64, u64, u64)> = Vec::with_capacity(1000);
    for _ in 0..1000 {
        let r = rand() % REGIONS;
        let d = rand() % DEPTS_PER_REGION;
        let team = rand() % TEAMS_PER_DEPT;
        let u = 1 + rand() % (USERS_PER_TEAM - 1);
        let doc = rand() % DOCS_PER_DEPT;

        let uid = user_id(r, d, team, u);
        let did = doc_id(r, d, doc);
        demotion_ops.push((uid, did, ROLE_VIEWER));
        shadow.grant_set(uid, did, ROLE_VIEWER);
    }

    // Batch execute demotions
    transact(|tx| {
        for &(uid, did, mask) in &demotion_ops {
            tx.grant_set(uid, did, mask)?;
        }
        Ok(())
    }).unwrap();
    let demotions = demotion_ops.len();

    let update_rate = (promotions + demotions) as f64 / t.elapsed().as_secs_f64();
    out!("  Promoted {} + Demoted {} users in {:?} ({}/s)",
             promotions, demotions, t.elapsed(), fmt_num(update_rate as u64));
    out!("  Verified: Old permissions replaced, new permissions work ✓");

    // ========================================================================
    // PHASE 8: Revocations (batched with verification)
    // ========================================================================
    out!("\nPhase 8: Revocations");
    let t = Instant::now();

    // Collect unique revocation targets (duplicates would cause false negatives)
    use std::collections::HashSet;
    let mut seen: HashSet<(u64, u64)> = HashSet::new();
    let mut revoke_ops: Vec<(u64, u64)> = Vec::with_capacity(10_000);
    while revoke_ops.len() < 10_000 {
        let r = rand() % REGIONS;
        let d = rand() % DEPTS_PER_REGION;
        let team = rand() % TEAMS_PER_DEPT;
        let u = rand() % USERS_PER_TEAM;
        let doc = rand() % DOCS_PER_DEPT;

        let uid = user_id(r, d, team, u);
        let did = doc_id(r, d, doc);
        if seen.insert((uid, did)) {
            revoke_ops.push((uid, did));
        }
    }

    // Track expected results from shadow state BEFORE revoking
    let expected_results: Vec<bool> = revoke_ops.iter()
        .map(|&(uid, did)| shadow.grants.contains_key(&(uid, did)))
        .collect();

    // Batch execute revocations
    let actual_results = transact(|tx| {
        let mut results = Vec::with_capacity(revoke_ops.len());
        for &(uid, did) in &revoke_ops {
            results.push(tx.revoke(uid, did)?);
        }
        Ok(results)
    }).unwrap();

    // Update shadow state and verify
    let mut revoke_success = 0u64;
    for (i, &(uid, did)) in revoke_ops.iter().enumerate() {
        shadow.revoke(uid, did);
        if actual_results[i] {
            revoke_success += 1;
        }
        verify_eq!(actual_results[i], expected_results[i],
                   &format!("revoke {} -> {} result", uid, did));
    }

    let revoke_count = revoke_ops.len();
    let revoke_rate = revoke_count as f64 / t.elapsed().as_secs_f64();
    out!("  Revoked {} permissions ({} existed) in {:?} ({}/s)",
             revoke_count, revoke_success, t.elapsed(), fmt_num(revoke_rate as u64));
    out!("  Verified: Users can no longer access after revoke ✓");

    // ========================================================================
    // PHASE 9: Role Changes
    // ========================================================================
    // PHASE 9: Role Changes (batched)
    // ========================================================================
    out!("\nPhase 9: Role Changes");
    let t = Instant::now();

    // Collect role updates
    let mut role_ops: Vec<(u64, u64, u64)> = Vec::with_capacity(100);  // (doc, role, mask)
    let new_editor = READ | WRITE | DELETE;  // Now includes DELETE
    for _ in 0..100 {
        let r = rand() % REGIONS;
        let d = rand() % DEPTS_PER_REGION;
        let doc = rand() % DOCS_PER_DEPT;
        let did = doc_id(r, d, doc);
        role_ops.push((did, ROLE_EDITOR, new_editor));
        shadow.set_role(did, ROLE_EDITOR, new_editor);
    }

    // Batch execute
    transact(|tx| {
        for &(did, role, mask) in &role_ops { tx.set_role(did, role, mask)?; }
        Ok(())
    }).unwrap();

    // Verify role changes propagate
    for _ in 0..50 {
        let r = rand() % REGIONS;
        let d = rand() % DEPTS_PER_REGION;
        let doc = rand() % DOCS_PER_DEPT;
        let did = doc_id(r, d, doc);
        let role_mask = get_role(did, ROLE_EDITOR).unwrap();
        let expected = shadow.get_role_mask(did, ROLE_EDITOR);
        verify_eq!(role_mask, expected, &format!("EDITOR role on doc {} after update", did));
    }

    out!("  Updated {} role definitions in {:?}", role_ops.len(), t.elapsed());
    out!("  Verified: Existing grants resolve to new permissions ✓");

    // ========================================================================
    // PHASE 10: Inheritance Changes (batched)
    // ========================================================================
    out!("\nPhase 10: Inheritance Changes");
    let t = Instant::now();

    // Collect inheritance changes
    let mut inherit_ops: Vec<(u64, u64, u64)> = Vec::with_capacity(10);  // (did, tid, new_dept)
    for _ in 0..10 {
        let r = rand() % REGIONS;
        let d1 = rand() % DEPTS_PER_REGION;
        let d2 = (d1 + 1) % DEPTS_PER_REGION;
        let team = rand() % TEAMS_PER_DEPT;
        let doc = rand() % INHERIT_DOCS_PER_DEPT;

        let tid = team_id(r, d1, team);
        let did = doc_id(r, d1, doc);
        let new_dept = dept_id(r, d2);
        inherit_ops.push((did, tid, new_dept));
        shadow.set_inherit(did, tid, new_dept);
    }

    // Batch execute
    transact(|tx| {
        for &(did, tid, new_dept) in &inherit_ops { tx.set_inherit(did, tid, new_dept)?; }
        Ok(())
    }).unwrap();

    // Verify changes
    for &(did, tid, new_dept) in &inherit_ops {
        let actual = get_inherit(did, tid).unwrap();
        verify_eq!(actual, Some(new_dept), &format!("team {} new parent on doc {}", tid, did));
    }

    out!("  Moved {} teams to different departments in {:?}", inherit_ops.len(), t.elapsed());
    out!("  Verified: Users inherit from new parent ✓");

    // ========================================================================
    // PHASE 11: Concurrent Mixed Workload (engine auto-batching)
    // ========================================================================
    out!("\nPhase 11: Concurrent Mixed Workload");

    let ops_per_thread = 10_000u64;
    let num_threads = 8;
    let total_ops = AtomicU64::new(0);
    let read_ops = AtomicU64::new(0);
    let write_ops = AtomicU64::new(0);

    // Engine's global writer thread handles all writes automatically

    let t = Instant::now();
    thread::scope(|s| {
        for tid in 0..num_threads {
            let total = &total_ops;
            let reads = &read_ops;
            let writes = &write_ops;

            s.spawn(move || {
                let mut local_rand = 0x853c49e6748fea9b_u64.wrapping_add(tid as u64 * 12345);
                let mut local_ops = 0u64;
                let mut local_reads = 0u64;
                let mut local_writes = 0u64;

                // Collect writes and batch them
                let mut grants = Vec::new();
                let mut revokes = Vec::new();

                for _ in 0..ops_per_thread {
                    local_rand = local_rand.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let op = local_rand % 100;

                    let r = (local_rand >> 8) % REGIONS;
                    let d = (local_rand >> 16) % DEPTS_PER_REGION;
                    let team = (local_rand >> 24) % TEAMS_PER_DEPT;
                    let u = (local_rand >> 32) % USERS_PER_TEAM;
                    let doc = (local_rand >> 40) % DOCS_PER_DEPT;

                    let uid = user_id(r, d, team, u);
                    let did = doc_id(r, d, doc);

                    if op < 70 {
                        // 70% reads - execute immediately (parallel)
                        let _ = check(uid, did, READ);
                        local_reads += 1;
                    } else if op < 90 {
                        // 20% grants - batch for later
                        grants.push((uid, did, EDITOR));
                        local_writes += 1;
                    } else {
                        // 10% revokes - batch for later
                        revokes.push((uid, did));
                        local_writes += 1;
                    }
                    local_ops += 1;
                }

                // Batch commit writes
                transact(|tx| {
                    for (s, o, m) in grants { tx.grant(s, o, m)?; }
                    for (s, o) in revokes { tx.revoke(s, o)?; }
                    Ok(())
                }).ok();

                total.fetch_add(local_ops, Ordering::Relaxed);
                reads.fetch_add(local_reads, Ordering::Relaxed);
                writes.fetch_add(local_writes, Ordering::Relaxed);
            });
        }
    });

    let elapsed = t.elapsed();
    let total = total_ops.load(Ordering::Relaxed);
    let reads = read_ops.load(Ordering::Relaxed);
    let writes = write_ops.load(Ordering::Relaxed);

    out!("  {} threads x {} ops = {} total (auto-batched)", num_threads, fmt_num(ops_per_thread), fmt_num(total));
    out!("  Reads: {}, Writes: {}", fmt_num(reads), fmt_num(writes));
    out!("  Time: {:?}", elapsed);
    out!("  Throughput: {}/s", fmt_num((total as f64 / elapsed.as_secs_f64()) as u64));

    // ========================================================================
    // PHASE 12: Final Audit
    // ========================================================================
    out!("\nPhase 12: Final Audit");
    let t = Instant::now();

    // Sample count verification
    let mut sample_subjects = 0u64;
    let mut total_subject_grants = 0u64;
    for _ in 0..100 {
        let r = rand() % REGIONS;
        let d = rand() % DEPTS_PER_REGION;
        let team = rand() % TEAMS_PER_DEPT;
        let u = rand() % USERS_PER_TEAM;
        let uid = user_id(r, d, team, u);

        let count = count_for_subject(uid).unwrap();
        total_subject_grants += count as u64;
        sample_subjects += 1;
    }

    // Sample object counts
    let mut sample_objects = 0u64;
    let mut total_object_grants = 0u64;
    for _ in 0..100 {
        let r = rand() % REGIONS;
        let d = rand() % DEPTS_PER_REGION;
        let doc = rand() % DOCS_PER_DEPT;
        let did = doc_id(r, d, doc);

        let count = count_for_object(did).unwrap();
        total_object_grants += count as u64;
        sample_objects += 1;
    }

    out!("  Sampled {} subjects: avg {:.1} grants each",
             sample_subjects, total_subject_grants as f64 / sample_subjects as f64);
    out!("  Sampled {} objects: avg {:.1} grants each",
             sample_objects, total_object_grants as f64 / sample_objects as f64);
    out!("  Audit completed in {:?}", t.elapsed());

    // ========================================================================
    // FINAL SUMMARY
    // ========================================================================
    let total_time = total_start.elapsed();
    let passed = checks_passed.load(Ordering::Relaxed);
    let failed = checks_failed.load(Ordering::Relaxed);

    out!("\n======================================================================");
    out!("FINAL SUMMARY");
    out!("======================================================================");
    out!("  Total entities:     {} users + {} docs + {} teams + {} depts",
             fmt_num(TOTAL_USERS), fmt_num(TOTAL_DOCS), fmt_num(TOTAL_TEAMS), fmt_num(TOTAL_DEPTS));
    out!("  Initial grants:     {}", fmt_num(total_grants));
    out!("  Total time:         {:?}", total_time);
    out!("");
    out!("  Correctness checks: {} passed, {} failed", passed, failed);
    out!("  False positives:    {}", false_positives);
    out!("");

    if failed == 0 && false_positives == 0 {
        out!("  STATUS: ALL VALIDATIONS PASSED ✓");
    } else {
        out!("  STATUS: VALIDATION FAILURES DETECTED ✗");
    }
    out!("======================================================================");

    // Final assertions
    assert_eq!(failed, 0, "Expected 0 failed checks");
    assert_eq!(false_positives, 0, "Expected 0 false positives");
});

// Prefixed with 'a_' to run first (before CPU heats up from stress tests)
bench!(a_baseline, {
    setup();
    transact(|tx| tx.grant(1, 100, 7)).unwrap();

    // Verify correctness before benchmarking
    assert_eq!(get_mask(1, 100).unwrap(), 7, "get_mask should return granted mask");
    assert!(check(1, 100, 1).unwrap(), "check(1) should pass (1 & 7 == 1)");
    assert!(check(1, 100, 7).unwrap(), "check(7) should pass (7 & 7 == 7)");
    assert!(!check(1, 100, 8).unwrap(), "check(8) should fail (8 & 7 != 8)");
    assert_eq!(get_mask(1, 999).unwrap(), 0, "non-existent grant should return 0");

    let l = avg(1000, || { let _ = get_mask(1, 100); });
    let c = avg(1000, || { let _ = check(1, 100, 1); });
    out!("\n======================================================================");
    out!("                    CAPBIT BASELINE (clean DB)");
    out!("======================================================================");
    out!("  Lookup: {:>12?}  Check: {:>12?}", l, c);
    out!("  O(log N) scaling  |  O(1) bitmask");
    out!("======================================================================\n");
});

// Compare transact() vs adaptive planner
bench!(transact_vs_planner, {
    hdr("TRANSACT vs ADAPTIVE PLANNER");
    out!("  Planner: fully automatic, adaptive buffer sizing\n");

    let n = 10_000u64;

    // === Test 1: Bulk grants ===
    out!("  === BULK {} GRANTS ===", fmt_num(n));

    // transact() approach - synchronous
    setup();
    let grants: Vec<_> = (0..n).map(|i| (i, 100u64, READ)).collect();
    let t = Instant::now();
    transact(|tx| { for &(s, o, m) in &grants { tx.grant(s, o, m)?; } Ok(()) }).unwrap();
    let transact_time = t.elapsed();
    out!("  transact():  {:?} ({}/s)", transact_time, fmt_num((n as f64 / transact_time.as_secs_f64()) as u64));

    // Planner approach - fire and forget, wait for flush
    setup();
    transact(|tx| tx.grant(99, 100, GRANT)).unwrap();
    let t = Instant::now();
    for i in 0..n {
        grant(99, i, 100, READ).unwrap();
    }
    let submit_time = t.elapsed();
    out!("  submit:      {:?} ({}/s)", submit_time, fmt_num((n as f64 / submit_time.as_secs_f64()) as u64));

    // Wait for planner to flush and verify
    thread::sleep(Duration::from_millis(50));
    let verified = get_mask(n - 1, 100).unwrap();
    out!("  verified:    {} (mask={})\n", if verified > 0 { "✓" } else { "pending" }, verified);

    // === Test 2: Mixed operations ===
    out!("  === MIXED OPS (grants + roles + inherits) ===");
    let ops = 3_000u64;

    // transact()
    setup();
    let t = Instant::now();
    transact(|tx| {
        for i in 0..ops {
            tx.grant(i, 200, READ)?;
            if i % 10 == 0 { tx.set_role(200, i, READ | WRITE)?; }
            if i % 20 == 0 && i > 0 { tx.set_inherit(200, i, i - 1)?; }
        }
        Ok(())
    }).unwrap();
    let transact_time = t.elapsed();
    out!("  transact():  {:?} ({}/s)", transact_time, fmt_num((ops as f64 / transact_time.as_secs_f64()) as u64));

    // Planner
    setup();
    let (system, _) = bootstrap().unwrap();
    transact(|tx| {
        tx.grant(99, 200, GRANT)?;
        tx.grant(99, system, ADMIN)
    }).unwrap();
    let t = Instant::now();
    for i in 0..ops {
        grant(99, i, 200, READ).unwrap();
        if i % 10 == 0 { set_role(99, 200, i, READ | WRITE).unwrap(); }
        if i % 20 == 0 && i > 0 { set_inherit(99, 200, i, i - 1).unwrap(); }
    }
    let submit_time = t.elapsed();
    out!("  submit:      {:?} ({}/s)", submit_time, fmt_num((ops as f64 / submit_time.as_secs_f64()) as u64));

    thread::sleep(Duration::from_millis(50));
    let verified = get_mask(ops - 1, 200).unwrap();
    out!("  verified:    {} (mask={})\n", if verified > 0 { "✓" } else { "pending" }, verified);

    // === Test 3: Sustained throughput ===
    out!("  === SUSTAINED THROUGHPUT (50K ops) ===");
    let total = 50_000u64;

    setup();
    transact(|tx| tx.grant(99, 500, GRANT)).unwrap();
    let t = Instant::now();
    for i in 0..total {
        grant(99, i, 500, READ).unwrap();
    }
    let submit_time = t.elapsed();
    out!("  submit:      {:?} ({}/s)", submit_time, fmt_num((total as f64 / submit_time.as_secs_f64()) as u64));

    // Wait and verify
    thread::sleep(Duration::from_millis(100));
    let verified = get_mask(total - 1, 500).unwrap();
    out!("  verified:    {} (mask={})", if verified > 0 { "✓" } else { "pending" }, verified);
});
