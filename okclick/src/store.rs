/// Tamper-evident persistence — same algorithm as ok typer.
///
/// Format of ~/.local/share/okcounter/history.txt (one line):
///   <ok>:<okay>:<total_session_secs>:<hmac_hex>
///
/// HMAC-SHA256 keyed with /etc/machine-id (fallback: /var/lib/dbus/machine-id).
/// Editing the count invalidates the MAC → load returns 0.
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use std::{fs, path::PathBuf};

type HmacSha256 = Hmac<Sha256>;

fn machine_key() -> Vec<u8> {
    let raw = fs::read_to_string("/etc/machine-id")
        .or_else(|_| fs::read_to_string("/var/lib/dbus/machine-id"))
        .unwrap_or_else(|_| "okcounter-fallback-key-no-machine-id".into());
    format!("okcounter:v1:{}", raw.trim()).into_bytes()
}

fn sign(ok: u64, okay: u64, secs: u64, key: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(format!("{}:{}:{}", ok, okay, secs).as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn verify(ok: u64, okay: u64, secs: u64, tag: &str, key: &[u8]) -> bool {
    let tag_bytes = match hex::decode(tag) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(format!("{}:{}:{}", ok, okay, secs).as_bytes());
    mac.verify_slice(&tag_bytes).is_ok()
}

fn data_path() -> PathBuf {
    let base = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(base).join(".local/share/okcounter")
}

/// Returns (ok_all_time, okay_all_time, saved_secs)
pub fn load() -> (u64, u64, u64) {
    let path = data_path().join("history.txt");
    let key = machine_key();

    let Ok(contents) = fs::read_to_string(&path) else {
        return (0, 0, 0);
    };

    let parts: Vec<&str> = contents.trim().splitn(4, ':').collect();

    // Migrate old 3-field format (ok:secs:hmac)
    if parts.len() == 3 {
        let Ok(ok) = parts[0].parse::<u64>() else { return (0, 0, 0) };
        let Ok(secs) = parts[1].parse::<u64>() else { return (0, 0, 0) };
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

    let Ok(ok) = parts[0].parse::<u64>() else { return (0, 0, 0) };
    let Ok(okay) = parts[1].parse::<u64>() else { return (0, 0, 0) };
    let Ok(secs) = parts[2].parse::<u64>() else { return (0, 0, 0) };

    if verify(ok, okay, secs, parts[3], &key) {
        (ok, okay, secs)
    } else {
        (0, 0, 0)
    }
}

pub fn save(ok: u64, okay: u64, total_secs: u64) {
    let dir = data_path();
    let _ = fs::create_dir_all(&dir);
    let key = machine_key();
    let tag = sign(ok, okay, total_secs, &key);
    let line = format!("{}:{}:{}:{}\n", ok, okay, total_secs, tag);
    let _ = fs::write(dir.join("history.txt"), line);
}

pub fn wipe() {
    let _ = fs::remove_file(data_path().join("history.txt"));
}
