//! Adaptive Planner - Automatic write batching for LMDB
//!
//! Fully automatic: no manual sync, no user intervention.
//! Dynamically adjusts batch size based on write pressure.

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::error::{CapbitError, Result};
use crate::tx::transact;

/// Operation types
#[derive(Debug)]
pub enum Op {
    Grant { subject: u64, object: u64, mask: u64 },
    Revoke { subject: u64, object: u64 },
    SetRole { object: u64, role: u64, mask: u64 },
    SetInherit { object: u64, child: u64, parent: u64 },
    RemoveInherit { object: u64, child: u64 },
}

/// Batch of operations with smart merging
struct Batch {
    grants: HashMap<(u64, u64), u64>,
    revokes: HashSet<(u64, u64)>,
    roles: HashMap<(u64, u64), u64>,
    inherits: HashMap<(u64, u64), u64>,
    rm_inherits: HashSet<(u64, u64)>,
    count: usize,  // Track count to avoid summing 5 collections
}

impl Batch {
    fn new() -> Self {
        Batch {
            grants: HashMap::with_capacity(256),
            revokes: HashSet::new(),
            roles: HashMap::new(),
            inherits: HashMap::new(),
            rm_inherits: HashSet::new(),
            count: 0,
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.count
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.count == 0
    }

    fn add(&mut self, op: Op) {
        self.count += 1;
        match op {
            Op::Grant { subject, object, mask } => {
                *self.grants.entry((subject, object)).or_insert(0) |= mask;
            }
            Op::Revoke { subject, object } => {
                self.grants.remove(&(subject, object));
                self.revokes.insert((subject, object));
            }
            Op::SetRole { object, role, mask } => {
                self.roles.insert((object, role), mask);
            }
            Op::SetInherit { object, child, parent } => {
                self.rm_inherits.remove(&(object, child));  // O(1) now
                self.inherits.insert((object, child), parent);
            }
            Op::RemoveInherit { object, child } => {
                self.inherits.remove(&(object, child));
                self.rm_inherits.insert((object, child));
            }
        }
    }

    fn flush(&mut self) -> Result<()> {
        if self.is_empty() {
            return Ok(());
        }

        self.count = 0;
        // Drain directly into transaction - no intermediate allocation
        transact(|tx| {
            for ((s, o), m) in self.grants.drain() {
                tx.grant(s, o, m)?;
            }
            for (s, o) in self.revokes.drain() {
                tx.revoke(s, o)?;
            }
            for ((o, r), m) in self.roles.drain() {
                tx.set_role(o, r, m)?;
            }
            for ((o, c), p) in self.inherits.drain() {
                tx.set_inherit(o, c, p)?;
            }
            for (o, c) in self.rm_inherits.drain() {
                tx.remove_inherit(o, c)?;
            }
            Ok(())
        })
    }
}

/// Adaptive buffer sizing
struct Adaptive {
    capacity: usize,
    min: usize,
    max: usize,
    // Rolling stats
    fills: u32,      // Hit capacity
    timeouts: u32,   // Flushed on timeout
    window: u32,     // Total flushes in window
}

impl Adaptive {
    fn new() -> Self {
        Adaptive {
            capacity: 200,  // Start moderate
            min: 50,
            max: 5000,
            fills: 0,
            timeouts: 0,
            window: 0,
        }
    }

    #[inline]
    fn should_flush(&self, batch_len: usize, elapsed: Duration) -> bool {
        batch_len >= self.capacity || elapsed >= Duration::from_millis(20)
    }

    fn record(&mut self, batch_len: usize, was_timeout: bool) {
        self.window += 1;
        if was_timeout {
            self.timeouts += 1;
        } else if batch_len >= self.capacity {
            self.fills += 1;
        }

        // Adapt every 8 flushes
        if self.window >= 8 {
            self.adapt();
        }
    }

    fn adapt(&mut self) {
        if self.window == 0 {
            return;
        }

        let fill_pct = self.fills * 100 / self.window;
        let timeout_pct = self.timeouts * 100 / self.window;

        if fill_pct > 60 {
            // High pressure: grow
            self.capacity = (self.capacity * 3 / 2).min(self.max);
        } else if timeout_pct > 60 && self.capacity > self.min * 2 {
            // Low pressure: shrink
            self.capacity = (self.capacity * 2 / 3).max(self.min);
        }

        self.fills = 0;
        self.timeouts = 0;
        self.window = 0;
    }
}

/// The planner - fire and forget writes
pub struct Planner {
    tx: Sender<Op>,
    #[allow(dead_code)]
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl Planner {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<Op>();

        let handle = thread::spawn(move || {
            writer_loop(rx);
        });

        Planner {
            tx,
            handle: Mutex::new(Some(handle)),
        }
    }

    /// Submit an operation - fire and forget
    #[inline]
    pub fn submit(&self, op: Op) -> Result<()> {
        self.tx.send(op).map_err(|_| CapbitError("Planner closed".into()))
    }
}

impl Default for Planner {
    fn default() -> Self {
        Self::new()
    }
}

/// Writer loop - fully automatic
fn writer_loop(rx: Receiver<Op>) {
    let mut batch = Batch::new();
    let mut adaptive = Adaptive::new();
    let mut last_flush = Instant::now();

    loop {
        // Non-blocking drain of available ops
        loop {
            match rx.try_recv() {
                Ok(op) => {
                    batch.add(op);
                    if adaptive.should_flush(batch.len(), last_flush.elapsed()) {
                        let len = batch.len();
                        let _ = batch.flush();
                        adaptive.record(len, false);
                        last_flush = Instant::now();
                    }
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Final flush on shutdown
                    let _ = batch.flush();
                    return;
                }
            }
        }

        // Flush if we have pending ops and timeout elapsed
        if !batch.is_empty() && last_flush.elapsed() >= Duration::from_millis(20) {
            let len = batch.len();
            let _ = batch.flush();
            adaptive.record(len, true);
            last_flush = Instant::now();
        }

        // Wait for more ops (with timeout for periodic flush)
        match rx.recv_timeout(Duration::from_millis(20)) {
            Ok(op) => {
                batch.add(op);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Timeout flush handled above on next iteration
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                let _ = batch.flush();
                return;
            }
        }
    }
}
