use super::*;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::net::IpAddr;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

const MAX_DOWNLOAD_BYTES: u64 = 100 * 1024 * 1024 * 1024;
static JOBS: OnceLock<Mutex<HashMap<String, DownloadJob>>> = OnceLock::new();

fn jobs() -> &'static Mutex<HashMap<String, DownloadJob>> {
    JOBS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DownloadStatus {
    id: String,
    title: String,
    filename: String,
    file_path: String,
    status: String,
    received_bytes: u64,
    total_bytes: Option<u64>,
    error: Option<String>,
}

struct DownloadJob {
    info: DownloadStatus,
    cancel: bool,
}

fn with_job(id: &str, update: impl FnOnce(&mut DownloadJob)) {
    if let Ok(mut map) = jobs().lock() {
        if let Some(job) = map.get_mut(id) { update(job); }
    }
}

fn valid_public_https(url: &reqwest::Url) -> bool {
    if url.scheme() != "https" || url.username() != "" || url.password().is_some() {
        return false;
    }
    let Some(host) = url.host_str() else { return false };
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    if host == "localhost" || host.ends_with(".localhost")
        || host.ends_with(".local") || host.ends_with(".internal")
        || super::known_ad_network(&host) {
        return false;
    }
    if let Ok(ip) = host.trim_matches(['[', ']']).parse::<IpAddr>() {
        match ip {
            IpAddr::V4(ip) =>
                !ip.is_private() && !ip.is_loopback() && !ip.is_link_local()
                    && !ip.is_multicast() && !ip.is_unspecified() && !ip.is_broadcast(),
            IpAddr::V6(ip) =>
                !ip.is_loopback() && !ip.is_unspecified()
                    && !ip.is_multicast() && !ip.is_unique_local()
                    && !ip.is_unicast_link_local(),
        }
    } else {
        !host.is_empty()
    }
}

fn archive_name(name: &str) -> bool {
    if name.len() > 210 || name.is_empty() || name == "." || name == ".."
        || name.chars().any(|c| c.is_control() || matches!(c, '/' | '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*'))
        || name.ends_with('.') || name.ends_with(' ') {
        return false;
    }
    let name = name.to_ascii_lowercase();
    name.ends_with(".zip") || name.ends_with(".rar") || name.ends_with(".7z")
        || name.ends_with(".7z.001") || name.ends_with(".7z.002")
        || name.ends_with(".7z.003") || name.ends_with(".7z.004")
        || name.ends_with(".part1.rar") || name.ends_with(".part01.rar")
        || name.ends_with(".part2.rar") || name.ends_with(".part02.rar")
        || name.ends_with(".part3.rar") || name.ends_with(".part03.rar")
}

fn validate_archive_header(name: &str, bytes: &[u8]) -> bool {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".zip") {
        bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06")
            || bytes.starts_with(b"PK\x07\x08")
    } else if lower.ends_with(".7z") || lower.ends_with(".7z.001") {
        bytes.starts_with(b"7z\xbc\xaf\x27\x1c")
    } else if lower.ends_with(".rar") && !lower.contains(".part2.") && !lower.contains(".part02.")
        && !lower.contains(".part3.") && !lower.contains(".part03.") {
        bytes.starts_with(b"Rar!\x1a\x07")
    } else {
        // Non-first multipart volumes may not carry a format signature.
        !bytes.starts_with(b"<!DOCTYPE") && !bytes.starts_with(b"<html")
    }
}

fn choose_path(filename: &str) -> Result<std::path::PathBuf, String> {
    let home = std::env::var_os("USERPROFILE")
        .ok_or("Windows user profile not found")?;
    let dir = std::path::PathBuf::from(home).join("Downloads");
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let initial = dir.join(filename);
    if !initial.exists() && !initial.with_extension(format!("{}.dusk-part", initial.extension().and_then(|e|e.to_str()).unwrap_or(""))).exists() {
        return Ok(initial);
    }
    let source = std::path::Path::new(filename);
    let stem = source.file_stem().and_then(|value| value.to_str()).unwrap_or("Archive");
    let ext = source.extension().and_then(|value| value.to_str()).unwrap_or("zip");
    for number in 2..1000 {
        let candidate = dir.join(format!("{stem} ({number}).{ext}"));
        if !candidate.exists() { return Ok(candidate); }
    }
    Err("Could not allocate another filename in Downloads.".into())
}

fn update_complete(id: &str, state: &str, error: Option<String>) {
    with_job(id, |job| {
        job.info.status = state.into();
        job.info.error = error;
    });
}

