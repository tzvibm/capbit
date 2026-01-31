//! Compact EntityId representation using length-prefixed byte array.
//!
//! Format: [type_len: u8][type_bytes][id_bytes]
//!
//! Benefits over "type:id" string:
//! - O(1) type/id extraction (no colon scanning)
//! - 8 bytes less stack overhead than String
//! - Prefix scans still work for "all entities of type X"
//! - No collision risk (unlike hashing)

use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

/// Maximum length for entity type (255 bytes)
pub const MAX_TYPE_LEN: usize = 255;

/// Error type for EntityId operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityIdError {
    pub message: String,
}

impl fmt::Display for EntityIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for EntityIdError {}

/// Compact entity identifier.
///
/// Internally stores: [type_len: u8][type_bytes][id_bytes]
#[derive(Clone)]
pub struct EntityId {
    data: Box<[u8]>,
}

impl EntityId {
    /// Create from separate type and id.
    ///
    /// # Example
    /// ```
    /// let eid = EntityId::new("user", "alice").unwrap();
    /// assert_eq!(eid.entity_type(), "user");
    /// assert_eq!(eid.id(), "alice");
    /// ```
    pub fn new(entity_type: &str, id: &str) -> Result<Self, EntityIdError> {
        let type_bytes = entity_type.as_bytes();
        let id_bytes = id.as_bytes();

        if type_bytes.is_empty() {
            return Err(EntityIdError {
                message: "Entity type cannot be empty".into(),
            });
        }

        if type_bytes.len() > MAX_TYPE_LEN {
            return Err(EntityIdError {
                message: format!(
                    "Entity type too long: {} bytes (max {})",
                    type_bytes.len(),
                    MAX_TYPE_LEN
                ),
            });
        }

        if id_bytes.is_empty() {
            return Err(EntityIdError {
                message: "Entity id cannot be empty".into(),
            });
        }

        let mut data = Vec::with_capacity(1 + type_bytes.len() + id_bytes.len());
        data.push(type_bytes.len() as u8);
        data.extend_from_slice(type_bytes);
        data.extend_from_slice(id_bytes);

        Ok(Self {
            data: data.into_boxed_slice(),
        })
    }

    /// Parse from "type:id" string format (for migration/compatibility).
    ///
    /// # Example
    /// ```
    /// let eid = EntityId::parse("user:alice").unwrap();
    /// assert_eq!(eid.entity_type(), "user");
    /// assert_eq!(eid.id(), "alice");
    /// ```
    pub fn parse(s: &str) -> Result<Self, EntityIdError> {
        let (entity_type, id) = s.split_once(':').ok_or_else(|| EntityIdError {
            message: format!("Invalid entity ID '{}': must be 'type:id' format", s),
        })?;
        Self::new(entity_type, id)
    }

    /// Create from raw bytes (for deserialization from storage).
    ///
    /// Validates the format is correct.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, EntityIdError> {
        if bytes.is_empty() {
            return Err(EntityIdError {
                message: "Empty byte array".into(),
            });
        }

        let type_len = bytes[0] as usize;

        if type_len == 0 {
            return Err(EntityIdError {
                message: "Entity type length cannot be zero".into(),
            });
        }

        if bytes.len() < 1 + type_len + 1 {
            return Err(EntityIdError {
                message: format!(
                    "Byte array too short: need at least {} bytes, got {}",
                    1 + type_len + 1,
                    bytes.len()
                ),
            });
        }

        // Validate UTF-8 for type
        std::str::from_utf8(&bytes[1..1 + type_len]).map_err(|e| EntityIdError {
            message: format!("Invalid UTF-8 in entity type: {}", e),
        })?;

        // Validate UTF-8 for id
        std::str::from_utf8(&bytes[1 + type_len..]).map_err(|e| EntityIdError {
            message: format!("Invalid UTF-8 in entity id: {}", e),
        })?;

