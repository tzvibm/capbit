//! Capbit - capability-based access control as atomized data.
//!
//! u64 IDs, u64 permission bitmasks, policy-qualified edges (Necessary/Possible/Not),
//! groups with write-time materialized closure, qualified delegation chains, and an
//! audit log written atomically with every mutation. Backed by fjall (LSM-tree).
//!
//! All expansion happens at write time. Every check is a bounded number of key reads.
//! The system governs itself: granting, revoking, defining roles, and creating objects
//! are permission bits checked by the same resolution as everything else.

use fjall::{Config, Keyspace, PartitionCreateOptions, PartitionHandle, PersistMode};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Actor lacks the required permission bits.
    Denied,
    /// Tuple already exists (role definition, bootstrap, object).
    Exists,
    /// Referenced tuple does not exist (e.g. granting an undefined role).
    NotFound,
    /// Operation would create a membership or delegation cycle.
    Cycle,
    /// Underlying storage error.
    Storage(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Denied => write!(f, "denied"),
            Error::Exists => write!(f, "exists"),
            Error::NotFound => write!(f, "not found"),
            Error::Cycle => write!(f, "cycle"),
            Error::Storage(s) => write!(f, "storage: {s}"),
        }
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;
fn err(e: impl std::fmt::Display) -> Error { Error::Storage(e.to_string()) }

// ---------------------------------------------------------------------------
// Policy: the strength qualifier carried by every edge
// ---------------------------------------------------------------------------

/// Edge strength. Composition along a chain takes the minimum; `Not` absorbs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Policy {
    /// Explicit denial / exclusion. Overrides positive edges.
    Not = 0,
    /// Discretionary, conditional (DAC-like).
    Possible = 1,
    /// Structural, mandatory (MAC-like).
    Necessary = 2,
}

impl Policy {
    fn from_u64(v: u64) -> Policy {
        match v {
            0 => Policy::Not,
            1 => Policy::Possible,
            _ => Policy::Necessary,
        }
    }
    fn min(self, other: Policy) -> Policy { if self <= other { self } else { other } }
}

// ---------------------------------------------------------------------------
// Reserved IDs and roles
// ---------------------------------------------------------------------------

pub const _SYSTEM: u64 = 1;
pub const _ROOT: u64 = 2;

pub const _OWNER: u64 = 1;
pub const _ADMIN: u64 = 2;
pub const _EDITOR: u64 = 3;
pub const _VIEWER: u64 = 4;
/// Granting this role on a group object makes the subject a member of the group.
pub const _MEMBER: u64 = 5;

// ---------------------------------------------------------------------------
// Permission bit space
//
// Bits 0..16 are reserved meta-permissions: they govern capbit itself.
// Bits 16..64 are application permissions, free for the embedding app.
// ---------------------------------------------------------------------------

/// Read role definitions, grants, memberships, delegations on an object.
pub const _READ_META: u64 = 1 << 0;
/// Define, update, delete role definitions on an object.
pub const _DEFINE: u64 = 1 << 1;
/// Grant roles (including membership) on an object.
pub const _GRANT: u64 = 1 << 2;
/// Revoke roles on an object.
pub const _REVOKE: u64 = 1 << 3;
/// Create or remove delegation edges on an object.
pub const _DELEGATE: u64 = 1 << 4;
/// Create new objects (checked against `_SYSTEM`).
pub const _CREATE_OBJ: u64 = 1 << 5;
/// Delete an object and cascade all its tuples.
pub const _DELETE_OBJ: u64 = 1 << 6;
/// Set or clear the type-parent of an object.
pub const _SET_PARENT: u64 = 1 << 7;
/// Read the audit log (checked against `_SYSTEM`).
pub const _AUDIT: u64 = 1 << 8;

/// All bits reserved for meta-permissions.
pub const META_BITS: u64 = 0xFFFF;
/// All bits available to the application.
pub const APP_BITS: u64 = !META_BITS;
/// First application bit index.
pub const APP_BIT_BASE: u32 = 16;
/// Application permission bit `n` (0..48).
pub const fn app_bit(n: u32) -> u64 { 1 << (APP_BIT_BASE + n) }

/// Conventional application bits (apps may redefine freely).
pub const APP_READ: u64 = app_bit(0);
pub const APP_WRITE: u64 = app_bit(1);
pub const APP_DELETE: u64 = app_bit(2);

/// Default role masks seeded on bootstrap and object creation.
pub const OWNER_BITS: u64 = u64::MAX;
pub const ADMIN_BITS: u64 = _READ_META | _GRANT | _REVOKE | _DELEGATE | _SET_PARENT | APP_BITS;
pub const EDITOR_BITS: u64 = _READ_META | APP_READ | APP_WRITE;
pub const VIEWER_BITS: u64 = _READ_META | APP_READ;

const MAX_DELEGATION_HOPS: usize = 10;
const MAX_TYPE_DEPTH: usize = 8;

// ---------------------------------------------------------------------------
// Key/value helpers
// ---------------------------------------------------------------------------

#[inline]
fn key(a: u64, b: u64) -> [u8; 16] {
    let mut x = [0u8; 16];
    x[..8].copy_from_slice(&a.to_be_bytes());
    x[8..].copy_from_slice(&b.to_be_bytes());
    x
}

#[inline]
fn key3(a: u64, b: u64, c: u64) -> [u8; 24] {
    let mut x = [0u8; 24];
    x[..8].copy_from_slice(&a.to_be_bytes());
    x[8..16].copy_from_slice(&b.to_be_bytes());
    x[16..].copy_from_slice(&c.to_be_bytes());
    x
}

#[inline]
fn u64_at(k: &[u8], pos: usize) -> u64 { u64::from_be_bytes(k[pos * 8..(pos + 1) * 8].try_into().unwrap()) }

#[inline]
fn val(v: &[u8]) -> u64 { u64_at(v, 0) }

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Resolution result
// ---------------------------------------------------------------------------

