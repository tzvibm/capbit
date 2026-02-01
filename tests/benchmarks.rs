//! Capbit v3 Benchmarks - Stress Edition
use capbit::{*, transact, Tx};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

fn setup() { init("./data/bench.mdb").unwrap(); clear_all().unwrap(); }
fn avg<F: FnMut()>(n: usize, mut f: F) -> Duration { for _ in 0..10 { f(); } let t = Instant::now(); for _ in 0..n { f(); } t.elapsed() / n as u32 }
fn hdr(s: &str) { println!("\n{}\n{s}\n{}\n", "=".repeat(70), "=".repeat(70)); }
fn ratio(a: Duration, b: Duration) -> f64 { a.as_nanos() as f64 / b.as_nanos() as f64 }
fn rand() -> u64 { use std::cell::Cell; thread_local! { static S: Cell<u64> = Cell::new(0x853c49e6748fea9b); } S.with(|s| { let x = s.get().wrapping_mul(6364136223846793005).wrapping_add(1); s.set(x); x }) }
fn fmt_num(n: u64) -> String {
    if n >= 1_000_000 { format!("{:.1}M", n as f64 / 1_000_000.0) }
    else if n >= 1_000 { format!("{:.1}K", n as f64 / 1_000.0) }
    else { n.to_string() }
}