        Ok(Self {
            data: bytes.into(),
        })
    }

    /// Create from raw bytes without validation (unsafe but fast).
    ///
    /// # Safety
    /// Caller must ensure bytes are valid EntityId format with valid UTF-8.
    #[inline]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> Self {
        Self {
            data: bytes.into(),
        }
    }

    /// Get the entity type.
    #[inline]
    pub fn entity_type(&self) -> &str {
        let len = self.data[0] as usize;
        // SAFETY: we validate UTF-8 on construction
        unsafe { std::str::from_utf8_unchecked(&self.data[1..1 + len]) }
    }

    /// Get the entity id (the part after the type).
    #[inline]
    pub fn id(&self) -> &str {
        let len = self.data[0] as usize;
        // SAFETY: we validate UTF-8 on construction
        unsafe { std::str::from_utf8_unchecked(&self.data[1 + len..]) }
    }

    /// Get the raw bytes for storage.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get the type length byte position (always 0).
    #[inline]
    pub fn type_len(&self) -> usize {
        self.data[0] as usize
    }

    /// Get total byte length.
    #[inline]
    pub fn byte_len(&self) -> usize {
        self.data.len()
    }

    /// Check if this is a meta-type entity (_type:*).
    #[inline]
    pub fn is_meta_type(&self) -> bool {
        self.entity_type() == "_type"
    }

    /// Check if entity type starts with underscore (internal types).
    #[inline]
    pub fn is_internal_type(&self) -> bool {
        self.data.get(1) == Some(&b'_')
    }

    /// Get the corresponding meta-type entity for this entity's type.
    ///
    /// E.g., "user:alice" → "_type:user"
    pub fn meta_type(&self) -> Self {
        // SAFETY: "_type" is valid and entity_type() returns valid str
        Self::new("_type", self.entity_type()).unwrap()
    }

    /// Convert to "type:id" string format.
    pub fn to_string_format(&self) -> String {
        format!("{}:{}", self.entity_type(), self.id())
    }

    /// Borrow as Cow<str> in "type:id" format (allocates).
    pub fn as_cow_str(&self) -> Cow<'_, str> {
        Cow::Owned(self.to_string_format())
    }
}

// ============================================================================
// Prefix generation for range scans
// ============================================================================

/// Generate a prefix for scanning all entities of a given type.
///
/// # Example
/// ```
/// let prefix = EntityId::prefix_for_type("user");
/// // Use with LMDB prefix scan to find all user:* entities
/// ```
impl EntityId {
    /// Generate prefix bytes for scanning all entities of a type.
    pub fn prefix_for_type(entity_type: &str) -> Vec<u8> {
        let type_bytes = entity_type.as_bytes();
        let mut prefix = Vec::with_capacity(1 + type_bytes.len());
        prefix.push(type_bytes.len() as u8);
        prefix.extend_from_slice(type_bytes);
        prefix
    }

    /// Check if this entity matches a type prefix.
    #[inline]
    pub fn has_type(&self, entity_type: &str) -> bool {
        self.entity_type() == entity_type
    }

    /// Check if this entity's bytes start with the given prefix.
    #[inline]
    pub fn starts_with(&self, prefix: &[u8]) -> bool {
        self.data.starts_with(prefix)
    }
}

// ============================================================================
// Trait implementations
// ============================================================================

impl PartialEq for EntityId {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for EntityId {}

impl PartialOrd for EntityId {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EntityId {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.data.cmp(&other.data)
    }
}

impl Hash for EntityId {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.entity_type(), self.id())
    }
}

impl fmt::Debug for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntityId")
            .field("type", &self.entity_type())
            .field("id", &self.id())
            .field("bytes", &self.data.len())
            .finish()
    }
}

// ============================================================================
// Conversion traits
// ============================================================================

impl TryFrom<&str> for EntityId {
    type Error = EntityIdError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

impl TryFrom<String> for EntityId {
    type Error = EntityIdError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::parse(&s)
    }
}

impl TryFrom<&[u8]> for EntityId {
    type Error = EntityIdError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::from_bytes(bytes)
    }
}

impl AsRef<[u8]> for EntityId {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

// ============================================================================
// Serde support (optional, behind feature flag if desired)
// ============================================================================

#[cfg(feature = "serde")]
impl serde::Serialize for EntityId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize as "type:id" string for JSON compatibility
        serializer.serialize_str(&self.to_string_format())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for EntityId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_accessors() {
        let eid = EntityId::new("user", "alice").unwrap();
        assert_eq!(eid.entity_type(), "user");
        assert_eq!(eid.id(), "alice");
        assert_eq!(eid.to_string(), "user:alice");
    }

    #[test]
    fn test_parse() {
        let eid = EntityId::parse("resource:doc123").unwrap();
        assert_eq!(eid.entity_type(), "resource");
        assert_eq!(eid.id(), "doc123");
    }

    #[test]
    fn test_parse_invalid() {
        assert!(EntityId::parse("no_colon").is_err());
        assert!(EntityId::parse("").is_err());
        assert!(EntityId::parse(":empty_type").is_err());
        assert!(EntityId::parse("empty_id:").is_err());
    }