/// Three-bucket resolution result. `denied` bits override the other two.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Masks {
    /// Bits held at Necessary strength (structural / mandatory).
    pub necessary: u64,
    /// Bits held at Possible strength (discretionary / conditional).
    pub possible: u64,
    /// Bits explicitly denied. Already subtracted from the other buckets.
    pub denied: u64,
}

impl Masks {
    /// Flat effective mask: everything held at any positive strength, post-deny.
    pub fn allowed(&self) -> u64 { self.necessary | self.possible }
}

/// One row of the audit log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuditEntry {
    pub seq: u64,
    pub ts_ms: u64,
    pub actor: u64,
    pub op: u64,
    pub args: [u64; 4],
}

/// Audit op codes.
pub mod op {
    pub const BOOTSTRAP: u64 = 1;
    pub const CREATE_OBJECT: u64 = 2;
    pub const DELETE_OBJECT: u64 = 3;
    pub const DEFINE_ROLE: u64 = 4;
    pub const UPDATE_ROLE: u64 = 5;
    pub const DELETE_ROLE: u64 = 6;
    pub const GRANT: u64 = 7;
    pub const REVOKE: u64 = 8;
    pub const DELEGATE: u64 = 9;
    pub const UNDELEGATE: u64 = 10;
    pub const SET_PARENT: u64 = 11;
    pub const CLEAR_PARENT: u64 = 12;
    pub const CLEAR: u64 = 13;
}

/// Open options.
#[derive(Debug, Clone)]
pub struct Options {
    /// fsync every mutation. Default true: losing a revoke is a security event.
    /// Set false for bulk loads or when the host journals elsewhere.
    pub durable: bool,
}
impl Default for Options {
    fn default() -> Self { Options { durable: true } }
}

// Membership-edge overrides so closure recomputation can see uncommitted changes.
enum EdgeOp {
    Set(u64, u64, Policy), // (member, group, policy) upserted
    Remove(u64, u64),      // (member, group) removed
    DropGroup(u64),        // all edges into this group removed (group deletion)
}

// ---------------------------------------------------------------------------
// Capbit instance
// ---------------------------------------------------------------------------

/// An embedded capbit store. Cheap to share behind `Arc`; all methods take `&self`.
/// Reads are lock-free; mutations are serialized internally (auth check and write
/// happen under the same lock, so a concurrent revoke cannot race a grant).
pub struct Capbit {
    ks: Keyspace,
    objects: PartitionHandle,         // (object, role) -> mask
    parents: PartitionHandle,         // (object) -> type-parent
    parents_rev: PartitionHandle,     // (parent, object) -> 1
    subjects: PartitionHandle,        // (subject, object, role) -> policy
    subjects_rev: PartitionHandle,    // (object, subject, role) -> policy
    closure: PartitionHandle,         // (member, group) -> policy   (materialized)
    closure_rev: PartitionHandle,     // (group, member) -> policy   (materialized)
    delegations: PartitionHandle,     // (subject, object, role) -> (parent, policy)
    delegations_rev: PartitionHandle, // (object, subject, role) -> (parent, policy)
    audit: PartitionHandle,           // (seq) -> (ts, actor, op, args[4])
    write: Mutex<u64>,                // serializes mutations; holds next audit seq
    durable: bool,
}

impl Capbit {
    pub fn open(path: impl AsRef<Path>) -> Result<Capbit> {
        Capbit::open_with(path, Options::default())
    }

    pub fn open_with(path: impl AsRef<Path>, opts: Options) -> Result<Capbit> {
        std::fs::create_dir_all(path.as_ref()).map_err(err)?;
        let ks = Config::new(path.as_ref()).open().map_err(err)?;
        let o = PartitionCreateOptions::default;
        let p = |name: &str| ks.open_partition(name, o()).map_err(err);
        let audit = p("audit")?;
        let next_seq = audit.last_key_value().map_err(err)?.map(|(k, _)| u64_at(&k, 0) + 1).unwrap_or(0);
        Ok(Capbit {
            objects: p("objects")?,
            parents: p("parents")?,
            parents_rev: p("parents_rev")?,
            subjects: p("subjects")?,
            subjects_rev: p("subjects_rev")?,
            closure: p("closure")?,
            closure_rev: p("closure_rev")?,
            delegations: p("delegations")?,
            delegations_rev: p("delegations_rev")?,
            audit,
            write: Mutex::new(next_seq),
            durable: opts.durable,
            ks,
        })
    }

    // -- storage primitives --------------------------------------------------

    fn get_u64(&self, p: &PartitionHandle, k: &[u8]) -> Result<Option<u64>> {
        Ok(p.get(k).map_err(err)?.map(|v| val(&v)))
    }

    fn commit(&self, batch: fjall::Batch) -> Result<()> {
        batch.commit().map_err(err)?;
        self.ks
            .persist(if self.durable { PersistMode::SyncAll } else { PersistMode::Buffer })
            .map_err(err)
    }

    fn audit_row(&self, b: &mut fjall::Batch, seq: &mut u64, actor: u64, opcode: u64, args: [u64; 4]) {
        let mut v = [0u8; 56];
        for (i, x) in [now_ms(), actor, opcode, args[0], args[1], args[2], args[3]].iter().enumerate() {
            v[i * 8..(i + 1) * 8].copy_from_slice(&x.to_be_bytes());
        }
        b.insert(&self.audit, seq.to_be_bytes(), v);
        *seq += 1;
    }

    fn auth(&self, actor: u64, obj: u64, req: u64) -> Result<()> {
        if self.get_masks(actor, obj)?.allowed() & req == req { Ok(()) } else { Err(Error::Denied) }
    }

    // -- role definitions (type-as-object fallback) ---------------------------

    /// Effective mask for a role on an object, falling back through the
    /// type-parent chain. Undefined roles resolve to 0 — never to anything else.
    pub fn role_mask(&self, obj: u64, role: u64) -> Result<u64> {
        let mut cur = obj;
        for _ in 0..MAX_TYPE_DEPTH {
            if let Some(m) = self.get_u64(&self.objects, &key(cur, role))? {
                return Ok(m);
            }
            match self.get_u64(&self.parents, &cur.to_be_bytes())? {
                Some(p) => cur = p,
                None => return Ok(0),
            }
        }
        Ok(0)
    }

