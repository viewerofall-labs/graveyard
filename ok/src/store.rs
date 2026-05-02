/// Tamper-evident persistence for the ok counter.
///
/// Format of ~/.local/share/okcounter/history.txt (one line):
///   <count>:<total_session_secs>:<hmac_hex>
///
/// The HMAC is HMAC-SHA256 keyed with the machine-id read from
/// /etc/machine-id (falling back to /var/lib/dbus/machine-id).
/// Editing the count invalidates the MAC → load returns 0.
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::{
    fs,
    path::PathBuf,
};

type HmacSha256 = Hmac<Sha256>;

// ── key material ────────────────────────────────────────────────────────────

fn machine_key() -> Vec<u8> {
    // Try canonical location first, then dbus fallback
    let raw = fs::read_to_string("/etc/machine-id")
        .or_else(|_| fs::read_to_string("/var/lib/dbus/machine-id"))
        .unwrap_or_else(|_| "okcounter-fallback-key-no-machine-id".into());
    // Trim whitespace and append a domain separator so the key is purpose-bound
    format!("okcounter:v1:{}", raw.trim()).into_bytes()
}

// ── MAC helpers ──────────────────────────────────────────────────────────────

fn sign(ok: u64, okay: u64, secs: u64, key: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(format!("{}:{}:{}", ok, okay, secs).as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn verify(ok: u64, okay: u64, secs: u64, tag: &str, key: &[u8]) -> bool {
    let expected = sign(ok, okay, secs, key);
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(format!("{}:{}:{}", ok, okay, secs).as_bytes());
    let tag_bytes = match hex::decode(tag) {
        Ok(b) => b,
        Err(_) => return false,
    };
    mac.verify_slice(&tag_bytes).is_ok() && expected == tag
}

// ── public API ───────────────────────────────────────────────────────────────

fn data_path() -> PathBuf {
    let base = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(base)
        .join(".local/share/okcounter")
}

/// Returns (ok_all_time, okay_all_time, saved_secs)
pub fn load() -> (u64, u64, u64) {
    let path = data_path().join("history.txt");
    let key = machine_key();

    let Ok(contents) = fs::read_to_string(&path) else {
        return (0, 0, 0);
    };

    let parts: Vec<&str> = contents.trim().splitn(4, ':').collect();

    // Support old 3-field format (ok:secs:hmac) — migrate gracefully
    if parts.len() == 3 {
        let Ok(ok)   = parts[0].parse::<u64>() else { return (0, 0, 0) };
        let Ok(secs) = parts[1].parse::<u64>() else { return (0, 0, 0) };
        // Old MAC was over "ok:secs" — re-verify with old scheme
        let mut mac = HmacSha256::new_from_slice(&key).expect("hmac");
        mac.update(format!("{}:{}", ok, secs).as_bytes());
        let tag_bytes = match hex::decode(parts[2]) {
            Ok(b) => b,
            Err(_) => return (0, 0, 0),
        };
        if mac.verify_slice(&tag_bytes).is_ok() {
            return (ok, 0, secs);
        }
        return (0, 0, 0);
    }

    if parts.len() != 4 {
        return (0, 0, 0);
    }

    let Ok(ok)   = parts[0].parse::<u64>() else { return (0, 0, 0) };
    let Ok(okay) = parts[1].parse::<u64>() else { return (0, 0, 0) };
    let Ok(secs) = parts[2].parse::<u64>() else { return (0, 0, 0) };
    let tag = parts[3];

    if verify(ok, okay, secs, tag, &key) {
        (ok, okay, secs)
    } else {
        (0, 0, 0)
    }
}

pub fn wipe() {
    let path = data_path().join("history.txt");
    let _ = fs::remove_file(path);
}

pub fn save(ok: u64, okay: u64, total_secs: u64) {
    let dir = data_path();
    let _ = fs::create_dir_all(&dir);
    let key = machine_key();
    let tag = sign(ok, okay, total_secs, &key);
    let line = format!("{}:{}:{}:{}\n", ok, okay, total_secs, tag);
    let _ = fs::write(dir.join("history.txt"), line);
}
