//! Interleaved key encoding for normalized LMDB storage.
//!
//! Keys use interleaved bits for dual O(log n) lookup via thread-local mode switching.
//! - Even bits: first dimension (entity_id, seeker_id)
//! - Odd bits: second dimension (name_hash, scope_id)

use std::cell::Cell;
use std::cmp::Ordering;

// ============================================================================
// Thread-Local Compare Mode
// ============================================================================

thread_local! {
    /// false = compare even bits (first dimension: ID/Seeker)
    /// true  = compare odd bits (second dimension: Name/Scope)
    pub static COMPARE_MODE: Cell<bool> = const { Cell::new(false) };
}

/// Set mode to compare by first dimension (entity_id, seeker_id) - even bits
#[inline]
pub fn set_mode_first() {
    COMPARE_MODE.with(|m| m.set(false));
}

/// Set mode to compare by second dimension (name_hash, scope_id) - odd bits
#[inline]
pub fn set_mode_second() {
    COMPARE_MODE.with(|m| m.set(true));
}

/// Get current compare mode (for debugging)
#[inline]
pub fn get_mode() -> bool {
    COMPARE_MODE.with(|m| m.get())
}

// ============================================================================
// Bit Interleaving
// ============================================================================

/// Interleave two u32s into one u64
/// - a bits go to even positions (0, 2, 4, ...)
/// - b bits go to odd positions (1, 3, 5, ...)
#[inline]
pub fn interleave(a: u32, b: u32) -> u64 {
    let mut result = 0u64;
    for i in 0..32 {
        result |= (((a >> i) & 1) as u64) << (2 * i);      // a bits at even positions
        result |= (((b >> i) & 1) as u64) << (2 * i + 1);  // b bits at odd positions
    }
    result
}

/// De-interleave u64 back to two u32s
/// - even positions → a (first dimension)
/// - odd positions → b (second dimension)
#[inline]
pub fn deinterleave(z: u64) -> (u32, u32) {
    let mut a = 0u32;
    let mut b = 0u32;
    for i in 0..32 {
        a |= (((z >> (2 * i)) & 1) as u32) << i;      // even positions → a
        b |= (((z >> (2 * i + 1)) & 1) as u32) << i;  // odd positions → b
    }
    (a, b)
}

// ============================================================================
// Name Hashing
// ============================================================================

/// Hash a name to u32 for interleaving
/// Uses first 4 bytes in big-endian order to preserve lexicographic prefix ordering
#[inline]
pub fn name_hash(name: &str) -> u32 {
    let bytes = name.as_bytes();
    let mut h = 0u32;
    for (i, &b) in bytes.iter().take(4).enumerate() {
        h |= (b as u32) << (8 * (3 - i));  // big-endian for lex order
    }
    h
}

// ============================================================================
// Key Construction
// ============================================================================

/// Entity key: [type_id:4][interleaved(entity_id, name_hash):8] = 12 bytes
#[inline]
pub fn entity_key(type_id: u32, entity_id: u32, name: &str) -> [u8; 12] {
    let mut key = [0u8; 12];
    key[0..4].copy_from_slice(&type_id.to_be_bytes());
    let z = interleave(entity_id, name_hash(name));
    key[4..12].copy_from_slice(&z.to_be_bytes());
    key
}

/// Grant key: [seeker_type:4][seeker_id:4][scope_type:4][scope_id:4][role_id:4] = 20 bytes
/// Seeker fields first for efficient prefix scan by seeker
#[inline]
pub fn grant_key(seeker_type: u32, seeker_id: u32, scope_type: u32, scope_id: u32, role_id: u32) -> [u8; 20] {
    let mut key = [0u8; 20];
    key[0..4].copy_from_slice(&seeker_type.to_be_bytes());
    key[4..8].copy_from_slice(&seeker_id.to_be_bytes());
    key[8..12].copy_from_slice(&scope_type.to_be_bytes());
    key[12..16].copy_from_slice(&scope_id.to_be_bytes());
    key[16..20].copy_from_slice(&role_id.to_be_bytes());
    key
}