    fn role_defined(&self, obj: u64, role: u64) -> Result<bool> {
        let mut cur = obj;
        for _ in 0..MAX_TYPE_DEPTH {
            if self.objects.get(key(cur, role)).map_err(err)?.is_some() {
                return Ok(true);
            }
            match self.get_u64(&self.parents, &cur.to_be_bytes())? {
                Some(p) => cur = p,
                None => return Ok(false),
            }
        }
        Ok(false)
    }

    // -- resolution -----------------------------------------------------------

    /// (role, policy) grant edges for `sub` on `obj`.
    fn grants_on(&self, sub: u64, obj: u64) -> Result<Vec<(u64, Policy)>> {
        let mut out = Vec::new();
        for kv in self.subjects.prefix(key(sub, obj)) {
            let (k, v) = kv.map_err(err)?;
            out.push((u64_at(&k, 2), Policy::from_u64(val(&v))));
        }
        Ok(out)
    }

    fn bucket(&self, m: &mut Masks, obj: u64, role: u64, pol: Policy) -> Result<()> {
        let mask = self.role_mask(obj, role)?;
        match pol {
            Policy::Not => m.denied |= mask,
            Policy::Possible => m.possible |= mask,
            Policy::Necessary => m.necessary |= mask,
        }
        Ok(())
    }

    /// Full three-bucket resolution: direct grants, group grants through the
    /// materialized closure, and delegation chains. Bounded reads, no recursion.
    pub fn get_masks(&self, sub: u64, obj: u64) -> Result<Masks> {
        let mut m = Masks::default();
        // 1. Direct grants.
        for (role, pol) in self.grants_on(sub, obj)? {
            self.bucket(&mut m, obj, role, pol)?;
        }
        // 2. Group grants: subject's effective groups are precomputed.
        for kv in self.closure.prefix(sub.to_be_bytes()) {
            let (k, v) = kv.map_err(err)?;
            let (group, gpol) = (u64_at(&k, 1), Policy::from_u64(val(&v)));
            if gpol == Policy::Not {
                continue; // excluded from the group: neither its grants nor its denials
            }
            for (role, rpol) in self.grants_on(group, obj)? {
                if role == _MEMBER {
                    continue; // group-of-group edges are already folded into the closure
                }
                let eff = if rpol == Policy::Not { Policy::Not } else { gpol.min(rpol) };
                self.bucket(&mut m, obj, role, eff)?;
            }
        }
        // 3. Delegation chains.
        for kv in self.delegations.prefix(key(sub, obj)) {
            let (k, v) = kv.map_err(err)?;
            let (role, parent, dpol) = (u64_at(&k, 2), u64_at(&v, 0), Policy::from_u64(u64_at(&v, 1)));
            if let Some(pol) = self.resolve_delegation(sub, obj, role, parent, dpol)? {
                self.bucket(&mut m, obj, role, pol)?;
            }
        }
        m.necessary &= !m.denied;
        m.possible &= !m.denied;
        Ok(m)
    }

    /// Walk a delegation chain. The chain resolves iff some ancestor actually
    /// holds the role (directly or via groups); strength is min over all edges.
    fn resolve_delegation(&self, sub: u64, obj: u64, role: u64, parent: u64, pol: Policy) -> Result<Option<Policy>> {
        if pol == Policy::Not {
            return Ok(None); // blocked link
        }
        let mut strength = pol;
        let mut cur = parent;
        let mut seen = HashSet::from([sub]);
        for _ in 0..MAX_DELEGATION_HOPS {
            if !seen.insert(cur) {
                return Ok(None);
            }
            // Does cur hold the role directly?
            if let Some(p) = self.get_u64(&self.subjects, &key3(cur, obj, role))? {
                return Ok(match Policy::from_u64(p) {
                    Policy::Not => None,
                    p => Some(strength.min(p)),
                });
            }
            // Or via its groups?
            let mut best: Option<Policy> = None;
            for kv in self.closure.prefix(cur.to_be_bytes()) {
                let (k, v) = kv.map_err(err)?;
                let (group, gpol) = (u64_at(&k, 1), Policy::from_u64(val(&v)));
                if gpol == Policy::Not {
                    continue;
                }
                if let Some(rp) = self.get_u64(&self.subjects, &key3(group, obj, role))? {
                    let rp = Policy::from_u64(rp);
                    if rp == Policy::Not {
                        continue;
                    }
                    let eff = gpol.min(rp);
                    if best.is_none_or(|b| eff > b) {
                        best = Some(eff);
                    }
                }
            }
            if let Some(b) = best {
                return Ok(Some(strength.min(b)));
            }
            // Follow the chain.
            match self.delegations.get(key3(cur, obj, role)).map_err(err)? {
                Some(v) => {
                    let dp = Policy::from_u64(u64_at(&v, 1));
                    if dp == Policy::Not {
                        return Ok(None);
                    }
                    strength = strength.min(dp);
                    cur = u64_at(&v, 0);
                }
                None => return Ok(None),
            }
        }
        Ok(None)
    }

    /// Flat effective mask (post-deny).
    pub fn get_mask(&self, sub: u64, obj: u64) -> Result<u64> { Ok(self.get_masks(sub, obj)?.allowed()) }

    /// Does `sub` hold every bit in `req` on `obj`?
    pub fn check(&self, sub: u64, obj: u64, req: u64) -> Result<bool> {
        Ok(self.get_mask(sub, obj)? & req == req)
    }

    /// Direct grant lookup (no resolution): does the tuple exist, and how strong?
    pub fn check_subject(&self, sub: u64, obj: u64, role: u64) -> Result<Option<Policy>> {
        Ok(self.get_u64(&self.subjects, &key3(sub, obj, role))?.map(Policy::from_u64))
    }

