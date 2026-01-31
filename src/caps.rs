//! System capability constants for Capbit v2

/// System capabilities as bitmask constants
#[allow(non_snake_case)]
pub mod SystemCap {
    // Type management (on _type:_type scope)
    pub const TYPE_CREATE: u64 = 0x0001;
    pub const TYPE_DELETE: u64 = 0x0002;

    // Entity management (on _type:{type} scopes)
    pub const ENTITY_CREATE: u64 = 0x0004;
    pub const ENTITY_DELETE: u64 = 0x0008;

    // Grants (relations between entities)
    pub const GRANT_READ: u64 = 0x0010;
    pub const GRANT_WRITE: u64 = 0x0020;
    pub const GRANT_DELETE: u64 = 0x0040;

    // Capability definitions
    pub const CAP_READ: u64 = 0x0080;
    pub const CAP_WRITE: u64 = 0x0100;
    pub const CAP_DELETE: u64 = 0x0200;

    // Delegations (inheritance)
    pub const DELEGATE_READ: u64 = 0x0400;
    pub const DELEGATE_WRITE: u64 = 0x0800;
    pub const DELEGATE_DELETE: u64 = 0x1000;

    // System visibility (can view _type:* entities, grants, caps)
    pub const SYSTEM_READ: u64 = 0x2000;

    // Password/credential management (on _type:{type} scopes)
    pub const PASSWORD_ADMIN: u64 = 0x4000;

    // Composites (low-level)
    pub const GRANT_ADMIN: u64 = GRANT_READ | GRANT_WRITE | GRANT_DELETE;
    pub const CAP_ADMIN: u64 = CAP_READ | CAP_WRITE | CAP_DELETE;
    pub const DELEGATE_ADMIN: u64 = DELEGATE_READ | DELEGATE_WRITE | DELEGATE_DELETE;
    pub const READ_ONLY: u64 = GRANT_READ | CAP_READ | DELEGATE_READ;

    // Composites (high-level admin roles)
    // ENTITY_ADMIN: full control over entities of a type (create, delete, grant, caps, delegate)
    pub const ENTITY_ADMIN: u64 = ENTITY_CREATE | ENTITY_DELETE | CAP_ADMIN | GRANT_ADMIN | DELEGATE_ADMIN;
    // TYPE_ADMIN: full control over types (create, delete types) plus entity admin plus system visibility plus password admin
    pub const TYPE_ADMIN: u64 = TYPE_CREATE | TYPE_DELETE | ENTITY_ADMIN | SYSTEM_READ | PASSWORD_ADMIN;
    // ALL: every capability bit
    pub const ALL: u64 = 0x7FFF;
}