/// Grant seeker prefix: [seeker_type:4][seeker_id:4] = 8 bytes
/// For prefix scanning all grants from a specific seeker
#[inline]
pub fn grant_seeker_prefix(seeker_type: u32, seeker_id: u32) -> [u8; 8] {
    let mut prefix = [0u8; 8];
    prefix[0..4].copy_from_slice(&seeker_type.to_be_bytes());
    prefix[4..8].copy_from_slice(&seeker_id.to_be_bytes());
    prefix
}

/// Grant seeker+scope prefix: [seeker_type:4][seeker_id:4][scope_type:4][scope_id:4] = 16 bytes
/// For prefix scanning grants from a specific seeker to a specific scope
#[inline]
pub fn grant_seeker_scope_prefix(seeker_type: u32, seeker_id: u32, scope_type: u32, scope_id: u32) -> [u8; 16] {
    let mut prefix = [0u8; 16];
    prefix[0..4].copy_from_slice(&seeker_type.to_be_bytes());
    prefix[4..8].copy_from_slice(&seeker_id.to_be_bytes());
    prefix[8..12].copy_from_slice(&scope_type.to_be_bytes());
    prefix[12..16].copy_from_slice(&scope_id.to_be_bytes());
    prefix
}

/// Capability key: [scope_type:4][scope_id:4][role_id:4] = 12 bytes
#[inline]
pub fn capability_key(scope_type: u32, scope_id: u32, role_id: u32) -> [u8; 12] {
    let mut key = [0u8; 12];
    key[0..4].copy_from_slice(&scope_type.to_be_bytes());
    key[4..8].copy_from_slice(&scope_id.to_be_bytes());
    key[8..12].copy_from_slice(&role_id.to_be_bytes());
    key
}

/// Inheritance/delegation key: [seeker_type:4][seeker_id:4][scope_type:4][scope_id:4][source_type:4][source_id:4] = 24 bytes
/// Seeker fields first for efficient prefix scan by seeker
#[inline]
pub fn inheritance_key(seeker_type: u32, seeker_id: u32, scope_type: u32, scope_id: u32, source_type: u32, source_id: u32) -> [u8; 24] {
    let mut key = [0u8; 24];
    key[0..4].copy_from_slice(&seeker_type.to_be_bytes());
    key[4..8].copy_from_slice(&seeker_id.to_be_bytes());
    key[8..12].copy_from_slice(&scope_type.to_be_bytes());
    key[12..16].copy_from_slice(&scope_id.to_be_bytes());
    key[16..20].copy_from_slice(&source_type.to_be_bytes());
    key[20..24].copy_from_slice(&source_id.to_be_bytes());
    key
}

/// Inheritance seeker prefix: [seeker_type:4][seeker_id:4] = 8 bytes
/// For prefix scanning all inheritances for a specific seeker
#[inline]
pub fn inheritance_seeker_prefix(seeker_type: u32, seeker_id: u32) -> [u8; 8] {
    let mut prefix = [0u8; 8];
    prefix[0..4].copy_from_slice(&seeker_type.to_be_bytes());
    prefix[4..8].copy_from_slice(&seeker_id.to_be_bytes());
    prefix
}

/// Inheritance seeker+scope prefix: [seeker_type:4][seeker_id:4][scope_type:4][scope_id:4] = 16 bytes
/// For prefix scanning inheritances for a specific seeker on a specific scope
#[inline]
pub fn inheritance_seeker_scope_prefix(seeker_type: u32, seeker_id: u32, scope_type: u32, scope_id: u32) -> [u8; 16] {
    let mut prefix = [0u8; 16];
    prefix[0..4].copy_from_slice(&seeker_type.to_be_bytes());
    prefix[4..8].copy_from_slice(&seeker_id.to_be_bytes());
    prefix[8..12].copy_from_slice(&scope_type.to_be_bytes());
    prefix[12..16].copy_from_slice(&scope_id.to_be_bytes());
    prefix
}