    // -- group closure maintenance --------------------------------------------

    /// Direct membership edges of `x`: (group, policy), with pending overrides applied.
    fn member_edges(&self, x: u64, ov: &[EdgeOp]) -> Result<Vec<(u64, Policy)>> {
        let mut edges = Vec::new();
        for kv in self.subjects.prefix(x.to_be_bytes()) {
            let (k, v) = kv.map_err(err)?;
            if u64_at(&k, 2) == _MEMBER {
                edges.push((u64_at(&k, 1), Policy::from_u64(val(&v))));
            }
        }
        for o in ov {
            match *o {
                EdgeOp::Set(m, g, p) if m == x => {
                    edges.retain(|(eg, _)| *eg != g);
                    edges.push((g, p));
                }
                EdgeOp::Remove(m, g) if m == x => edges.retain(|(eg, _)| *eg != g),
                EdgeOp::DropGroup(g) => edges.retain(|(eg, _)| *eg != g),
                _ => {}
            }
        }
        Ok(edges)
    }

    /// Recompute the full effective-membership set of `x`.
    ///
    /// Pass 1: widest-path reachability over positive edges (strength = min along
    /// a path, max across paths). Not-edges encountered from any positively
    /// reachable node mark their target group as denied — conservatively sourced
    /// from the unrestricted reachability set. Pass 2: reachability again, with
    /// denied groups impassable. Result: positive strengths plus explicit Not rows.
    fn compute_closure(&self, x: u64, ov: &[EdgeOp]) -> Result<HashMap<u64, Policy>> {
        let mut edges: HashMap<u64, Vec<(u64, Policy)>> = HashMap::new();
        let load = |this: &Capbit, cache: &mut HashMap<u64, Vec<(u64, Policy)>>, n: u64| -> Result<()> {
            if let std::collections::hash_map::Entry::Vacant(e) = cache.entry(n) {
                e.insert(this.member_edges(n, ov)?);
            }
            Ok(())
        };

        // Pass 1: unrestricted positive reachability + collect deny targets.
        let mut best: HashMap<u64, Policy> = HashMap::new();
        let mut denied: HashSet<u64> = HashSet::new();
        let mut queue = VecDeque::from([(x, Policy::Necessary)]);
        while let Some((n, s)) = queue.pop_front() {
            load(self, &mut edges, n)?;
            for &(g, p) in &edges[&n] {
                if p == Policy::Not {
                    denied.insert(g);
                    continue;
                }
                if g == x {
                    continue;
                }
                let ns = s.min(p);
                if best.get(&g).is_none_or(|&old| ns > old) {
                    best.insert(g, ns);
                    queue.push_back((g, ns));
                }
            }
        }

        // Pass 2: reachability with denied groups impassable.
        let mut out: HashMap<u64, Policy> = HashMap::new();
        let mut queue = VecDeque::from([(x, Policy::Necessary)]);
        let mut strengths: HashMap<u64, Policy> = HashMap::new();
        while let Some((n, s)) = queue.pop_front() {
            load(self, &mut edges, n)?;
            for &(g, p) in &edges[&n] {
                if p == Policy::Not || g == x || denied.contains(&g) {
                    continue;
                }
                let ns = s.min(p);
                if strengths.get(&g).is_none_or(|&old| ns > old) {
                    strengths.insert(g, ns);
                    queue.push_back((g, ns));
                }
            }
        }
        out.extend(strengths);
        for g in denied {
            out.insert(g, Policy::Not);
        }
        Ok(out)
    }

    /// Entities whose effective membership could change when `group`'s edges change:
    /// the group itself plus everything currently inside it.
    fn affected_by(&self, group: u64) -> Result<HashSet<u64>> {
        let mut out = HashSet::from([group]);
        for kv in self.closure_rev.prefix(group.to_be_bytes()) {
            let (k, _) = kv.map_err(err)?;
            out.insert(u64_at(&k, 1));
        }
        Ok(out)
    }

    /// Diff-and-write recomputed closures for all affected entities into `batch`.
    fn rebuild_closures(&self, batch: &mut fjall::Batch, affected: &HashSet<u64>, ov: &[EdgeOp]) -> Result<()> {
        for &x in affected {
            let new = self.compute_closure(x, ov)?;
            let mut old: HashMap<u64, Policy> = HashMap::new();
            for kv in self.closure.prefix(x.to_be_bytes()) {
                let (k, v) = kv.map_err(err)?;
                old.insert(u64_at(&k, 1), Policy::from_u64(val(&v)));
            }
            for &g in old.keys() {
                if !new.contains_key(&g) {
                    batch.remove(&self.closure, key(x, g));
                    batch.remove(&self.closure_rev, key(g, x));
                }
            }
            for (&g, &p) in &new {
                if old.get(&g) != Some(&p) {
                    batch.insert(&self.closure, key(x, g), (p as u64).to_be_bytes());
                    batch.insert(&self.closure_rev, key(g, x), (p as u64).to_be_bytes());
                }
            }
        }
        Ok(())
    }

    // -- bootstrap & object lifecycle ------------------------------------------

    fn seed_roles(&self, b: &mut fjall::Batch, obj: u64) {
        for (role, mask) in [(_OWNER, OWNER_BITS), (_ADMIN, ADMIN_BITS), (_EDITOR, EDITOR_BITS), (_VIEWER, VIEWER_BITS)] {
            b.insert(&self.objects, key(obj, role), mask.to_be_bytes());
        }
    }

    fn put_grant(&self, b: &mut fjall::Batch, sub: u64, obj: u64, role: u64, pol: Policy) {
        b.insert(&self.subjects, key3(sub, obj, role), (pol as u64).to_be_bytes());
        b.insert(&self.subjects_rev, key3(obj, sub, role), (pol as u64).to_be_bytes());
    }

    fn del_grant(&self, b: &mut fjall::Batch, sub: u64, obj: u64, role: u64) {
        b.remove(&self.subjects, key3(sub, obj, role));
        b.remove(&self.subjects_rev, key3(obj, sub, role));
    }

