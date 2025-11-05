//! Anonymous machine ID generation with salted hashing
//!
//! Following Turborepo's approach for privacy-preserving analytics

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use uuid::Uuid;

use crate::config::get_telemetry_dir;

/// Get or generate an anonymous machine ID
///
/// The machine ID is a SHA256 hash of:
/// - MAC address (or hostname as fallback)
/// - Random UUID salt (stored in ~/.cagents/telemetry/salt)
///
/// This ensures:
/// - The ID is stable for a given machine
/// - The ID is completely anonymous and unlinkable across machines
/// - Different machines with the same MAC have different IDs (due to unique salts)
pub fn get_or_generate_machine_id() -> Result<String> {
    let telemetry_dir = get_telemetry_dir()?;
    let machine_id_path = telemetry_dir.join("machine_id");

    // Try to load cached machine ID
    if machine_id_path.exists() {
        if let Ok(id) = fs::read_to_string(&machine_id_path) {
            if !id.trim().is_empty() {
                return Ok(id.trim().to_string());
            }
        }
    }

    // Generate new machine ID
    let machine_id = generate_machine_id()?;

    // Cache it
    fs::write(&machine_id_path, &machine_id)?;

    Ok(machine_id)
}

/// Generate a new anonymous machine ID
fn generate_machine_id() -> Result<String> {
    // Get stable machine identifier
    let machine_identifier = get_machine_identifier()?;

    // Get or create salt
    let salt = get_or_create_salt()?;

    // Hash with salt using SHA256
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(machine_identifier.as_bytes());
    let hash = hasher.finalize();

    // Return hex-encoded hash (64 chars)
    Ok(format!("{:x}", hash))
}

/// Get a stable machine identifier (MAC address, hostname, or UUID fallback)
fn get_machine_identifier() -> Result<String> {
    // Try MAC address first (most stable)
    if let Ok(mac) = mac_address::get_mac_address() {
        if let Some(mac_addr) = mac {
            return Ok(mac_addr.to_string());
        }
    }

    // Fallback to hostname
    if let Ok(hostname) = hostname::get() {
        if let Some(hostname_str) = hostname.to_str() {
            if !hostname_str.is_empty() {
                return Ok(hostname_str.to_string());
            }
        }
    }

    // Last resort: generate a new UUID
    // This won't be stable across runs, but at least it's anonymous
    Ok(Uuid::new_v4().to_string())
}

/// Get or create a random salt for hashing
fn get_or_create_salt() -> Result<String> {
    let telemetry_dir = get_telemetry_dir()?;
    let salt_path = telemetry_dir.join("salt");

    // Try to load existing salt
    if salt_path.exists() {
        if let Ok(salt) = fs::read_to_string(&salt_path) {
            if !salt.trim().is_empty() {
                return Ok(salt.trim().to_string());
            }
        }
    }

    // Generate new salt
    let salt = Uuid::new_v4().to_string();

    // Store it
    fs::write(&salt_path, &salt)
        .context("Failed to write salt file")?;

    Ok(salt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_machine_id_is_stable() {
        let id1 = generate_machine_id().unwrap();
        let id2 = generate_machine_id().unwrap();

        // Should be the same on the same machine
        assert_eq!(id1, id2);
        assert_eq!(id1.len(), 64); // SHA256 hex is 64 chars
    }

    #[test]
    fn test_get_machine_identifier() {
        let identifier = get_machine_identifier().unwrap();
        assert!(!identifier.is_empty());
    }

    #[test]
    fn test_salt_persistence() {
        let salt1 = get_or_create_salt().unwrap();
        let salt2 = get_or_create_salt().unwrap();

        // Salt should be stable
        assert_eq!(salt1, salt2);
        assert!(!salt1.is_empty());
    }
}