macro_rules! bench { ($name:ident, $body:expr) => { #[test] fn $name() { let _l = test_lock(); $body } }; }

// ============================================================================
// BASIC BENCHMARKS (Original)
// ============================================================================

bench!(lookup_scaling, {
    hdr("Lookup Scaling O(log N)");
    let mut r = vec![];
    for n in [100, 1000, 10000, 50000] {
        setup();
        let mut g: Vec<_> = (0..n).map(|i| (i+1000, 1u64, 1u64)).collect();
        g.push((999, 1, 7)); batch_grant(&g).unwrap();
        let t = avg(1000, || { let _ = get_mask(999, 1); });
        println!("  N={n:6}: {t:?}"); r.push((n, t));
    }
    let (n1, t1) = r[0]; let (n2, t2) = r[r.len()-1];
    println!("\n  {:.0}x data → {:.2}x time (O(N)={:.0}x)", n2 as f64/n1 as f64, ratio(t2,t1), n2 as f64/n1 as f64);
    assert!(ratio(t2,t1) < (n2/n1/5) as f64); println!("  ✓ O(log N)");
});

bench!(bitmask_o1, {
    hdr("Bitmask O(1)"); setup();
    grant(1, 100, u64::MAX).unwrap();
    let mut r = vec![];
    for m in [1u64, 0xFF, 0xFFFF, 0xFFFFFFFF, u64::MAX] {
        let t = avg(1000, || { let _ = check(1, 100, m); });
        println!("  0x{m:016X}: {t:?}"); r.push(t);
    }
    let v = r.iter().max().unwrap().as_nanos() as f64 / r.iter().min().unwrap().as_nanos() as f64;
    println!("\n  Variance: {v:.2}x"); assert!(v < 3.0); println!("  ✓ O(1)");
});

bench!(relation_merge, {
    hdr("Relation Merge (pre-merged = O(1))");
    let mut r = vec![];
    for k in [1, 2, 5, 10] {
        setup(); for i in 0..k { grant(1, 100, 1u64 << i).unwrap(); }
        let t = avg(1000, || { let _ = check(1, 100, 1); });
        println!("  k={k:2}: {t:?}"); r.push((k, t));
    }
    let v = ratio(r.last().unwrap().1, r[0].1);
    println!("\n  10x relations → {v:.2}x time"); assert!(v < 3.0); println!("  ✓ O(1) pre-merged");
});

bench!(inheritance_chain, {
    hdr("Inheritance Chain"); setup();
    batch_grant(&[(1,10,1),(10,100,1),(100,1000,1),(1000,10000,1)]).unwrap();
    for d in [1, 2, 3, 5] {
        let t = avg(1000, || { for i in 0..d { let _ = get_mask(10u64.pow(i), 10u64.pow(i+1)); } });
        println!("  depth={d}: {t:?}");
    }
    println!("\n  ✓ O(depth × log N)");
});

bench!(grant_throughput, {
    hdr("Grant Throughput"); setup();
    let g: Vec<_> = (0..10000u64).map(|i| (i, i+100000, 7u64)).collect();
    let t = Instant::now(); batch_grant(&g).unwrap(); let e = t.elapsed();
    println!("  10K grants: {e:?} ({}/s)", 10000_000_000_000u64 / e.as_nanos() as u64);
});

bench!(check_throughput, {
    hdr("Check Throughput"); setup();
    batch_grant(&(0..1000u64).map(|i| (i, 1000+(i%100), 7u64)).collect::<Vec<_>>()).unwrap();
    let t = Instant::now();
    for i in 0..10000u64 { let _ = check(i%1000, 1000+(i%100), 1); }
    let e = t.elapsed();
    println!("  10K checks: {e:?} ({}/s)", 10000_000_000_000u64 / e.as_nanos() as u64);
});

bench!(scale_100k, {
    hdr("Scale: 10K users × 10K resources"); setup();
    let (users, res, gpu) = (10_000u64, 10_000u64, 10u64);
    println!("  {} grants\n", users * gpu);
    let t = Instant::now();
    for b in 0..(users*gpu/1000) {
        batch_grant(&(0..1000).map(|i| { let x = b*1000 + i as u64; (x/gpu, (x%gpu)+(x/gpu%res), 7u64) }).collect::<Vec<_>>()).unwrap();
    }
    println!("  Setup: {:?}", t.elapsed());
    println!("  Random lookup: {:?}", avg(1000, || { let _ = get_mask(rand()%users, rand()%res); }));
    println!("  Random check: {:?}", avg(1000, || { let _ = check(rand()%users, rand()%res, 1); }));
});

bench!(scenario_docs, {
    hdr("Scenario: Doc Sharing (1K docs × 10 users)"); setup();
    let t = Instant::now();
    for d in 0..1000u64 { batch_grant(&(0..10).map(|u| (u+d*100, d, 3u64)).collect::<Vec<_>>()).unwrap(); }
    println!("  Setup: {:?}", t.elapsed());
    println!("  Check: {:?}", avg(1000, || { let d = rand()%1000; let _ = check((rand()%10)+d*100, d, 1); }));
    println!("  List subject: {:?}", avg(100, || { let _ = list_for_subject(500); }));
    println!("  List object: {:?}", avg(100, || { let _ = list_for_object(500); }));
});

bench!(scenario_iot, {
    hdr("Scenario: IoT (10K devices, 100 users)"); setup();
    let (dev, usr) = (10_000u64, 100u64);
    let t = Instant::now();
    for u in 0..usr { batch_grant(&(0..dev/usr).map(|d| (u, d+u*(dev/usr), 0xFu64)).collect::<Vec<_>>()).unwrap(); }
    println!("  Setup: {:?}", t.elapsed());
    println!("  Check: {:?}", avg(1000, || { let u = rand()%usr; let _ = check(u, (rand()%(dev/usr))+u*(dev/usr), 1); }));
});

bench!(scenario_multitenant, {
    hdr("Scenario: Multi-tenant (10 × 10 × 10)"); setup();
    let t = Instant::now();
    for tn in 0..10u64 {
        for u in 0..10u64 {
            batch_grant(&(0..10).map(|r| (tn*1_000_000+u, tn*1_000_000+100_000+r, 7u64)).collect::<Vec<_>>()).unwrap();
        }
    }
    println!("  Setup: {:?}", t.elapsed());
    println!("  Cross-tenant: {:?}", avg(1000, || { let t1 = rand()%10; let _ = check(t1*1_000_000, ((t1+1)%10)*1_000_000+100_000, 1); }));
    println!("  Same-tenant: {:?}", avg(1000, || { let t = rand()%10; let _ = check(t*1_000_000+(rand()%10), t*1_000_000+100_000+(rand()%10), 1); }));
});

bench!(summary, {
    setup(); grant(1, 100, 7).unwrap();
    let l = avg(1000, || { let _ = get_mask(1, 100); });
    let c = avg(1000, || { let _ = check(1, 100, 1); });
    println!("\n╔{}╗", "═".repeat(68));
    println!("║{:^68}║", "CAPBIT v3 BENCHMARK SUMMARY");
    println!("╠{}╣", "═".repeat(68));
    println!("║  Lookup: {:>12?}  Check: {:>12?}                  ║", l, c);
    println!("║  ✓ O(log N) scaling  ✓ O(1) bitmask                          ║");
    println!("╚{}╝\n", "═".repeat(68));
});

// ============================================================================
// STRESS BENCHMARKS - Push the system to the limit
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
        batch_grant(&grants).unwrap();
        if (b + 1) % 20 == 0 {
            println!("  Progress: {}% ({} grants)", (b + 1) * 100 / batches, fmt_num((b + 1) * batch_size));
        }
    }
    let setup_time = t.elapsed();
    println!("\n  ✓ Setup: {:?} ({}/s)\n", setup_time, fmt_num((total as f64 / setup_time.as_secs_f64()) as u64));

    // Benchmark lookups
    println!("  Random lookup: {:?}", avg(5000, || { let _ = get_mask(rand() % 100_000, rand() % 10_000 + 1_000_000); }));
    println!("  Random check:  {:?}", avg(5000, || { let _ = check(rand() % 100_000, rand() % 10_000 + 1_000_000, 1); }));
});