    /// One-time setup: creates `_SYSTEM` with default roles and grants `_ROOT` owner.
    /// Everything else follows from the same recursion — no other special case exists.
    pub fn bootstrap(&self) -> Result<(u64, u64)> {
        let mut seq = self.write.lock().map_err(err)?;
        if self.objects.get(key(_SYSTEM, _OWNER)).map_err(err)?.is_some() {
            return Err(Error::Exists);
        }
        let mut b = self.ks.batch();
        self.seed_roles(&mut b, _SYSTEM);
        self.put_grant(&mut b, _ROOT, _SYSTEM, _OWNER, Policy::Necessary);
        self.audit_row(&mut b, &mut seq, _ROOT, op::BOOTSTRAP, [_SYSTEM, _ROOT, 0, 0]);
        self.commit(b)?;
        Ok((_SYSTEM, _ROOT))
    }

    /// Create an object: requires `_CREATE_OBJ` on `_SYSTEM`. Seeds the default
    /// roles and atomically grants the creator owner — so a fresh object is
    /// immediately governable by its creator (this is what makes the recursion work).
    pub fn create_object(&self, actor: u64, obj: u64) -> Result<()> {
        let mut seq = self.write.lock().map_err(err)?;
        self.auth(actor, _SYSTEM, _CREATE_OBJ)?;
        let exists = self.objects.prefix(obj.to_be_bytes()).next().is_some()
            || self.subjects_rev.prefix(obj.to_be_bytes()).next().is_some();
        if exists {
            return Err(Error::Exists);
        }
        let mut b = self.ks.batch();
        self.seed_roles(&mut b, obj);
        self.put_grant(&mut b, actor, obj, _OWNER, Policy::Necessary);
        self.audit_row(&mut b, &mut seq, actor, op::CREATE_OBJECT, [obj, 0, 0, 0]);
        self.commit(b)
    }

    /// Delete an object and cascade every tuple that references it: role
    /// definitions, grants in both directions, memberships (with closure rebuild),
    /// delegations, and parent links. Requires `_DELETE_OBJ` on the object.
    pub fn delete_object(&self, actor: u64, obj: u64) -> Result<()> {
        let mut seq = self.write.lock().map_err(err)?;
        self.auth(actor, obj, _DELETE_OBJ)?;
        let mut b = self.ks.batch();
        // Closure rebuild set: everything inside obj (if it was a group), plus obj itself.
        let affected: HashSet<u64> = self.affected_by(obj)?.into_iter().filter(|&x| x != obj).collect();
        // Role definitions.
        for kv in self.objects.prefix(obj.to_be_bytes()) {
            b.remove(&self.objects, kv.map_err(err)?.0);
        }
        // Grants on obj.
        for kv in self.subjects_rev.prefix(obj.to_be_bytes()) {
            let (k, _) = kv.map_err(err)?;
            let (sub, role) = (u64_at(&k, 1), u64_at(&k, 2));
            self.del_grant(&mut b, sub, obj, role);
        }
        // Grants held by obj elsewhere (including its own memberships).
        for kv in self.subjects.prefix(obj.to_be_bytes()) {
            let (k, _) = kv.map_err(err)?;
            let (o, role) = (u64_at(&k, 1), u64_at(&k, 2));
            self.del_grant(&mut b, obj, o, role);
        }
        // Delegations in both roles.
        for kv in self.delegations_rev.prefix(obj.to_be_bytes()) {
            let (k, _) = kv.map_err(err)?;
            b.remove(&self.delegations, key3(u64_at(&k, 1), obj, u64_at(&k, 2)));
            b.remove(&self.delegations_rev, k);
        }
        for kv in self.delegations.prefix(obj.to_be_bytes()) {
            let (k, _) = kv.map_err(err)?;
            b.remove(&self.delegations_rev, key3(u64_at(&k, 1), obj, u64_at(&k, 2)));
            b.remove(&self.delegations, k);
        }
        // Parent links: obj's own parent, and orphan obj's children.
        if let Some(p) = self.get_u64(&self.parents, &obj.to_be_bytes())? {
            b.remove(&self.parents, obj.to_be_bytes());
            b.remove(&self.parents_rev, key(p, obj));
        }
        for kv in self.parents_rev.prefix(obj.to_be_bytes()) {
            let (k, _) = kv.map_err(err)?;
            b.remove(&self.parents, u64_at(&k, 1).to_be_bytes());
            b.remove(&self.parents_rev, k);
        }
        // Obj's own closure rows.
        for kv in self.closure.prefix(obj.to_be_bytes()) {
            let (k, _) = kv.map_err(err)?;
            let other = u64_at(&k, 1);
            b.remove(&self.closure, k);
            b.remove(&self.closure_rev, key(other, obj));
        }
        for kv in self.closure_rev.prefix(obj.to_be_bytes()) {
            let (k, _) = kv.map_err(err)?;
            let other = u64_at(&k, 1);
            b.remove(&self.closure_rev, k);
            b.remove(&self.closure, key(other, obj));
        }
        // Rebuild closures of former members with all edges into obj dropped.
        self.rebuild_closures(&mut b, &affected, &[EdgeOp::DropGroup(obj)])?;
        self.audit_row(&mut b, &mut seq, actor, op::DELETE_OBJECT, [obj, 0, 0, 0]);
        self.commit(b)
    }

    // -- role definitions -------------------------------------------------------

    pub fn define_role(&self, actor: u64, obj: u64, role: u64, mask: u64) -> Result<()> {
        let mut seq = self.write.lock().map_err(err)?;
        self.auth(actor, obj, _DEFINE)?;
        if self.objects.get(key(obj, role)).map_err(err)?.is_some() {
            return Err(Error::Exists);
        }
        let mut b = self.ks.batch();
        b.insert(&self.objects, key(obj, role), mask.to_be_bytes());
        self.audit_row(&mut b, &mut seq, actor, op::DEFINE_ROLE, [obj, role, mask, 0]);
        self.commit(b)
    }