    #[test]
    fn test_bytes_roundtrip() {
        let eid1 = EntityId::new("team", "engineering").unwrap();
        let bytes = eid1.as_bytes();
        let eid2 = EntityId::from_bytes(bytes).unwrap();
        assert_eq!(eid1, eid2);
    }

    #[test]
    fn test_byte_format() {
        let eid = EntityId::new("user", "alice").unwrap();
        let bytes = eid.as_bytes();

        // [4, 'u', 's', 'e', 'r', 'a', 'l', 'i', 'c', 'e']
        assert_eq!(bytes[0], 4); // type length
        assert_eq!(&bytes[1..5], b"user");
        assert_eq!(&bytes[5..], b"alice");
    }

    #[test]
    fn test_meta_type() {
        let eid = EntityId::new("user", "alice").unwrap();
        let meta = eid.meta_type();
        assert_eq!(meta.entity_type(), "_type");
        assert_eq!(meta.id(), "user");
        assert_eq!(meta.to_string(), "_type:user");
    }

    #[test]
    fn test_is_meta_type() {
        let meta = EntityId::new("_type", "user").unwrap();
        assert!(meta.is_meta_type());

        let regular = EntityId::new("user", "alice").unwrap();
        assert!(!regular.is_meta_type());
    }

    #[test]
    fn test_is_internal_type() {
        let internal = EntityId::new("_type", "user").unwrap();
        assert!(internal.is_internal_type());

        let regular = EntityId::new("user", "alice").unwrap();
        assert!(!regular.is_internal_type());
    }

    #[test]
    fn test_prefix_for_type() {
        let prefix = EntityId::prefix_for_type("user");
        assert_eq!(prefix, vec![4, b'u', b's', b'e', b'r']);

        let eid = EntityId::new("user", "alice").unwrap();
        assert!(eid.starts_with(&prefix));

        let other = EntityId::new("team", "dev").unwrap();
        assert!(!other.starts_with(&prefix));
    }

    #[test]
    fn test_has_type() {
        let eid = EntityId::new("user", "alice").unwrap();
        assert!(eid.has_type("user"));
        assert!(!eid.has_type("team"));
    }

    #[test]
    fn test_ordering() {
        let a = EntityId::new("user", "alice").unwrap();
        let b = EntityId::new("user", "bob").unwrap();
        let c = EntityId::new("team", "dev").unwrap();

        assert!(a < b); // same type, alice < bob
        assert!(c < a); // team (4 bytes) vs user (4 bytes), 't' < 'u'
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(EntityId::new("user", "alice").unwrap());
        set.insert(EntityId::new("user", "bob").unwrap());

        assert!(set.contains(&EntityId::new("user", "alice").unwrap()));
        assert!(!set.contains(&EntityId::new("user", "charlie").unwrap()));
    }

    #[test]
    fn test_empty_type_error() {
        assert!(EntityId::new("", "alice").is_err());
    }

    #[test]
    fn test_empty_id_error() {
        assert!(EntityId::new("user", "").is_err());
    }

    #[test]
    fn test_long_type() {
        let long_type = "a".repeat(255);
        let eid = EntityId::new(&long_type, "id").unwrap();
        assert_eq!(eid.entity_type(), long_type);

        let too_long = "a".repeat(256);
        assert!(EntityId::new(&too_long, "id").is_err());
    }

    #[test]
    fn test_unicode() {
        let eid = EntityId::new("用户", "アリス").unwrap();
        assert_eq!(eid.entity_type(), "用户");
        assert_eq!(eid.id(), "アリス");
        assert_eq!(eid.to_string(), "用户:アリス");
    }

    #[test]
    fn test_try_from_str() {
        let eid: EntityId = "user:alice".try_into().unwrap();
        assert_eq!(eid.entity_type(), "user");
        assert_eq!(eid.id(), "alice");
    }

    #[test]
    fn test_debug_format() {
        let eid = EntityId::new("user", "alice").unwrap();
        let debug = format!("{:?}", eid);
        assert!(debug.contains("user"));
        assert!(debug.contains("alice"));
    }

    #[test]
    fn test_clone() {
        let eid1 = EntityId::new("user", "alice").unwrap();
        let eid2 = eid1.clone();
        assert_eq!(eid1, eid2);
    }

    #[test]
    fn test_memory_size() {
        let eid = EntityId::new("user", "alice").unwrap();

        // Stack size: Box<[u8]> = ptr + len = 16 bytes
        assert_eq!(std::mem::size_of::<EntityId>(), 16);

        // Heap size: 1 + 4 + 5 = 10 bytes
        assert_eq!(eid.byte_len(), 10);
    }
}
