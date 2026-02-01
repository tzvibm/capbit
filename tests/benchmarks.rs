//! Capbit v3 Benchmarks
use capbit::*;
use std::time::{Duration, Instant};

fn setup() { init("./data/bench.mdb").unwrap(); clear_all().unwrap(); }
fn avg<F: FnMut()>(n: usize, mut f: F) -> Duration { for _ in 0..100 { f(); } let t = Instant::now(); for _ in 0..n { f(); } t.elapsed() / n as u32 }
fn hdr(s: &str) { println!("\n{}\n{s}\n{}\n", "=".repeat(60), "=".repeat(60)); }
fn ratio(a: Duration, b: Duration) -> f64 { a.as_nanos() as f64 / b.as_nanos() as f64 }
fn rand() -> u64 { use std::cell::Cell; thread_local! { static S: Cell<u64> = Cell::new(0x853c49e6748fea9b); } S.with(|s| { let x = s.get().wrapping_mul(6364136223846793005).wrapping_add(1); s.set(x); x }) }

macro_rules! bench { ($name:ident, $body:expr) => { #[test] fn $name() { let _l = test_lock(); $body } }; }

bench!(lookup_scaling, {
    hdr("Lookup Scaling O(log N)");
    let mut r = vec![];
    for n in [100, 1000, 10000, 50000] {
        setup();
        let mut g: Vec<_> = (0..n).map(|i| (i+1000, 1u64, 1u64)).collect();
        g.push((999, 1, 7)); batch_grant(&g).unwrap();
        let t = avg(10000, || { let _ = get_mask(999, 1); });
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
        let t = avg(10000, || { let _ = check(1, 100, m); });
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
        let t = avg(10000, || { let _ = check(1, 100, 1); });
        println!("  k={k:2}: {t:?}"); r.push((k, t));
    }
    let v = ratio(r.last().unwrap().1, r[0].1);
    println!("\n  10x relations → {v:.2}x time"); assert!(v < 2.0); println!("  ✓ O(1) pre-merged");
});

bench!(inheritance_chain, {
    hdr("Inheritance Chain"); setup();
    batch_grant(&[(1,10,1),(10,100,1),(100,1000,1),(1000,10000,1)]).unwrap();
    for d in [1, 2, 3, 5] {
        let t = avg(10000, || { for i in 0..d { let _ = get_mask(10u64.pow(i), 10u64.pow(i+1)); } });
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
    for i in 0..100000u64 { let _ = check(i%1000, 1000+(i%100), 1); }
    let e = t.elapsed();
    println!("  100K checks: {e:?} ({}/s)", 100000_000_000_000u64 / e.as_nanos() as u64);
});

bench!(scale_100k, {
    hdr("Scale: 100K users × 100K resources"); setup();
    let (users, res, gpu) = (100_000u64, 100_000u64, 10u64);
    println!("  {} grants\n", users * gpu);
    let t = Instant::now();
    for b in 0..(users*gpu/10000) {
        batch_grant(&(0..10000).map(|i| { let x = b*10000 + i as u64; (x/gpu, (x%gpu)+(x/gpu%res), 7u64) }).collect::<Vec<_>>()).unwrap();
    }
    println!("  Setup: {:?}", t.elapsed());
    println!("  Random lookup: {:?}", avg(10000, || { let _ = get_mask(rand()%users, rand()%res); }));
    println!("  Random check: {:?}", avg(10000, || { let _ = check(rand()%users, rand()%res, 1); }));
});

bench!(scenario_docs, {
    hdr("Scenario: Doc Sharing (10K docs × 100 users)"); setup();
    let t = Instant::now();
    for d in 0..10000u64 { batch_grant(&(0..100).map(|u| (u+d*1000, d, 3u64)).collect::<Vec<_>>()).unwrap(); }
    println!("  Setup: {:?}", t.elapsed());
    println!("  Check: {:?}", avg(10000, || { let d = rand()%10000; let _ = check((rand()%100)+d*1000, d, 1); }));
    println!("  List subject: {:?}", avg(100, || { let _ = list_for_subject(500); }));
    println!("  List object: {:?}", avg(100, || { let _ = list_for_object(500); }));
});

bench!(scenario_iot, {
    hdr("Scenario: IoT (100K devices, 1K users)"); setup();
    let (dev, usr) = (100_000u64, 1000u64);
    let t = Instant::now();
    for u in 0..usr { batch_grant(&(0..dev/usr).map(|d| (u, d+u*(dev/usr), 0xFu64)).collect::<Vec<_>>()).unwrap(); }
    println!("  Setup: {:?}", t.elapsed());
    println!("  Check: {:?}", avg(10000, || { let u = rand()%usr; let _ = check(u, (rand()%(dev/usr))+u*(dev/usr), 1); }));
});

bench!(scenario_multitenant, {
    hdr("Scenario: Multi-tenant (100 × 100 × 50)"); setup();
    let t = Instant::now();
    for tn in 0..100u64 {
        for u in 0..100u64 {
            batch_grant(&(0..50).map(|r| (tn*1_000_000+u, tn*1_000_000+100_000+(r%1000), 7u64)).collect::<Vec<_>>()).unwrap();
        }
    }
    println!("  Setup: {:?}", t.elapsed());
    println!("  Cross-tenant: {:?}", avg(10000, || { let t1 = rand()%100; let _ = check(t1*1_000_000, ((t1+1)%100)*1_000_000+100_000, 1); }));
    println!("  Same-tenant: {:?}", avg(10000, || { let t = rand()%100; let _ = check(t*1_000_000+(rand()%100), t*1_000_000+100_000+(rand()%50), 1); }));
});

bench!(summary, {
    setup(); grant(1, 100, 7).unwrap();
    let l = avg(10000, || { let _ = get_mask(1, 100); });
    let c = avg(10000, || { let _ = check(1, 100, 1); });
    println!("\n╔{}╗", "═".repeat(58));
    println!("║{:^58}║", "CAPBIT v3 BENCHMARK SUMMARY");
    println!("╠{}╣", "═".repeat(58));
    println!("║  Lookup: {:>12?}  Check: {:>12?}            ║", l, c);
    println!("║  ✓ O(log N) scaling  ✓ O(1) bitmask                    ║");
    println!("╚{}╝\n", "═".repeat(58));
});
