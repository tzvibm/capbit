//! Capbit Benchmarks - Stress Edition
//! Uses transact() for write operations (bypasses protection for benchmarking)

use capbit::*;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::sync::Once;

static INIT: Once = Once::new();

fn bench_db_path() -> String {
    std::env::var("CAPBIT_BENCH_DB").unwrap_or_else(|_| {
        let tmp = std::env::temp_dir();
        tmp.join("capbit_bench.mdb").to_string_lossy().to_string()
    })
}

fn setup() {
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

fn hdr(s: &str) { println!("\n{}\n{s}\n{}\n", "=".repeat(70), "=".repeat(70)); }
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
        let t = avg(1000, || { let _ = get_mask(999, 1); });
        println!("  N={n:6}: {t:?}"); r.push((n, t));
    }
    let (n1, t1) = r[0]; let (n2, t2) = r[r.len()-1];
    println!("\n  {:.0}x data -> {:.2}x time (O(N)={:.0}x)", n2 as f64/n1 as f64, ratio(t2,t1), n2 as f64/n1 as f64);
    assert!(ratio(t2,t1) < (n2/n1/5) as f64); println!("  OK O(log N)");
});

bench!(bitmask_o1, {
    hdr("Bitmask O(1)"); setup();
    transact(|tx| tx.grant(1, 100, u64::MAX)).unwrap();
    let mut r = vec![];
    for m in [1u64, 0xFF, 0xFFFF, 0xFFFFFFFF, u64::MAX] {
        let t = avg(1000, || { let _ = check(1, 100, m); });
        println!("  0x{m:016X}: {t:?}"); r.push(t);
    }
    let v = r.iter().max().unwrap().as_nanos() as f64 / r.iter().min().unwrap().as_nanos() as f64;
    println!("\n  Variance: {v:.2}x"); assert!(v < 3.0); println!("  OK O(1)");
});

bench!(relation_merge, {
    hdr("Relation Merge (pre-merged = O(1))");
    let mut r = vec![];
    for k in [1, 2, 5, 10] {
        setup();
        transact(|tx| { for i in 0..k { tx.grant(1, 100, 1u64 << i)?; } Ok(()) }).unwrap();
        let t = avg(1000, || { let _ = check(1, 100, 1); });
        println!("  k={k:2}: {t:?}"); r.push((k, t));
    }
    let v = ratio(r.last().unwrap().1, r[0].1);
    println!("\n  10x relations -> {v:.2}x time"); assert!(v < 3.0); println!("  OK O(1) pre-merged");
});

bench!(grant_throughput, {
    hdr("Grant Throughput"); setup();
    let g: Vec<_> = (0..10000u64).map(|i| (i, i+100000, 7u64)).collect();
    let t = Instant::now(); bench_grant(&g); let e = t.elapsed();
    println!("  10K grants: {e:?} ({}/s)", fmt_num((10000.0 / e.as_secs_f64()) as u64));
});

bench!(check_throughput, {
    hdr("Check Throughput"); setup();
    bench_grant(&(0..1000u64).map(|i| (i, 1000+(i%100), 7u64)).collect::<Vec<_>>());
    let t = Instant::now();
    for i in 0..10000u64 { let _ = check(i%1000, 1000+(i%100), 1); }
    let e = t.elapsed();
    println!("  10K checks: {e:?} ({}/s)", fmt_num((10000.0 / e.as_secs_f64()) as u64));
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

    println!("  Target: {} grants in batches of {}\n", fmt_num(total), fmt_num(batch_size));

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
            println!("  Progress: {}% ({} grants)", (b + 1) * 100 / batches, fmt_num((b + 1) * batch_size));
        }
    }
    let setup_time = t.elapsed();
    println!("\n  Setup: {:?} ({}/s)\n", setup_time, fmt_num((total as f64 / setup_time.as_secs_f64()) as u64));

    println!("  Random lookup: {:?}", avg(5000, || { let _ = get_mask(rand() % 100_000, rand() % 10_000 + 1_000_000); }));
    println!("  Random check:  {:?}", avg(5000, || { let _ = check(rand() % 100_000, rand() % 10_000 + 1_000_000, 1); }));
});