    pub fn update_role(&self, actor: u64, obj: u64, role: u64, mask: u64) -> Result<()> {
        let mut seq = self.write.lock().map_err(err)?;
        self.auth(actor, obj, _DEFINE)?;
        if self.objects.get(key(obj, role)).map_err(err)?.is_none() {
            return Err(Error::NotFound);
        }
        let mut b = self.ks.batch();
        b.insert(&self.objects, key(obj, role), mask.to_be_bytes());
        self.audit_row(&mut b, &mut seq, actor, op::UPDATE_ROLE, [obj, role, mask, 0]);
        self.commit(b)
    }

    /// Delete a role definition and cascade every grant and delegation that
    /// references it — no dangling tuples, no resurrection-by-redefinition.
    pub fn delete_role(&self, actor: u64, obj: u64, role: u64) -> Result<()> {
        let mut seq = self.write.lock().map_err(err)?;
        self.auth(actor, obj, _DEFINE)?;
        let mut b = self.ks.batch();
        b.remove(&self.objects, key(obj, role));
        let mut member_cascade = false;
        for kv in self.subjects_rev.prefix(obj.to_be_bytes()) {
            let (k, _) = kv.map_err(err)?;
            if u64_at(&k, 2) == role {
                self.del_grant(&mut b, u64_at(&k, 1), obj, role);
                member_cascade |= role == _MEMBER;
            }
        }
        for kv in self.delegations_rev.prefix(obj.to_be_bytes()) {
            let (k, _) = kv.map_err(err)?;
            if u64_at(&k, 2) == role {
                b.remove(&self.delegations, key3(u64_at(&k, 1), obj, role));
                b.remove(&self.delegations_rev, k);
            }
        }
        if member_cascade {
            let affected: HashSet<u64> = self.affected_by(obj)?.into_iter().filter(|&x| x != obj).collect();
            self.rebuild_closures(&mut b, &affected, &[EdgeOp::DropGroup(obj)])?;
        }
        self.audit_row(&mut b, &mut seq, actor, op::DELETE_ROLE, [obj, role, 0, 0]);
        self.commit(b)
    }

    pub fn get_role(&self, actor: u64, obj: u64, role: u64) -> Result<Option<u64>> {
        self.auth(actor, obj, _READ_META)?;
        self.get_u64(&self.objects, &key(obj, role))
    }

    pub fn list_roles(&self, actor: u64, obj: u64) -> Result<Vec<(u64, u64)>> {
        self.auth(actor, obj, _READ_META)?;
        let mut out = Vec::new();
        for kv in self.objects.prefix(obj.to_be_bytes()) {
            let (k, v) = kv.map_err(err)?;
            out.push((u64_at(&k, 1), val(&v)));
        }
        Ok(out)
    }

    // -- grants (and membership: a membership IS a grant of `_MEMBER`) -----------

    /// Grant `role` on `obj` to `sub` at `policy` strength. Requires `_GRANT`.
    /// `Policy::Not` is an explicit deny of the role's bits. The role must be
    /// defined on the object (or its type chain), except `_MEMBER` which needs
    /// no mask. Granting `_MEMBER` makes `sub` a member of group `obj` and
    /// rebuilds the affected closure rows atomically.
    pub fn grant(&self, actor: u64, sub: u64, obj: u64, role: u64, policy: Policy) -> Result<()> {
        let mut seq = self.write.lock().map_err(err)?;
        self.auth(actor, obj, _GRANT)?;
        if role != _MEMBER && !self.role_defined(obj, role)? {
            return Err(Error::NotFound);
        }
        if role == _MEMBER {
            if sub == obj {
                return Err(Error::Cycle);
            }
            // Cycle iff obj already (positively) reaches sub.
            if let Some(p) = self.get_u64(&self.closure, &key(obj, sub))? {
                if Policy::from_u64(p) != Policy::Not {
                    return Err(Error::Cycle);
                }
            }
        }
        let mut b = self.ks.batch();
        self.put_grant(&mut b, sub, obj, role, policy);
        if role == _MEMBER {
            let ov = [EdgeOp::Set(sub, obj, policy)];
            let mut affected = self.affected_by(sub)?;
            affected.insert(sub);
            self.rebuild_closures(&mut b, &affected, &ov)?;
        }
        self.audit_row(&mut b, &mut seq, actor, op::GRANT, [sub, obj, role, policy as u64]);
        self.commit(b)
    }

    /// Revoke a role grant. Requires `_REVOKE`. Membership revocations rebuild
    /// the affected closure rows atomically.
    pub fn revoke(&self, actor: u64, sub: u64, obj: u64, role: u64) -> Result<()> {
        let mut seq = self.write.lock().map_err(err)?;
        self.auth(actor, obj, _REVOKE)?;
        let mut b = self.ks.batch();
        self.del_grant(&mut b, sub, obj, role);
        if role == _MEMBER {
            let ov = [EdgeOp::Remove(sub, obj)];
            let mut affected = self.affected_by(sub)?;
            affected.insert(sub);
            self.rebuild_closures(&mut b, &affected, &ov)?;
        }
        self.audit_row(&mut b, &mut seq, actor, op::REVOKE, [sub, obj, role, 0]);
        self.commit(b)
    }

    /// Convenience: membership is a grant of `_MEMBER` on the group object.
    pub fn add_member(&self, actor: u64, member: u64, group: u64, policy: Policy) -> Result<()> {
        self.grant(actor, member, group, _MEMBER, policy)
    }

    pub fn remove_member(&self, actor: u64, member: u64, group: u64) -> Result<()> {
        self.revoke(actor, member, group, _MEMBER)
    }

    // -- delegation ---------------------------------------------------------------

