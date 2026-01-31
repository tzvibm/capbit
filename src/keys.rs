//! Length-prefixed key encoding for LMDB storage.
//!
//! All keys are encoded as: [len1][bytes1][len2][bytes2]...
//! - No delimiters, no escaping, any bytes allowed
//! - O(1) parsing per part (just read length byte)
//! - Works for any number of parts

/// Build a length-prefixed key from parts
///
/// # Example
/// ```
/// let key = build_key(&["user:alice", "editor", "resource:doc1"]);
/// // Result: [10]user:alice[6]editor[13]resource:doc1
/// ```
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

/// Parse a length-prefixed key into parts
///
/// # Example
/// ```
/// let parts = parse_key(&key);
/// // ["user:alice", "editor", "resource:doc1"]
/// ```
pub fn parse_key(bytes: &[u8]) -> Vec<&str> {
    let mut parts = Vec::with_capacity(4); // most keys have 2-3 parts
    let mut i = 0;
    while i < bytes.len() {
        let len = bytes[i] as usize;
        if i + 1 + len > bytes.len() {
            break;
        }
        // SAFETY: we only store valid UTF-8
        let part = unsafe { std::str::from_utf8_unchecked(&bytes[i + 1..i + 1 + len]) };
        parts.push(part);
        i += 1 + len;
    }
    parts
}

/// Parse first N parts from a key (avoids allocating full vec)
#[inline]
pub fn parse_first(bytes: &[u8], n: usize) -> Vec<&str> {
    let mut parts = Vec::with_capacity(n);
    let mut i = 0;
    while i < bytes.len() && parts.len() < n {
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

/// Get the Nth part from a key without allocating
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

/// Get offset where part N starts (for slicing suffix)
#[inline]
pub fn part_offset(bytes: &[u8], n: usize) -> Option<usize> {
    let mut i = 0;
    let mut count = 0;
    while i < bytes.len() && count < n {
        let len = bytes[i] as usize;
        if i + 1 + len > bytes.len() {
            return None;
        }
        i += 1 + len;
        count += 1;
    }
    if count == n { Some(i) } else { None }
}

/// Check if key starts with prefix
#[inline]
pub fn starts_with(key: &[u8], prefix: &[u8]) -> bool {
    key.starts_with(prefix)
}

// ============================================================================
// Entity ID helpers (2-part: type, id)
// ============================================================================

/// Build entity key: [type_len][type][id_len][id]
#[inline]
pub fn entity_key(entity_type: &str, id: &str) -> Vec<u8> {
    build_key(&[entity_type, id])
}

/// Parse entity key into (type, id)
#[inline]
pub fn parse_entity(bytes: &[u8]) -> Option<(&str, &str)> {
    let entity_type = get_part(bytes, 0)?;
    let id = get_part(bytes, 1)?;
    Some((entity_type, id))
}

/// Convert "type:id" string to entity key bytes
pub fn entity_from_str(s: &str) -> Option<Vec<u8>> {
    let (t, id) = s.split_once(':')?;
    Some(entity_key(t, id))
}

/// Convert entity key bytes to "type:id" string for API display
pub fn entity_to_str(bytes: &[u8]) -> Option<String> {
    let (t, id) = parse_entity(bytes)?;
    Some(format!("{}:{}", t, id))
}

/// Get entity type from key
#[inline]
pub fn entity_type(bytes: &[u8]) -> Option<&str> {
    get_part(bytes, 0)
}

/// Get entity id from key
#[inline]
pub fn entity_id(bytes: &[u8]) -> Option<&str> {
    get_part(bytes, 1)
}

/// Check if entity type is internal (starts with _)
#[inline]
pub fn is_internal_type(bytes: &[u8]) -> bool {
    bytes.len() > 1 && bytes[1] == b'_'
}

/// Build meta-type key (_type:X) for an entity type
#[inline]
pub fn meta_type_key(entity_type: &str) -> Vec<u8> {
    entity_key("_type", entity_type)
}

/// Build meta-type key from entity key (user:alice -> _type:user)
#[inline]
pub fn meta_type_of(entity_bytes: &[u8]) -> Option<Vec<u8>> {
    let t = entity_type(entity_bytes)?;
    Some(meta_type_key(t))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_and_parse() {
        let key = build_key(&["user:alice", "editor", "resource:doc1"]);
        let parts = parse_key(&key);
        assert_eq!(parts, vec!["user:alice", "editor", "resource:doc1"]);
    }

    #[test]
    fn test_get_part() {
        let key = build_key(&["aaa", "bbb", "ccc"]);
        assert_eq!(get_part(&key, 0), Some("aaa"));
        assert_eq!(get_part(&key, 1), Some("bbb"));
        assert_eq!(get_part(&key, 2), Some("ccc"));
        assert_eq!(get_part(&key, 3), None);
    }

    #[test]
    fn test_prefix() {
        let key = build_key(&["user:alice", "editor", "resource:doc1"]);
        let prefix = build_prefix(&["user:alice"]);
        assert!(starts_with(&key, &prefix));

        let prefix2 = build_prefix(&["user:alice", "editor"]);
        assert!(starts_with(&key, &prefix2));

        let wrong = build_prefix(&["user:bob"]);
        assert!(!starts_with(&key, &wrong));
    }

    #[test]
    fn test_entity() {
        let key = entity_key("user", "alice");
        assert_eq!(entity_type(&key), Some("user"));
        assert_eq!(entity_id(&key), Some("alice"));
        assert_eq!(entity_to_str(&key), Some("user:alice".to_string()));
    }

    #[test]
    fn test_entity_from_str() {
        let key = entity_from_str("user:alice").unwrap();
        let (t, id) = parse_entity(&key).unwrap();
        assert_eq!(t, "user");
        assert_eq!(id, "alice");
    }

    #[test]
    fn test_meta_type() {
        let entity = entity_key("user", "alice");
        let meta = meta_type_of(&entity).unwrap();
        assert_eq!(entity_type(&meta), Some("_type"));
        assert_eq!(entity_id(&meta), Some("user"));
    }

    #[test]
    fn test_internal_type() {
        let internal = entity_key("_type", "user");
        assert!(is_internal_type(&internal));

        let regular = entity_key("user", "alice");
        assert!(!is_internal_type(&regular));
    }

    #[test]
    fn test_special_chars() {
        // Slashes, colons, anything allowed
        let key = build_key(&["user/admin", "edit:write", "resource\\doc"]);
        let parts = parse_key(&key);
        assert_eq!(parts, vec!["user/admin", "edit:write", "resource\\doc"]);
    }

    #[test]
    fn test_empty_parts() {
        let key = build_key(&["", "b", ""]);
        let parts = parse_key(&key);
        assert_eq!(parts, vec!["", "b", ""]);
    }

    #[test]
    fn test_part_offset() {
        let key = build_key(&["aaa", "bb", "c"]);
        // [3]aaa[2]bb[1]c = 9 bytes total
        // 0    4   7   9
        assert_eq!(part_offset(&key, 0), Some(0));
        assert_eq!(part_offset(&key, 1), Some(4));
        assert_eq!(part_offset(&key, 2), Some(7));
        assert_eq!(part_offset(&key, 3), Some(9)); // end
    }
}
