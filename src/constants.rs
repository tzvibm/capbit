//! Permission constants and system IDs

// Capability bit constants
pub const READ: u64 = 1;
pub const WRITE: u64 = 1 << 1;
pub const DELETE: u64 = 1 << 2;
pub const CREATE: u64 = 1 << 3;
pub const GRANT: u64 = 1 << 4;
pub const EXECUTE: u64 = 1 << 5;
pub const VIEW: u64 = 1 << 62;
pub const ADMIN: u64 = 1 << 63;

// System IDs (bootstrap always creates these)
pub const SYSTEM_ID: u64 = 1;
pub const ROOT_USER_ID: u64 = 2;

// Predefined roles on _system for granular inheritance
pub const ROLE_GRANTER: u64 = 1;   // Can grant/revoke on objects
pub const ROLE_ADMIN: u64 = 2;     // Can set_role/set_inherit
pub const ROLE_VIEWER: u64 = 3;    // Can list_for_object
pub const ROLE_FULL: u64 = 4;      // All system permissions

// Maximum inheritance chain depth (prevents infinite loops)
pub const MAX_INHERITANCE_DEPTH: usize = 10;

// Capability name mappings
const CAPS: &[(&str, u64)] = &[
    ("read", READ),
    ("write", WRITE),
    ("delete", DELETE),
    ("create", CREATE),
    ("grant", GRANT),
    ("execute", EXECUTE),
    ("view", VIEW),
    ("admin", ADMIN),
];

/// Convert a capability mask to a list of capability names
pub fn caps_to_names(mask: u64) -> Vec<&'static str> {
    CAPS.iter()
        .filter(|(_, b)| mask & b == *b)
        .map(|(n, _)| *n)
        .collect()
}

/// Convert a list of capability names to a mask
pub fn names_to_caps(names: &[&str]) -> u64 {
    names
        .iter()
        .filter_map(|n| CAPS.iter().find(|(k, _)| k == n).map(|(_, v)| v))
        .fold(0, |a, b| a | b)
}