    /// Delegate `role` on `obj` from `parent` to `sub` at `policy` strength.
    /// Resolution follows the chain until an ancestor actually holds the role;
    /// strength composes as min over all links — delegation can only attenuate.
    pub fn delegate(&self, actor: u64, sub: u64, obj: u64, role: u64, parent: u64, policy: Policy) -> Result<()> {
        let mut seq = self.write.lock().map_err(err)?;
        self.auth(actor, obj, _DELEGATE)?;
        if sub == parent {
            return Err(Error::Cycle);
        }
        if role != _MEMBER && !self.role_defined(obj, role)? {
            return Err(Error::NotFound);
        }
        // Reject chains that loop back to sub.
        let mut cur = parent;
        for _ in 0..MAX_DELEGATION_HOPS {
            match self.delegations.get(key3(cur, obj, role)).map_err(err)? {
                Some(v) => {
                    cur = u64_at(&v, 0);
                    if cur == sub {
                        return Err(Error::Cycle);
                    }
                }
                None => break,
            }
        }
        let mut v = [0u8; 16];
        v[..8].copy_from_slice(&parent.to_be_bytes());
        v[8..].copy_from_slice(&(policy as u64).to_be_bytes());
        let mut b = self.ks.batch();
        b.insert(&self.delegations, key3(sub, obj, role), v);
        b.insert(&self.delegations_rev, key3(obj, sub, role), v);
        self.audit_row(&mut b, &mut seq, actor, op::DELEGATE, [sub, obj, role, parent]);
        self.commit(b)
    }

    pub fn undelegate(&self, actor: u64, sub: u64, obj: u64, role: u64) -> Result<()> {
        let mut seq = self.write.lock().map_err(err)?;
        self.auth(actor, obj, _DELEGATE)?;
        let mut b = self.ks.batch();
        b.remove(&self.delegations, key3(sub, obj, role));
        b.remove(&self.delegations_rev, key3(obj, sub, role));
        self.audit_row(&mut b, &mut seq, actor, op::UNDELEGATE, [sub, obj, role, 0]);
        self.commit(b)
    }

    pub fn get_delegation(&self, actor: u64, sub: u64, obj: u64, role: u64) -> Result<Option<(u64, Policy)>> {
        self.auth(actor, obj, _READ_META)?;
        Ok(self.delegations.get(key3(sub, obj, role)).map_err(err)?
            .map(|v| (u64_at(&v, 0), Policy::from_u64(u64_at(&v, 1)))))
    }

    /// All delegation edges on an object: (subject, role, parent, policy).
    pub fn list_delegations_on(&self, actor: u64, obj: u64) -> Result<Vec<(u64, u64, u64, Policy)>> {
        self.auth(actor, obj, _READ_META)?;
        let mut out = Vec::new();
        for kv in self.delegations_rev.prefix(obj.to_be_bytes()) {
            let (k, v) = kv.map_err(err)?;
            out.push((u64_at(&k, 1), u64_at(&k, 2), u64_at(&v, 0), Policy::from_u64(u64_at(&v, 1))));
        }
        Ok(out)
    }

    // -- type-as-object -------------------------------------------------------------

    /// Set the type-parent of an object. Role lookups fall back through the
    /// parent chain, so one role change on a type object repolicies every instance.
    pub fn set_parent(&self, actor: u64, obj: u64, parent: u64) -> Result<()> {
        let mut seq = self.write.lock().map_err(err)?;
        self.auth(actor, obj, _SET_PARENT)?;
        if obj == parent {
            return Err(Error::Cycle);
        }
        let mut cur = parent;
        for _ in 0..MAX_TYPE_DEPTH {
            match self.get_u64(&self.parents, &cur.to_be_bytes())? {
                Some(p) if p == obj => return Err(Error::Cycle),
                Some(p) => cur = p,
                None => break,
            }
        }
        let mut b = self.ks.batch();
        if let Some(old) = self.get_u64(&self.parents, &obj.to_be_bytes())? {
            b.remove(&self.parents_rev, key(old, obj));
        }
        b.insert(&self.parents, obj.to_be_bytes(), parent.to_be_bytes());
        b.insert(&self.parents_rev, key(parent, obj), 1u64.to_be_bytes());
        self.audit_row(&mut b, &mut seq, actor, op::SET_PARENT, [obj, parent, 0, 0]);
        self.commit(b)
    }

    pub fn clear_parent(&self, actor: u64, obj: u64) -> Result<()> {
        let mut seq = self.write.lock().map_err(err)?;
        self.auth(actor, obj, _SET_PARENT)?;
        let mut b = self.ks.batch();
        if let Some(old) = self.get_u64(&self.parents, &obj.to_be_bytes())? {
            b.remove(&self.parents_rev, key(old, obj));
        }
        b.remove(&self.parents, obj.to_be_bytes());
        self.audit_row(&mut b, &mut seq, actor, op::CLEAR_PARENT, [obj, 0, 0, 0]);
        self.commit(b)
    }

    pub fn get_parent(&self, actor: u64, obj: u64) -> Result<Option<u64>> {
        self.auth(actor, obj, _READ_META)?;
        self.get_u64(&self.parents, &obj.to_be_bytes())
    }

    // -- list & audit queries ----------------------------------------------------------

    /// Roles `sub` holds directly on `obj`. Allowed for `sub` itself or `_READ_META`.
    pub fn list_roles_for(&self, actor: u64, sub: u64, obj: u64) -> Result<Vec<(u64, Policy)>> {
        if actor != sub {
            self.auth(actor, obj, _READ_META)?;
        }
        self.grants_on(sub, obj)
    }

    /// Every direct grant held by `sub`: (object, role, policy).
    pub fn list_grants(&self, actor: u64, sub: u64) -> Result<Vec<(u64, u64, Policy)>> {
        if actor != sub {
            self.auth(actor, _SYSTEM, _READ_META)?;
        }
        let mut out = Vec::new();
        for kv in self.subjects.prefix(sub.to_be_bytes()) {
            let (k, v) = kv.map_err(err)?;
            out.push((u64_at(&k, 1), u64_at(&k, 2), Policy::from_u64(val(&v))));
        }
        Ok(out)
    }