bench!(stress_deep_inheritance, {
    hdr("STRESS: Deep Inheritance Chains");
    setup();

    let chains = 1000u64;
    let depth = 10u64;

    println!("  Creating {} inheritance chains of depth {}\n", chains, depth);

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
    println!("  Setup: {:?}\n", t.elapsed());

    for d in [1, 3, 5, 7, 9] {
        let t = avg(1000, || {
            let c = rand() % chains;
            let base = c * 100;
            let _ = get_mask(base + d, base + 1000);
        });
        println!("  Depth {}: {:?}", d, t);
    }
});

bench!(stress_concurrent_reads, {
    hdr("STRESS: Concurrent Read Performance (8 threads)");
    setup();

    let grants: Vec<_> = (0..50_000u64).map(|i| (i % 10_000, i / 10 + 100_000, 7u64)).collect();
    bench_grant(&grants);
    println!("  Setup: 50K grants across 10K users\n");

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

    println!("  {} threads x {} ops = {} total", num_threads, fmt_num(ops_per_thread), fmt_num(total));
    println!("  Time: {:?}", elapsed);
    println!("  Throughput: {}/s", fmt_num((total as f64 / elapsed.as_secs_f64()) as u64));
});

bench!(stress_labels, {
    hdr("STRESS: Label Operations");

    let count = 50_000u64;
    println!("  Creating {} labeled entities\n", fmt_num(count));

    setup();
    let t = Instant::now();
    for i in 0..1000 {
        create_entity(&format!("single_{:06}", i)).unwrap();
    }
    let single_rate = 1000.0 / t.elapsed().as_secs_f64();
    println!("  Single create (1K): {:?} ({}/s)", t.elapsed(), fmt_num(single_rate as u64));

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
    println!("  Batched create ({}): {:?} ({}/s)", fmt_num(count), t.elapsed(), fmt_num(batch_rate as u64));
    println!("  Speedup: {:.0}x\n", batch_rate / single_rate);

    let t = Instant::now();
    for _ in 0..10_000 {
        let _ = get_id_by_label(&format!("entity_{:06}", rand() % count));
    }
    println!("  Lookup by name (10K): {:?}", t.elapsed());
});

bench!(stress_enterprise_sim, {
    hdr("STRESS: Enterprise Simulation");
    println!("  Simulating: 10K employees, 1K teams, 100K documents\n");
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
            println!("  Teams: {}/{}...", team + 1, teams);
        }
    }
    println!("\n  Setup: {:?}", t.elapsed());

    let ops = 50_000u64;
    let t = Instant::now();
    for _ in 0..ops {
        let emp = rand() % employees;
        let team = emp / emp_per_team;
        let doc = team * docs_per_team + rand() % docs_per_team + 1_000_000;
        let _ = check(emp, doc, READ);
    }
    println!("  {} permission checks: {:?} ({}/s)",
        fmt_num(ops), t.elapsed(), fmt_num((ops as f64 / t.elapsed().as_secs_f64()) as u64));
});

// Prefixed with 'a_' to run first (before CPU heats up from stress tests)
bench!(a_baseline, {
    setup();
    transact(|tx| tx.grant(1, 100, 7)).unwrap();
    let l = avg(1000, || { let _ = get_mask(1, 100); });
    let c = avg(1000, || { let _ = check(1, 100, 1); });
    println!("\n======================================================================");
    println!("                    CAPBIT BASELINE (clean DB)");
    println!("======================================================================");
    println!("  Lookup: {:>12?}  Check: {:>12?}", l, c);
    println!("  O(log N) scaling  |  O(1) bitmask");
    println!("======================================================================\n");
});