/// Cap label key: [scope_type:4][scope_id:4][bit:4] = 12 bytes
#[inline]
pub fn cap_label_key(scope_type: u32, scope_id: u32, bit: u32) -> [u8; 12] {
    let mut key = [0u8; 12];
    key[0..4].copy_from_slice(&scope_type.to_be_bytes());
    key[4..8].copy_from_slice(&scope_id.to_be_bytes());
    key[8..12].copy_from_slice(&bit.to_be_bytes());
    key
}

// ============================================================================
// Key Parsing
// ============================================================================

/// Parse entity key back to (type_id, entity_id, name_hash)
#[inline]
pub fn parse_entity_key(key: &[u8]) -> Option<(u32, u32, u32)> {
    if key.len() < 12 { return None; }
    let type_id = u32::from_be_bytes(key[0..4].try_into().ok()?);
    let z = u64::from_be_bytes(key[4..12].try_into().ok()?);
    let (entity_id, name_hash) = deinterleave(z);
    Some((type_id, entity_id, name_hash))
}

/// Parse grant key back to (seeker_type, seeker_id, scope_type, scope_id, role_id)
#[inline]
pub fn parse_grant_key(key: &[u8]) -> Option<(u32, u32, u32, u32, u32)> {
    if key.len() < 20 { return None; }
    let seeker_type = u32::from_be_bytes(key[0..4].try_into().ok()?);
    let seeker_id = u32::from_be_bytes(key[4..8].try_into().ok()?);
    let scope_type = u32::from_be_bytes(key[8..12].try_into().ok()?);
    let scope_id = u32::from_be_bytes(key[12..16].try_into().ok()?);
    let role_id = u32::from_be_bytes(key[16..20].try_into().ok()?);
    Some((seeker_type, seeker_id, scope_type, scope_id, role_id))
}

/// Parse capability key back to (scope_type, scope_id, role_id)
#[inline]
pub fn parse_capability_key(key: &[u8]) -> Option<(u32, u32, u32)> {
    if key.len() < 12 { return None; }
    let scope_type = u32::from_be_bytes(key[0..4].try_into().ok()?);
    let scope_id = u32::from_be_bytes(key[4..8].try_into().ok()?);
    let role_id = u32::from_be_bytes(key[8..12].try_into().ok()?);
    Some((scope_type, scope_id, role_id))
}

/// Parse inheritance key back to (seeker_type, seeker_id, scope_type, scope_id, source_type, source_id)
/// Key format: [seeker_type:4][seeker_id:4][scope_type:4][scope_id:4][source_type:4][source_id:4] = 24 bytes
#[inline]
pub fn parse_inheritance_key(key: &[u8]) -> Option<(u32, u32, u32, u32, u32, u32)> {
    if key.len() < 24 { return None; }
    let seeker_type = u32::from_be_bytes(key[0..4].try_into().ok()?);
    let seeker_id = u32::from_be_bytes(key[4..8].try_into().ok()?);
    let scope_type = u32::from_be_bytes(key[8..12].try_into().ok()?);
    let scope_id = u32::from_be_bytes(key[12..16].try_into().ok()?);
    let source_type = u32::from_be_bytes(key[16..20].try_into().ok()?);
    let source_id = u32::from_be_bytes(key[20..24].try_into().ok()?);
    Some((seeker_type, seeker_id, scope_type, scope_id, source_type, source_id))
}

/// Parse cap label key back to (scope_type, scope_id, bit)
#[inline]
pub fn parse_cap_label_key(key: &[u8]) -> Option<(u32, u32, u32)> {
    if key.len() < 12 { return None; }
    let scope_type = u32::from_be_bytes(key[0..4].try_into().ok()?);
    let scope_id = u32::from_be_bytes(key[4..8].try_into().ok()?);
    let bit = u32::from_be_bytes(key[8..12].try_into().ok()?);
    Some((scope_type, scope_id, bit))
}

// ============================================================================
// Comparators
// ============================================================================

