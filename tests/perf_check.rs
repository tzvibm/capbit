//! Performance check - O(log N) verification

use capbit::{
    init, bootstrap, protected, check_access,
    clear_all, test_lock,
};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use std::sync::Once;

static INIT: Once = Once::new();
static mut TEST_DIR: Option<TempDir> = None;

fn setup() {
    INIT.call_once(|| {
        let dir = TempDir::new().unwrap();
        init(dir.path().to_str().unwrap()).unwrap();
        unsafe { TEST_DIR = Some(dir); }
    });
}

fn setup_bootstrapped() -> std::sync::MutexGuard<'static, ()> {
    let lock = test_lock();
    setup();
    clear_all().unwrap();
    bootstrap("root").unwrap();
    lock
}

fn avg_time<F>(iterations: usize, mut f: F) -> Duration
where
    F: FnMut(),
{
    for _ in 0..10 { f(); }
    let start = Instant::now();
    for _ in 0..iterations { f(); }
    start.elapsed() / iterations as u32
}

#[test]
fn perf_ologn_check() {
    let _lock = setup_bootstrapped();

    println!("\n=== O(log N) Verification ===\n");

    let scales = [100, 500, 1000, 2000];
    let mut results: Vec<(usize, Duration)> = Vec::new();

    for &n in &scales {
        clear_all().unwrap();
        bootstrap("root").unwrap();

        protected::create_entity("user:root", "resource", "target").unwrap();
        protected::set_capability("user:root", "resource:target", "member", 0x01).unwrap();

        for i in 0..n {
            protected::create_entity("user:root", "user", &format!("u{}", i)).unwrap();
            protected::set_grant("user:root", &format!("user:u{}", i), "member", "resource:target").unwrap();
        }

        protected::create_entity("user:root", "user", "test_user").unwrap();
        protected::set_capability("user:root", "resource:target", "editor", 0x03).unwrap();
        protected::set_grant("user:root", "user:test_user", "editor", "resource:target").unwrap();

        let avg = avg_time(1000, || {
            let _ = check_access("user:test_user", "resource:target", None).unwrap();
        });

        results.push((n, avg));
        println!("  N={:5} entities: {:?} per lookup", n, avg);
    }

    println!("\n  Analysis:");
    if results.len() >= 2 {
        let (n1, t1) = results[0];
        let (n2, t2) = results[results.len() - 1];

        let n_ratio = n2 as f64 / n1 as f64;
        let t_ratio = t2.as_nanos() as f64 / t1.as_nanos() as f64;
        let expected_linear = n_ratio;
        let expected_log = (n2 as f64).ln() / (n1 as f64).ln();

        println!("  Data growth: {:.1}x ({} -> {})", n_ratio, n1, n2);
        println!("  Time growth: {:.2}x", t_ratio);
        println!("  If O(N):     {:.1}x expected", expected_linear);
        println!("  If O(log N): {:.2}x expected", expected_log);

        if t_ratio < expected_linear / 2.0 {
            println!("\n  ✓ Sub-linear: O(log N) confirmed!");
        } else {
            println!("\n  ⚠ Linear or worse scaling detected");
        }
    }
}
