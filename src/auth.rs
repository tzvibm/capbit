//! Authentication module for Capbit
//!
//! Provides token-based session management.

use sha2::{Sha256, Digest};
use crate::core::{CapbitError, Result, with_write_txn_pub, with_read_txn_pub, current_epoch_pub};
use crate::bootstrap;

/// Session info returned by list_sessions
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub entity_id: String,
    pub created_at: u64,
    pub expires_at: u64, // 0 = never
}

/// Result from bootstrap_with_token
#[derive(Debug, Clone)]
pub struct BootstrapResult {
    pub root_entity: String,
    pub token: String,
    pub epoch: u64,
}

/// Generate a cryptographically secure token (32 bytes, base64url encoded)
pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).expect("Failed to generate random bytes");
    base64url_encode(&bytes)
}

/// Hash token with SHA-256 for storage
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Base64url encode without padding
fn base64url_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut result = String::with_capacity((data.len() * 4 + 2) / 3);
    for chunk in data.chunks(3) {
        let n = match chunk.len() {
            3 => ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32),
            2 => ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8),
            1 => (chunk[0] as u32) << 16,
            _ => unreachable!(),
        };
        result.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        result.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { result.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char); }
        if chunk.len() > 2 { result.push(ALPHABET[(n & 0x3F) as usize] as char); }
    }
    result
}

/// Hex encode
mod hex {
    pub fn encode(data: impl AsRef<[u8]>) -> String {
        data.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
    }
}

/// Encode session: reuse keys.rs length-prefix format
fn encode_session(entity_id: &str, created_at: u64, expires_at: u64) -> String {
    let key = crate::keys::build_key(&[entity_id, &created_at.to_string(), &expires_at.to_string()]);
    unsafe { String::from_utf8_unchecked(key) }
}

/// Decode session
fn decode_session(data: &str) -> Option<(String, u64, u64)> {
    let parts = crate::keys::parse_key(data.as_bytes());
    if parts.len() != 3 { return None; }
    Some((parts[0].to_string(), parts[1].parse().ok()?, parts[2].parse().ok()?))
}

/// Build session index key
fn session_idx_key(entity_id: &str, hash: &str) -> String {
    let key = crate::keys::build_key(&[entity_id, hash]);
    unsafe { String::from_utf8_unchecked(key) }
}

/// Create a session, returns token
pub fn create_session(entity_id: &str, ttl_secs: Option<u64>) -> Result<String> {
    let token = generate_token();
    let hash = hash_token(&token);
    let now = current_epoch_pub();
    let expires = ttl_secs.map(|t| now + t * 1000).unwrap_or(0);

    with_write_txn_pub(|txn, dbs| {
        // Store: hash → length-prefixed session data
        let value = encode_session(entity_id, now, expires);
        dbs.sessions.put(txn, &hash, &value)
            .map_err(|e| CapbitError { message: e.to_string() })?;

        // Index: length-prefixed key
        let idx_key = session_idx_key(entity_id, &hash);
        dbs.sessions_by_entity.put(txn, &idx_key, &expires.to_string())
            .map_err(|e| CapbitError { message: e.to_string() })?;

        Ok(())
    })?;

    Ok(token)
}

/// Validate token, returns entity_id if valid
pub fn validate_session(token: &str) -> Result<String> {
    let hash = hash_token(token);

    with_read_txn_pub(|txn, dbs| {
        let value = dbs.sessions.get(txn, &hash)
            .map_err(|e| CapbitError { message: e.to_string() })?
            .ok_or_else(|| CapbitError { message: "Invalid token".into() })?;

        let (entity_id, _, expires) = decode_session(value)
            .ok_or_else(|| CapbitError { message: "Corrupted session".into() })?;

        // Check expiry (0 = never expires)
        if expires > 0 && expires < current_epoch_pub() {
            return Err(CapbitError { message: "Token expired".into() });
        }

        Ok(entity_id)
    })
}

/// Revoke a session by token
pub fn revoke_session(token: &str) -> Result<bool> {
    let hash = hash_token(token);

    with_write_txn_pub(|txn, dbs| {
        // Get entity_id first for index cleanup
        let value = match dbs.sessions.get(txn, &hash).map_err(|e| CapbitError { message: e.to_string() })? {
            Some(v) => v.to_string(),
            None => return Ok(false),
        };

        let (entity_id, _, _) = decode_session(&value).unwrap_or_default();

        // Delete main entry
        dbs.sessions.delete(txn, &hash).map_err(|e| CapbitError { message: e.to_string() })?;

        // Delete index
        let idx_key = session_idx_key(&entity_id, &hash);
        dbs.sessions_by_entity.delete(txn, &idx_key).map_err(|e| CapbitError { message: e.to_string() })?;

        Ok(true)
    })
}