/// Compare two keys by even bits only (first dimension: entity_id, seeker_id)
/// Masks for O(1) selective bit comparison
const EVEN_MASK: u64 = 0x5555_5555_5555_5555; // Binary: 0101...0101 - extracts even-positioned bits
const ODD_MASK: u64 = 0xAAAA_AAAA_AAAA_AAAA;  // Binary: 1010...1010 - extracts odd-positioned bits

/// Compare two keys by even bits only (first dimension: entity_id, seeker_id)
/// O(1) using bitwise mask - just AND + integer compare
#[inline]
fn compare_even_bits(za: u64, zb: u64) -> Ordering {
    (za & EVEN_MASK).cmp(&(zb & EVEN_MASK))
}

/// Compare two keys by odd bits only (second dimension: name_hash, scope_id)
/// O(1) using bitwise mask - just AND + integer compare
#[inline]
fn compare_odd_bits(za: u64, zb: u64) -> Ordering {
    (za & ODD_MASK).cmp(&(zb & ODD_MASK))
}

/// Single comparator for entity database
/// Reads thread-local mode to decide which bits to compare
/// Entity key format: [type_id:4][interleaved:8]
pub fn entity_compare(a: &[u8], b: &[u8]) -> Ordering {
    // First compare type_id prefix
    match a[0..4].cmp(&b[0..4]) {
        Ordering::Equal => {}
        other => return other,
    }

    // Get interleaved portion
    let za = u64::from_be_bytes(a[4..12].try_into().unwrap_or([0; 8]));
    let zb = u64::from_be_bytes(b[4..12].try_into().unwrap_or([0; 8]));

    // Compare based on mode
    if COMPARE_MODE.with(|m| m.get()) {
        compare_odd_bits(za, zb)
    } else {
        compare_even_bits(za, zb)
    }
}

/// Simple comparator for grant database
/// Grant key format: [seeker_type:4][seeker_id:4][scope_type:4][scope_id:4][role_id:4] = 20 bytes
/// Sequential keys enable O(log n) prefix scans on seeker
pub fn grant_compare(a: &[u8], b: &[u8]) -> Ordering {
    a.cmp(b)  // Simple lexicographic - seeker fields first enables prefix scan
}

/// Simple comparator for inheritance database
/// Key format: [seeker_type:4][seeker_id:4][scope_type:4][scope_id:4][source_type:4][source_id:4] = 24 bytes
/// Sequential keys enable O(log n) prefix scans on seeker
pub fn inheritance_compare(a: &[u8], b: &[u8]) -> Ordering {
    a.cmp(b)  // Simple lexicographic - seeker fields first enables prefix scan
}

/// Standard comparator for capability database (no interleaving)
/// Capability key format: [scope_type:4][scope_id:4][role_id:4]
pub fn capability_compare(a: &[u8], b: &[u8]) -> Ordering {
    a.cmp(b)  // Simple lexicographic
}

/// Standard comparator for cap_labels database (no interleaving)
/// Cap label key format: [scope_type:4][scope_id:4][bit:4]
pub fn cap_label_compare(a: &[u8], b: &[u8]) -> Ordering {
    a.cmp(b)  // Simple lexicographic
}

// ============================================================================
// Prefix Construction
// ============================================================================

/// Create entity prefix for type_id scan
#[inline]
pub fn entity_type_prefix(type_id: u32) -> [u8; 4] {
    type_id.to_be_bytes()
}

/// Create grant prefix for seeker scan (mode=first) or scope scan (mode=second)
/// Note: For proper prefix scanning, we need the full interleaved portion
#[inline]
pub fn grant_prefix_for_id(id: u32, is_second: bool) -> [u8; 8] {
    let z = if is_second {
        interleave(0, id)  // scope_id in odd bits
    } else {
        interleave(id, 0)  // seeker_id in even bits
    };
    z.to_be_bytes()
}

/// Create capability prefix for scope
#[inline]
pub fn capability_scope_prefix(scope_type: u32, scope_id: u32) -> [u8; 8] {
    let mut prefix = [0u8; 8];
    prefix[0..4].copy_from_slice(&scope_type.to_be_bytes());
    prefix[4..8].copy_from_slice(&scope_id.to_be_bytes());
    prefix
}