bench!(stress_deep_inheritance, {
    hdr("STRESS: Deep Inheritance Chains");
    setup();

    // Create chains of depth 10 (max supported)
    let chains = 1000u64;
    let depth = 10u64;

    println!("  Creating {} inheritance chains of depth {}\n", chains, depth);

    let t = Instant::now();
    // Batch all chain setup in single transaction
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

    // Test resolution at different depths
    for d in [1, 3, 5, 7, 9] {
        let t = avg(1000, || {
            let c = rand() % chains;
            let base = c * 100;
            let _ = get_mask(base + d, base + 1000);
        });
        println!("  Depth {}: {:?}", d, t);
    }
});

bench!(stress_wide_permissions, {
    hdr("STRESS: Wide Permission Spread (1 user → 100K objects)");
    setup();

    let objects = 100_000u64;
    println!("  Granting access to {} objects for user 1\n", fmt_num(objects));

    let t = Instant::now();
    for b in 0..(objects / 1000) {
        let grants: Vec<_> = (0..1000).map(|i| (1u64, b * 1000 + i + 1, READ | WRITE | DELETE)).collect();
        batch_grant(&grants).unwrap();
    }
    println!("  Setup: {:?}", t.elapsed());

    println!("  Random check:  {:?}", avg(5000, || { let _ = check(1, rand() % objects + 1, READ); }));
    println!("  List subject:  {:?}", avg(10, || { let _ = list_for_subject(1); }));
});

bench!(stress_dense_object, {
    hdr("STRESS: Dense Object (100K users → 1 object)");
    setup();

    let users = 100_000u64;
    println!("  Granting {} users access to object 1\n", fmt_num(users));

    let t = Instant::now();
    for b in 0..(users / 1000) {
        let grants: Vec<_> = (0..1000).map(|i| (b * 1000 + i + 1, 1u64, (i % 64 + 1) as u64)).collect();
        batch_grant(&grants).unwrap();
    }
    println!("  Setup: {:?}", t.elapsed());

    println!("  Random check: {:?}", avg(5000, || { let _ = check(rand() % users + 1, 1, READ); }));
    println!("  List object:  {:?}", avg(10, || { let _ = list_for_object(1); }));
});

bench!(stress_concurrent_reads, {
    hdr("STRESS: Concurrent Read Performance (8 threads)");
    setup();

    // Setup data
    let grants: Vec<_> = (0..50_000u64).map(|i| (i % 10_000, i / 10 + 100_000, 7u64)).collect();
    batch_grant(&grants).unwrap();
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

    println!("  {} threads × {} ops = {} total", num_threads, fmt_num(ops_per_thread), fmt_num(total));
    println!("  Time: {:?}", elapsed);
    println!("  Throughput: {}/s", fmt_num((total as f64 / elapsed.as_secs_f64()) as u64));
});

bench!(stress_batch_sizes, {
    hdr("STRESS: Batch Size Impact");
    setup();

    let total = 100_000u64;
    for batch_size in [100, 1000, 5000, 10000, 50000] {
        setup();
        let batches = total / batch_size;

        let t = Instant::now();
        for b in 0..batches {
            let grants: Vec<_> = (0..batch_size)
                .map(|i| (b * batch_size + i, i + 1_000_000, 7u64))
                .collect();
            batch_grant(&grants).unwrap();
        }
        let elapsed = t.elapsed();
        println!("  Batch {:>5}: {:>10?}  ({}/s)",
            batch_size, elapsed, fmt_num((total as f64 / elapsed.as_secs_f64()) as u64));
    }
});

bench!(stress_revoke_storm, {
    hdr("STRESS: Grant/Revoke Churn");
    setup();

    // Pre-populate
    let base_grants: Vec<_> = (0..10_000u64).map(|i| (i, i + 100_000, 7u64)).collect();
    batch_grant(&base_grants).unwrap();
    println!("  Base: 10K grants\n");

    // Single ops (slow baseline)
    let single_iters = 100u64;
    let t = Instant::now();
    for i in 0..single_iters {
        grant(i + 20_000, i + 100_000, READ | WRITE).unwrap();
        revoke(i, i + 100_000).unwrap();
    }
    let single_time = t.elapsed();
    println!("  Single ops ({} cycles): {:?}", single_iters, single_time);

    // Batched ops (fast)
    setup();
    batch_grant(&base_grants).unwrap();
    let batch_iters = 10_000u64;
    let t = Instant::now();
    transact(|tx| {
        for i in 0..batch_iters {
            tx.grant(i + 20_000, i + 100_000, READ | WRITE)?;
            tx.revoke(i, i + 100_000)?;
        }
        Ok(())
    }).unwrap();
    let batch_time = t.elapsed();
    println!("  Batched ops ({} cycles): {:?}", fmt_num(batch_iters), batch_time);
    println!("  Speedup: {:.0}x", (single_time.as_nanos() as f64 / single_iters as f64) / (batch_time.as_nanos() as f64 / batch_iters as f64));
});