/// List all sessions for an entity
pub fn list_sessions(entity_id: &str) -> Result<Vec<SessionInfo>> {
    let prefix = crate::keys::build_prefix(&[entity_id]);
    let now = current_epoch_pub();

    with_read_txn_pub(|txn, dbs| {
        let mut results = Vec::new();
        let prefix_str = unsafe { std::str::from_utf8_unchecked(&prefix) };

        for item in dbs.sessions_by_entity.prefix_iter(txn, prefix_str)
            .map_err(|e| CapbitError { message: e.to_string() })?
        {
            let (key, _) = item.map_err(|e| CapbitError { message: e.to_string() })?;
            let parts = crate::keys::parse_key(key.as_bytes());
            if parts.len() != 2 { continue; }
            let hash = parts[1];

            if let Some(value) = dbs.sessions.get(txn, hash).map_err(|e| CapbitError { message: e.to_string() })? {
                if let Some((eid, created, expires)) = decode_session(value) {
                    // Skip expired (unless never expires)
                    if expires == 0 || expires >= now {
                        results.push(SessionInfo {
                            entity_id: eid,
                            created_at: created,
                            expires_at: expires,
                        });
                    }
                }
            }
        }

        Ok(results)
    })
}

/// Revoke all sessions for an entity
pub fn revoke_all_sessions(entity_id: &str) -> Result<u64> {
    let prefix = crate::keys::build_prefix(&[entity_id]);

    with_write_txn_pub(|txn, dbs| {
        let mut hashes = Vec::new();
        let prefix_str = unsafe { std::str::from_utf8_unchecked(&prefix) };

        // Collect hashes first
        for item in dbs.sessions_by_entity.prefix_iter(txn, prefix_str)
            .map_err(|e| CapbitError { message: e.to_string() })?
        {
            let (key, _) = item.map_err(|e| CapbitError { message: e.to_string() })?;
            let parts = crate::keys::parse_key(key.as_bytes());
            if parts.len() == 2 {
                hashes.push(parts[1].to_string());
            }
        }

        let count = hashes.len() as u64;

        // Delete sessions and indexes
        for hash in hashes {
            dbs.sessions.delete(txn, &hash).map_err(|e| CapbitError { message: e.to_string() })?;
            let idx_key = session_idx_key(entity_id, &hash);
            dbs.sessions_by_entity.delete(txn, &idx_key).map_err(|e| CapbitError { message: e.to_string() })?;
        }

        Ok(count)
    })
}

/// Bootstrap with token - calls bootstrap() and creates root session
pub fn bootstrap_with_token(root_id: &str) -> Result<BootstrapResult> {
    let epoch = bootstrap::bootstrap(root_id)?;
    let root_entity = format!("user:{}", root_id);
    let token = create_session(&root_entity, None)?;

    Ok(BootstrapResult { root_entity, token, epoch })
}

// ============================================================================
// Password Authentication
// ============================================================================

/// Generate random salt (16 bytes, hex encoded)
fn generate_salt() -> String {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).expect("Failed to generate random bytes");
    hex::encode(bytes)
}

/// Hash password with salt
fn hash_password(salt: &str, password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

/// Set password for an entity
pub fn set_password(entity_id: &str, password: &str) -> Result<()> {
    let salt = generate_salt();
    let hash = hash_password(&salt, password);
    let value = format!("{}|{}", salt, hash);

    with_write_txn_pub(|txn, dbs| {
        dbs.credentials.put(txn, entity_id, &value)
            .map_err(|e| CapbitError { message: e.to_string() })?;
        Ok(())
    })
}

/// Verify password for an entity
pub fn verify_password(entity_id: &str, password: &str) -> Result<bool> {
    with_read_txn_pub(|txn, dbs| {
        let value = match dbs.credentials.get(txn, entity_id)
            .map_err(|e| CapbitError { message: e.to_string() })? {
            Some(v) => v.to_string(),
            None => return Ok(false),
        };

        let parts: Vec<&str> = value.split('|').collect();
        if parts.len() != 2 {
            return Err(CapbitError { message: "Corrupted credentials".into() });
        }

        let salt = parts[0];
        let stored_hash = parts[1];
        let computed_hash = hash_password(salt, password);

        Ok(stored_hash == computed_hash)
    })
}

/// Login with password, returns token
pub fn login(entity_id: &str, password: &str) -> Result<String> {
    if !verify_password(entity_id, password)? {
        return Err(CapbitError { message: "Invalid credentials".into() });
    }
    create_session(entity_id, None)
}

/// Bootstrap with password - creates root with password and returns token
pub fn bootstrap_with_password(root_id: &str, password: &str) -> Result<BootstrapResult> {
    let epoch = bootstrap::bootstrap(root_id)?;
    let root_entity = format!("user:{}", root_id);
    set_password(&root_entity, password)?;
    let token = create_session(&root_entity, None)?;

    Ok(BootstrapResult { root_entity, token, epoch })
}