    /// Direct holders on `obj`, paginated by subject id: (subject, role, policy).
    /// This is the *compressed* answer — groups appear as single rows; expand
    /// members on demand via `members_of`.
    pub fn list_subjects(&self, actor: u64, obj: u64, after: Option<u64>, limit: usize) -> Result<Vec<(u64, u64, Policy)>> {
        self.auth(actor, obj, _READ_META)?;
        let start = match after {
            Some(a) => match a.checked_add(1) {
                Some(n) => key3(obj, n, 0),
                None => return Ok(Vec::new()),
            },
            None => key3(obj, 0, 0),
        };
        let mut out = Vec::new();
        for kv in self.subjects_rev.range(start.to_vec()..) {
            let (k, v) = kv.map_err(err)?;
            if u64_at(&k, 0) != obj || out.len() >= limit {
                break;
            }
            out.push((u64_at(&k, 1), u64_at(&k, 2), Policy::from_u64(val(&v))));
        }
        Ok(out)
    }

    /// Effective groups of an entity (from the materialized closure).
    pub fn groups_of(&self, actor: u64, ent: u64) -> Result<Vec<(u64, Policy)>> {
        if actor != ent {
            self.auth(actor, _SYSTEM, _READ_META)?;
        }
        let mut out = Vec::new();
        for kv in self.closure.prefix(ent.to_be_bytes()) {
            let (k, v) = kv.map_err(err)?;
            out.push((u64_at(&k, 1), Policy::from_u64(val(&v))));
        }
        Ok(out)
    }

    /// Effective (transitive) members of a group, paginated by member id.
    pub fn members_of(&self, actor: u64, group: u64, after: Option<u64>, limit: usize) -> Result<Vec<(u64, Policy)>> {
        self.auth(actor, group, _READ_META)?;
        let start = match after {
            Some(a) => match a.checked_add(1) {
                Some(n) => key(group, n),
                None => return Ok(Vec::new()),
            },
            None => key(group, 0),
        };
        let mut out = Vec::new();
        for kv in self.closure_rev.range(start.to_vec()..) {
            let (k, v) = kv.map_err(err)?;
            if u64_at(&k, 0) != group || out.len() >= limit {
                break;
            }
            out.push((u64_at(&k, 1), Policy::from_u64(val(&v))));
        }
        Ok(out)
    }

    fn positive_members(&self, group: u64) -> impl Iterator<Item = Result<Option<u64>>> + '_ {
        self.closure_rev.prefix(group.to_be_bytes()).map(|kv| {
            let (k, v) = kv.map_err(err)?;
            Ok((Policy::from_u64(val(&v)) != Policy::Not).then(|| u64_at(&k, 1)))
        })
    }

    /// Population algebra: members of both groups (sorted merge, no expansion stored).
    pub fn population_intersect(&self, actor: u64, a: u64, b: u64) -> Result<Vec<u64>> {
        self.auth(actor, a, _READ_META)?;
        self.auth(actor, b, _READ_META)?;
        let (mut ia, mut ib) = (self.positive_members(a), self.positive_members(b));
        let (mut out, mut x, mut y) = (Vec::new(), None, None);
        loop {
            if x.is_none() {
                x = match ia.next() { Some(r) => match r? { Some(v) => Some(v), None => continue }, None => break };
            }
            if y.is_none() {
                y = match ib.next() { Some(r) => match r? { Some(v) => Some(v), None => continue }, None => break };
            }
            let (xv, yv) = (x.unwrap(), y.unwrap());
            if xv == yv {
                out.push(xv);
                x = None;
                y = None;
            } else if xv < yv {
                x = None;
            } else {
                y = None;
            }
        }
        Ok(out)
    }

    /// Population algebra: members of `a` that are not (positively) in `b`.
    pub fn population_subtract(&self, actor: u64, a: u64, b: u64) -> Result<Vec<u64>> {
        self.auth(actor, a, _READ_META)?;
        self.auth(actor, b, _READ_META)?;
        let mut out = Vec::new();
        for r in self.positive_members(a) {
            if let Some(m) = r? {
                let in_b = self.get_u64(&self.closure_rev, &key(b, m))?
                    .is_some_and(|p| Policy::from_u64(p) != Policy::Not);
                if !in_b {
                    out.push(m);
                }
            }
        }
        Ok(out)
    }

    /// Read the audit log from `after` (exclusive). Requires `_AUDIT` on `_SYSTEM`.
    pub fn audit_read(&self, actor: u64, after: Option<u64>, limit: usize) -> Result<Vec<AuditEntry>> {
        self.auth(actor, _SYSTEM, _AUDIT)?;
        let start = after.map_or(0u64, |a| a.saturating_add(1));
        let mut out = Vec::new();
        for kv in self.audit.range(start.to_be_bytes().to_vec()..) {
            let (k, v) = kv.map_err(err)?;
            if out.len() >= limit {
                break;
            }
            out.push(AuditEntry {
                seq: u64_at(&k, 0),
                ts_ms: u64_at(&v, 0),
                actor: u64_at(&v, 1),
                op: u64_at(&v, 2),
                args: [u64_at(&v, 3), u64_at(&v, 4), u64_at(&v, 5), u64_at(&v, 6)],
            });
        }
        Ok(out)
    }

    /// Wipe everything, including the audit log. Requires `_DELETE_OBJ` on `_SYSTEM`.
    /// Dev/test utility — a wiped store must be bootstrapped again.
    pub fn clear(&self, actor: u64) -> Result<()> {
        let mut seq = self.write.lock().map_err(err)?;
        self.auth(actor, _SYSTEM, _DELETE_OBJ)?;
        let mut b = self.ks.batch();
        for p in [&self.objects, &self.parents, &self.parents_rev, &self.subjects, &self.subjects_rev,
                  &self.closure, &self.closure_rev, &self.delegations, &self.delegations_rev, &self.audit] {
            for kv in p.prefix([]) {
                b.remove(p, kv.map_err(err)?.0);
            }
        }
        self.commit(b)?;
        *seq = 0;
        Ok(())
    }
}