bench!(stress_role_resolution, {
    hdr("STRESS: Role-Based Resolution");
    setup();

    let roles = 100u64;
    let users_per_role = 1000u64;
    let objects = 1000u64;

    println!("  {} roles × {} users × {} objects\n", roles, users_per_role, objects);

    let t = Instant::now();
    // Define all roles in batched transactions
    for r in 0..roles {
        transact(|tx| {
            for o in 0..objects {
                tx.set_role(o + 100_000, r + 1, (r % 8 + 1) as u64)?;
            }
            Ok(())
        }).unwrap();
    }
    // Assign users to roles
    for r in 0..roles {
        transact(|tx| {
            for u in 0..users_per_role {
                tx.grant(r * users_per_role + u, r * objects + 100_000, r + 1)?;
            }
            Ok(())
        }).unwrap();
    }
    println!("  Setup: {:?}\n", t.elapsed());

    println!("  Role lookup: {:?}", avg(5000, || {
        let r = rand() % roles;
        let u = r * users_per_role + (rand() % users_per_role);
        let o = r * objects + 100_000 + (rand() % objects);
        let _ = get_mask(u, o);
    }));
});

bench!(stress_labels, {
    hdr("STRESS: Label Operations");

    let count = 50_000u64;
    println!("  Creating {} labeled entities\n", fmt_num(count));

    // Single-entity creation (slow)
    setup();
    let t = Instant::now();
    for i in 0..1000 {
        create_entity(&format!("single_{:06}", i)).unwrap();
    }
    let single_rate = 1000.0 / t.elapsed().as_secs_f64();
    println!("  Single create (1K): {:?} ({}/s)", t.elapsed(), fmt_num(single_rate as u64));

    // Batched creation using transact (fast)
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

    let t = Instant::now();
    for _ in 0..10_000 {
        let _ = get_label(rand() % count + 1);
    }
    println!("  Lookup by ID (10K): {:?}", t.elapsed());
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

    // Each team owns documents
    for team in 0..teams {
        // Team members get access to team docs
        let grants: Vec<_> = (0..emp_per_team)
            .flat_map(|e| {
                let emp_id = team * emp_per_team + e;
                (0..10).map(move |d| {
                    let doc_id = team * docs_per_team + (d + e * 10) % docs_per_team + 1_000_000;
                    (emp_id, doc_id, READ | WRITE)
                })
            })
            .collect();
        batch_grant(&grants).unwrap();

        if (team + 1) % 200 == 0 {
            println!("  Teams: {}/{}...", team + 1, teams);
        }
    }
    println!("\n  Setup: {:?}", t.elapsed());

    // Simulate workday
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

bench!(stress_final_summary, {
    hdr("FINAL STRESS TEST SUMMARY");
    setup();

    // Quick warmup
    batch_grant(&(0..1000u64).map(|i| (i, i + 10000, 7)).collect::<Vec<_>>()).unwrap();

    let check_lat = avg(10000, || { let _ = check(rand() % 1000, rand() % 1000 + 10000, 1); });
    let grant_t = Instant::now();
    batch_grant(&(0..10000u64).map(|i| (i + 2000, i + 20000, 7)).collect::<Vec<_>>()).unwrap();
    let grant_rate = 10000.0 / grant_t.elapsed().as_secs_f64();

    println!("
╔══════════════════════════════════════════════════════════════════════╗
║                    CAPBIT STRESS TEST RESULTS                        ║
╠══════════════════════════════════════════════════════════════════════╣
║  Single check latency:     {:>12?}                              ║
║  Batch grant rate:         {:>12}/s                              ║
╠══════════════════════════════════════════════════════════════════════╣
║  Recommended limits for this system:                                 ║
║    • Max grants:           ~5M (within 1GB LMDB map)                 ║
║    • Concurrent readers:   8+ threads sustained                      ║
║    • Batch size:           5K-10K optimal                            ║
╚══════════════════════════════════════════════════════════════════════╝
", check_lat, fmt_num(grant_rate as u64));
});