// ============================================================================
// Legacy Compatibility (for gradual migration)
// ============================================================================

/// Build a length-prefixed key from parts (legacy format)
#[inline]
pub fn build_key(parts: &[&str]) -> Vec<u8> {
    let total_len: usize = parts.iter().map(|p| 1 + p.len()).sum();
    let mut key = Vec::with_capacity(total_len);
    for part in parts {
        key.push(part.len() as u8);
        key.extend_from_slice(part.as_bytes());
    }
    key
}

/// Build a prefix for scanning (same as build_key, just clearer intent)
#[inline]
pub fn build_prefix(parts: &[&str]) -> Vec<u8> {
    build_key(parts)
}

/// Parse a length-prefixed key into parts (legacy format)
pub fn parse_key(bytes: &[u8]) -> Vec<&str> {
    let mut parts = Vec::with_capacity(4);
    let mut i = 0;
    while i < bytes.len() {
        let len = bytes[i] as usize;
        if i + 1 + len > bytes.len() {
            break;
        }
        let part = unsafe { std::str::from_utf8_unchecked(&bytes[i + 1..i + 1 + len]) };
        parts.push(part);
        i += 1 + len;
    }
    parts
}

/// Get the Nth part from a key without allocating (legacy format)
#[inline]
pub fn get_part(bytes: &[u8], n: usize) -> Option<&str> {
    let mut i = 0;
    let mut count = 0;
    while i < bytes.len() {
        let len = bytes[i] as usize;
        if i + 1 + len > bytes.len() {
            return None;
        }
        if count == n {
            return Some(unsafe { std::str::from_utf8_unchecked(&bytes[i + 1..i + 1 + len]) });
        }
        i += 1 + len;
        count += 1;
    }
    None
}

/// Build entity key (legacy format)
#[inline]
pub fn entity_key_legacy(entity_type: &str, id: &str) -> Vec<u8> {
    build_key(&[entity_type, id])
}

/// Convert "type:id" string to entity key bytes (legacy format)
pub fn entity_from_str(s: &str) -> Option<Vec<u8>> {
    let (t, id) = s.split_once(':')?;
    Some(entity_key_legacy(t, id))
}