fn download_file(id: String, url: reqwest::Url, filename: String, target: std::path::PathBuf) {
    let temp = target.with_extension(format!(
        "{}.dusk-part", target.extension().and_then(|ext| ext.to_str()).unwrap_or("archive")
    ));
    let output = (|| -> Result<(), String> {
        let client = reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(60))
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                if attempt.previous().len() > 8 || !valid_public_https(attempt.url()) {
                    attempt.stop()
                } else {
                    attempt.follow()
                }
            }))
            .user_agent("DuskLauncher/1.8")
            .build().map_err(|e| e.to_string())?;
        let mut response = client.get(url).send().map_err(|e| format!("Cannot connect to download: {e}"))?
            .error_for_status().map_err(|e| format!("File host declined download: {e}"))?;
        if !valid_public_https(response.url()) {
            return Err("Download was redirected to a blocked location.".into());
        }
        if let Some(size) = response.content_length() {
            if size > MAX_DOWNLOAD_BYTES {
                return Err("Archive exceeds 100 GiB download limit.".into());
            }
            with_job(&id, |job| job.info.total_bytes = Some(size));
        }
        let content_type = response.headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()).unwrap_or("")
            .to_ascii_lowercase();
        if content_type.contains("text/html") || content_type.contains("application/json") {
            return Err("The URL returned a web page, not an archive. Finish the host download steps and use the actual file link.".into());
        }

        let mut output = File::create(&temp).map_err(|e| e.to_string())?;
        let mut received = 0u64;
        let mut first = true;
        let mut chunk = [0u8; 64 * 1024];
        loop {
            if jobs().lock().map_err(|e| e.to_string())?
                .get(&id).map(|job| job.cancel).unwrap_or(true) {
                return Err("Cancelled".into());
            }
            let size = response.read(&mut chunk)
                .map_err(|e| format!("Download interrupted: {e}"))?;
            if size == 0 { break; }
            if first {
                first = false;
                if !validate_archive_header(&filename, &chunk[..size]) {
                    return Err("The server did not send an archive. It may require login, cookies, or a separate download step.".into());
                }
            }
            received = received.saturating_add(size as u64);
            if received > MAX_DOWNLOAD_BYTES {
                return Err("Archive exceeds 100 GiB download limit.".into());
            }
            output.write_all(&chunk[..size]).map_err(|e| e.to_string())?;
            with_job(&id, |job| job.info.received_bytes = received);
        }
        if received == 0 { return Err("The server returned an empty download.".into()); }
        if let Some(expected) = response.content_length() {
            if received != expected { return Err("Incomplete download: server sent fewer bytes than expected.".into()); }
        }
        output.sync_all().map_err(|e| e.to_string())?;
        drop(output);
        fs::rename(&temp, &target).map_err(|e| format!("Could not save completed archive: {e}"))?;
        Ok(())
    })();

    match output {
        Ok(()) => update_complete(&id, "completed", None),
        Err(message) => {
            let cancelled = message == "Cancelled";
            let _ = fs::remove_file(&temp);
            update_complete(&id, if cancelled { "cancelled" } else { "failed" },
                if cancelled { None } else { Some(message) });
        }
    }
}

#[tauri::command]
pub(crate) fn start_managed_download(url: String, filename: String, title: String) -> Result<DownloadStatus, String> {
    let parsed = reqwest::Url::parse(url.trim()).map_err(|_| "Invalid download URL".to_string())?;
    if !valid_public_https(&parsed) { return Err("Only public HTTPS download URLs are allowed.".into()); }
    let filename = filename.trim().to_string();
    if !archive_name(&filename) {
        return Err("Enter a safe archive filename ending in .zip, .rar, or .7z (including supported multipart archives).".into());
    }
    if title.trim().is_empty() { return Err("Enter a game title.".into()); }
    let target = choose_path(&filename)?;
    let id = Uuid::new_v4().to_string();
    let info = DownloadStatus {
        id: id.clone(), title: title.trim().chars().take(140).collect(),
        filename, file_path: target.to_string_lossy().into_owned(),
        status: "downloading".into(), received_bytes: 0, total_bytes: None, error: None,
    };
    let mut map = jobs().lock().map_err(|e| e.to_string())?;
    if map.values().filter(|job| job.info.status == "downloading").count() >= 3 {
        return Err("Maximum three simultaneous downloads.".into());
    }
    map.insert(id.clone(), DownloadJob { info: info.clone(), cancel: false });
    drop(map);
    let worker_filename = info.filename.clone();
    thread::spawn(move || download_file(id, parsed, worker_filename, target));
    Ok(info)
}

#[tauri::command]
pub(crate) fn list_managed_downloads() -> Result<Vec<DownloadStatus>, String> {
    let map = jobs().lock().map_err(|e| e.to_string())?;
    let mut items: Vec<_> = map.values().map(|job| job.info.clone()).collect();
    items.sort_by(|a,b| b.id.cmp(&a.id));
    Ok(items)
}

#[tauri::command]
pub(crate) fn cancel_managed_download(download_id: String) -> Result<(), String> {
    let mut map = jobs().lock().map_err(|e| e.to_string())?;
    let job = map.get_mut(&download_id).ok_or("Download not found.")?;
    if job.info.status != "downloading" { return Err("Download is no longer active.".into()); }
    job.cancel = true;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn allows_public_https_but_not_private_networks() {
        assert!(valid_public_https(&reqwest::Url::parse("https://cdn.example.org/game.zip").unwrap()));
        for url in ["http://cdn.example.org/file.zip", "https://localhost/file.zip",
            "https://127.0.0.1/test.zip", "https://192.168.1.10/a.zip",
            "https://user:pass@cdn.example.org/a.zip", "https://sub.exoclick.com/a.zip"] {
            assert!(!valid_public_https(&reqwest::Url::parse(url).unwrap()), "{url}");
        }
    }
    #[test]
    fn checks_filename_and_archive_signature() {
        assert!(archive_name("My Game.zip"));
        assert!(archive_name("Game.part1.rar"));
        assert!(!archive_name("../game.zip"));
        assert!(!archive_name("setup.exe"));
        assert!(validate_archive_header("game.zip", b"PK\x03\x04A"));
        assert!(validate_archive_header("game.rar", b"Rar!\x1a\x07"));
        assert!(!validate_archive_header("game.zip", b"<html>Access denied"));
    }
}
