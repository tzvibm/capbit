//! Large scale O(log N) verification with inheritance

use capbit::{
    init, bootstrap, protected, check_access, set_inheritance,
    clear_all, test_lock, SystemCap,
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

// =============================================================================
// TEST 1: Large Scale Entity Count
// =============================================================================

#[test]
fn test1_large_scale_entities() {
    let _lock = setup_bootstrapped();

    println!("\n==========================================================");
    println!("TEST 1: Large Scale Entity Count");
    println!("==========================================================\n");

    let scales = [1000, 5000, 10000, 20000];
    let mut results: Vec<(usize, Duration)> = Vec::new();

    for &n in &scales {
        clear_all().unwrap();
        bootstrap("root").unwrap();

        print!("  Creating {} entities...", n);
        std::io::Write::flush(&mut std::io::stdout()).ok();

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
        println!(" {:?} per lookup", avg);
    }

    print_scaling_analysis(&results);
}

// =============================================================================
// TEST 2: Deep Inheritance Chain
// =============================================================================

#[test]
fn test2_deep_inheritance() {
    let _lock = setup_bootstrapped();

    println!("\n==========================================================");
    println!("TEST 2: Deep Inheritance Chain");
    println!("==========================================================\n");

    let depths = [1, 5, 10, 20, 50];
    let mut results: Vec<(usize, Duration)> = Vec::new();

    for &depth in &depths {
        clear_all().unwrap();
        bootstrap("root").unwrap();

        print!("  Depth {}...", depth);
        std::io::Write::flush(&mut std::io::stdout()).ok();

        // Create resource with capability
        protected::create_entity("user:root", "resource", "doc").unwrap();
        protected::set_capability("user:root", "resource:doc", "viewer", 0x01).unwrap();

        // Create chain: team0 <- team1 <- team2 <- ... <- teamN <- alice
        // team0 has direct grant
        protected::create_entity("user:root", "team", "t0").unwrap();
        protected::set_grant("user:root", "team:t0", "viewer", "resource:doc").unwrap();

        // Build inheritance chain
        for i in 1..depth {
            protected::create_entity("user:root", "team", &format!("t{}", i)).unwrap();
            set_inheritance(
                &format!("team:t{}", i),
                "resource:doc",
                &format!("team:t{}", i - 1)
            ).unwrap();
        }

        // Alice inherits from last team in chain
        protected::create_entity("user:root", "user", "alice").unwrap();
        if depth > 0 {
            set_inheritance("user:alice", "resource:doc", &format!("team:t{}", depth - 1)).unwrap();
        }

        let avg = avg_time(500, || {
            let _ = check_access("user:alice", "resource:doc", None).unwrap();
        });

        // Verify alice gets access
        let caps = check_access("user:alice", "resource:doc", None).unwrap();
        let has_access = caps & 0x01 != 0;

        results.push((depth, avg));
        println!(" {:?} (access={})", avg, has_access);
    }

    println!("\n  Depth Scaling:");
    for i in 1..results.len() {
        let (d1, t1) = results[i-1];
        let (d2, t2) = results[i];
        let d_ratio = d2 as f64 / d1 as f64;
        let t_ratio = t2.as_nanos() as f64 / t1.as_nanos() as f64;
        println!("    depth {} -> {}: {:.1}x depth, {:.2}x time", d1, d2, d_ratio, t_ratio);
    }
}

// =============================================================================
// TEST 3: Wide Inheritance (Many Sources)
// =============================================================================

#[test]
fn test3_wide_inheritance() {
    let _lock = setup_bootstrapped();

    println!("\n==========================================================");
    println!("TEST 3: Wide Inheritance (Many Sources)");
    println!("==========================================================\n");

    let widths = [1, 5, 10, 20, 50];
    let mut results: Vec<(usize, Duration)> = Vec::new();

    for &width in &widths {
        clear_all().unwrap();
        bootstrap("root").unwrap();

        print!("  Width {}...", width);
        std::io::Write::flush(&mut std::io::stdout()).ok();

        protected::create_entity("user:root", "resource", "doc").unwrap();
        protected::create_entity("user:root", "user", "alice").unwrap();

        // Create width teams, each with a different capability bit
        for i in 0..width {
            let cap_bit = 1u64 << (i % 64);
            protected::create_entity("user:root", "team", &format!("t{}", i)).unwrap();
            protected::set_capability("user:root", "resource:doc", &format!("role{}", i), cap_bit).unwrap();
            protected::set_grant("user:root", &format!("team:t{}", i), &format!("role{}", i), "resource:doc").unwrap();

            // Alice inherits from each team
            set_inheritance("user:alice", "resource:doc", &format!("team:t{}", i)).unwrap();
        }

        let avg = avg_time(500, || {
            let _ = check_access("user:alice", "resource:doc", None).unwrap();
        });

        let caps = check_access("user:alice", "resource:doc", None).unwrap();
        results.push((width, avg));
        println!(" {:?} (caps=0x{:x})", avg, caps);
    }

    println!("\n  Width Scaling:");
    for i in 1..results.len() {
        let (w1, t1) = results[i-1];
        let (w2, t2) = results[i];
        let w_ratio = w2 as f64 / w1 as f64;
        let t_ratio = t2.as_nanos() as f64 / t1.as_nanos() as f64;
        println!("    width {} -> {}: {:.1}x sources, {:.2}x time", w1, w2, w_ratio, t_ratio);
    }
}

// =============================================================================
// TEST 4: Many Grants Per User
// =============================================================================

#[test]
fn test4_many_grants_per_user() {
    let _lock = setup_bootstrapped();

    println!("\n==========================================================");
    println!("TEST 4: Many Grants Per User");
    println!("==========================================================\n");

    let grant_counts = [1, 10, 50, 100, 200];
    let mut results: Vec<(usize, Duration)> = Vec::new();

    for &count in &grant_counts {
        clear_all().unwrap();
        bootstrap("root").unwrap();

        print!("  {} grants...", count);
        std::io::Write::flush(&mut std::io::stdout()).ok();

        protected::create_entity("user:root", "user", "alice").unwrap();

        // Create many resources, alice has grant on each
        for i in 0..count {
            protected::create_entity("user:root", "resource", &format!("r{}", i)).unwrap();
            protected::set_capability("user:root", &format!("resource:r{}", i), "viewer", 0x01).unwrap();
            protected::set_grant("user:root", "user:alice", "viewer", &format!("resource:r{}", i)).unwrap();
        }

        // Query the last resource
        let target = format!("resource:r{}", count - 1);
        let avg = avg_time(500, || {
            let _ = check_access("user:alice", &target, None).unwrap();
        });

        results.push((count, avg));
        println!(" {:?}", avg);
    }

    println!("\n  Grants Scaling (should be ~O(1) with prefix scan):");
    for i in 1..results.len() {
        let (g1, t1) = results[i-1];
        let (g2, t2) = results[i];
        let g_ratio = g2 as f64 / g1 as f64;
        let t_ratio = t2.as_nanos() as f64 / t1.as_nanos() as f64;
        println!("    {} -> {} grants: {:.1}x grants, {:.2}x time", g1, g2, g_ratio, t_ratio);
    }
}

// =============================================================================
// TEST 5: Combined Stress Test
// =============================================================================

#[test]
fn test5_combined_stress() {
    let _lock = setup_bootstrapped();

    println!("\n==========================================================");
    println!("TEST 5: Combined Stress Test");
    println!("==========================================================\n");

    clear_all().unwrap();
    bootstrap("root").unwrap();

    println!("  Setting up: 5000 users, 100 resources, 3-level inheritance...");

    // Create 100 resources
    for i in 0..100 {
        protected::create_entity("user:root", "resource", &format!("r{}", i)).unwrap();
        protected::set_capability("user:root", &format!("resource:r{}", i), "viewer", 0x01).unwrap();
        protected::set_capability("user:root", &format!("resource:r{}", i), "editor", 0x03).unwrap();
    }

    // Create 50 teams in 3 levels
    for i in 0..50 {
        protected::create_entity("user:root", "team", &format!("t{}", i)).unwrap();
    }

    // Level 1 teams (0-9) get direct grants
    for i in 0..10 {
        for r in 0..10 {
            protected::set_grant(
                "user:root",
                &format!("team:t{}", i),
                "viewer",
                &format!("resource:r{}", i * 10 + r)
            ).unwrap();
        }
    }

    // Level 2 teams (10-29) inherit from level 1
    for i in 10..30 {
        let parent = i % 10;
        for r in 0..10 {
            set_inheritance(
                &format!("team:t{}", i),
                &format!("resource:r{}", parent * 10 + r),
                &format!("team:t{}", parent)
            ).unwrap();
        }
    }

    // Level 3 teams (30-49) inherit from level 2
    for i in 30..50 {
        let parent = 10 + (i % 20);
        let grandparent = parent % 10;
        for r in 0..10 {
            set_inheritance(
                &format!("team:t{}", i),
                &format!("resource:r{}", grandparent * 10 + r),
                &format!("team:t{}", parent)
            ).unwrap();
        }
    }

    // Create 5000 users, each in a random team
    for i in 0..5000 {
        protected::create_entity("user:root", "user", &format!("u{}", i)).unwrap();
        let team = i % 50;
        let resource_base = (team % 10) * 10;
        for r in 0..10 {
            set_inheritance(
                &format!("user:u{}", i),
                &format!("resource:r{}", resource_base + r),
                &format!("team:t{}", team)
            ).unwrap();
        }
    }

    println!("  Setup complete. Running queries...\n");

    // Test queries at different "depths"
    let test_cases = [
        ("user:u5", "resource:r5", "Level 1 team member"),      // via team:t0
        ("user:u15", "resource:r5", "Level 2 team member"),     // via team:t10 -> team:t0
        ("user:u35", "resource:r5", "Level 3 team member"),     // via team:t30 -> team:t10 -> team:t0
        ("user:u4999", "resource:r95", "Deep user, far resource"),
    ];

    for (user, resource, desc) in test_cases {
        let avg = avg_time(500, || {
            let _ = check_access(user, resource, None).unwrap();
        });
        let caps = check_access(user, resource, None).unwrap();
        println!("  {}: {:?} (caps=0x{:x})", desc, avg, caps);
    }

    println!("\n  ✓ Stress test complete!");
}

// =============================================================================
// Helper
// =============================================================================

fn print_scaling_analysis(results: &[(usize, Duration)]) {
    println!("\n  Scaling Analysis:");
    if results.len() >= 2 {
        let (n1, t1) = results[0];
        let (n2, t2) = results[results.len() - 1];

        let n_ratio = n2 as f64 / n1 as f64;
        let t_ratio = t2.as_nanos() as f64 / t1.as_nanos() as f64;
        let expected_linear = n_ratio;
        let expected_log = (n2 as f64).ln() / (n1 as f64).ln();

        println!("  Data growth: {:.1}x ({} -> {})", n_ratio, n1, n2);
        println!("  Time growth: {:.2}x", t_ratio);
        println!("  Expected if O(N):     {:.1}x", expected_linear);
        println!("  Expected if O(log N): {:.2}x", expected_log);

        if t_ratio < expected_linear / 2.0 {
            println!("\n  ✓ VERIFIED: Sub-linear scaling!");
        } else {
            println!("\n  ⚠ Scaling issue detected");
        }
    }

    println!("\n  Step-by-step:");
    for i in 1..results.len() {
        let (n1, t1) = results[i-1];
        let (n2, t2) = results[i];
        let n_ratio = n2 as f64 / n1 as f64;
        let t_ratio = t2.as_nanos() as f64 / t1.as_nanos() as f64;
        println!("    {} -> {}: {:.1}x data, {:.2}x time", n1, n2, n_ratio, t_ratio);
    }
}