/// Convert entity key bytes to "type:id" string for API display (legacy format)
pub fn entity_to_str(bytes: &[u8]) -> Option<String> {
    let t = get_part(bytes, 0)?;
    let id = get_part(bytes, 1)?;
    Some(format!("{}:{}", t, id))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_interleave_edge_cases() {
        // All zeros
        let z = interleave(0, 0);
        assert_eq!(z, 0);
        assert_eq!(deinterleave(z), (0, 0));

        // All ones
        let z = interleave(u32::MAX, u32::MAX);
        assert_eq!(z, u64::MAX);
        assert_eq!(deinterleave(z), (u32::MAX, u32::MAX));

        // Only a
        let z = interleave(0xAAAAAAAA, 0);
        let (a, b) = deinterleave(z);
        assert_eq!(a, 0xAAAAAAAA);
        assert_eq!(b, 0);

        // Only b
        let z = interleave(0, 0x55555555);
        let (a, b) = deinterleave(z);
        assert_eq!(a, 0);
        assert_eq!(b, 0x55555555);
    }

    #[test]
    fn test_name_hash() {
        // Short names
        assert_eq!(name_hash(""), 0);
        assert_eq!(name_hash("a"), 0x61000000);
        assert_eq!(name_hash("ab"), 0x61620000);
        assert_eq!(name_hash("abc"), 0x61626300);
        assert_eq!(name_hash("abcd"), 0x61626364);

        // Long names (only first 4 chars used)
        assert_eq!(name_hash("abcd"), name_hash("abcdefgh"));

        // Lexicographic ordering preserved
        assert!(name_hash("alice") < name_hash("bob"));
        assert!(name_hash("a") < name_hash("b"));
    }

    #[test]
    fn test_entity_key() {
        let key = entity_key(1, 100, "alice");
        assert_eq!(key.len(), 12);

        let (type_id, entity_id, name_h) = parse_entity_key(&key).unwrap();
        assert_eq!(type_id, 1);
        assert_eq!(entity_id, 100);
        assert_eq!(name_h, name_hash("alice"));
    }

    #[test]
    fn test_grant_key() {
        let key = grant_key(1, 10, 2, 20, 5);
        assert_eq!(key.len(), 20);

        let (seeker_type, seeker_id, scope_type, scope_id, role_id) = parse_grant_key(&key).unwrap();
        assert_eq!(seeker_type, 1);
        assert_eq!(seeker_id, 10);
        assert_eq!(scope_type, 2);
        assert_eq!(scope_id, 20);
        assert_eq!(role_id, 5);
    }

    #[test]
    fn test_thread_local_mode() {
        let k1 = entity_key(1, 100, "alice");
        let k2 = entity_key(1, 100, "bob");
        let k3 = entity_key(1, 200, "alice");

        // Mode = ID (even bits) - same entity_id
        set_mode_first();
        assert_eq!(entity_compare(&k1, &k2), Ordering::Equal);  // same ID, diff name
        assert_ne!(entity_compare(&k1, &k3), Ordering::Equal);  // diff ID

        // Mode = Name (odd bits) - same name_hash
        set_mode_second();
        assert_ne!(entity_compare(&k1, &k2), Ordering::Equal);  // diff name
        assert_eq!(entity_compare(&k1, &k3), Ordering::Equal);  // same name
    }

    #[test]
    fn test_grant_sequential_ordering() {
        // Sequential keys: [seeker_type][seeker_id][scope_type][scope_id][role_id]
        // Lexicographic ordering enables O(log n) prefix scans on seeker
        let k1 = grant_key(1, 10, 2, 20, 1);  // seeker=(1,10), scope=(2,20)
        let k2 = grant_key(1, 10, 2, 30, 1);  // seeker=(1,10), scope=(2,30) - same seeker, diff scope
        let k3 = grant_key(1, 15, 2, 20, 1);  // seeker=(1,15), scope=(2,20) - diff seeker

        // Lexicographic comparison - k1 < k2 (differ at scope_id: 20 < 30)
        assert_eq!(grant_compare(&k1, &k2), Ordering::Less);

        // k1 < k3 (differ at seeker_id: 10 < 15)
        assert_eq!(grant_compare(&k1, &k3), Ordering::Less);

        // k2 > k3 (seeker_id: 10 < 15, so k2 before k3 in seeker order)
        // Actually k2.seeker = (1,10), k3.seeker = (1,15), so k2 < k3
        assert_eq!(grant_compare(&k2, &k3), Ordering::Less);

        // Same seeker prefix means adjacent in B-tree (enables prefix scan)
        // k1 and k2 have same seeker prefix [1, 10]
        let prefix = grant_seeker_prefix(1, 10);
        assert_eq!(&k1[0..8], &prefix[..]);
        assert_eq!(&k2[0..8], &prefix[..]);
    }

    #[test]
    fn test_legacy_build_and_parse() {
        let key = build_key(&["user:alice", "editor", "resource:doc1"]);
        let parts = parse_key(&key);
        assert_eq!(parts, vec!["user:alice", "editor", "resource:doc1"]);
    }

    #[test]
    fn test_legacy_get_part() {
        let key = build_key(&["aaa", "bbb", "ccc"]);
        assert_eq!(get_part(&key, 0), Some("aaa"));
        assert_eq!(get_part(&key, 1), Some("bbb"));
        assert_eq!(get_part(&key, 2), Some("ccc"));
        assert_eq!(get_part(&key, 3), None);
    }

    #[test]
    fn test_legacy_entity_from_str() {
        let key = entity_from_str("user:alice").unwrap();
        assert_eq!(entity_to_str(&key), Some("user:alice".to_string()));
    }
}
