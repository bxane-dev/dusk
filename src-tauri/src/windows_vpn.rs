//! Native Windows VPN profile integration. This does not operate a VPN
//! server or claim protection when Windows has no configured tunnel.
//! Credentials remain in Windows; Dusk never reads or stores passwords.
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WindowsVpnProfile {
    name: String,
    connected: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VpnConnectionResult {
    connected: bool,
    profile: Option<String>,
    message: String,
}

#[cfg(target_os = "windows")]
fn local_profiles() -> Result<Vec<WindowsVpnProfile>, String> {
    // A fixed script only: no user input is ever interpolated into PowerShell.
    let output = super::hidden_windows_command("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command",
            "$ErrorActionPreference='Stop'; @(Get-VpnConnection -ErrorAction SilentlyContinue | Select-Object Name,ConnectionStatus) | ConvertTo-Json -Compress"])
        .output().map_err(|_| "Could not query Windows VPN profiles.".to_string())?;
    if !output.status.success() {
        return Err("Windows could not list VPN profiles. Check Windows VPN settings.".into());
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    let raw = raw.trim().trim_start_matches('\u{feff}');
    if raw.is_empty() || raw == "null" { return Ok(Vec::new()); }
    let value: serde_json::Value = serde_json::from_str(raw)
        .map_err(|_| "Windows returned unreadable VPN profile information.".to_string())?;
    let values = match value {
        serde_json::Value::Array(values) => values,
        serde_json::Value::Object(_) => vec![value],
        _ => return Ok(Vec::new()),
    };
    let mut profiles = Vec::new();
    for value in values {
        let Some(name) = value.get("Name").and_then(|v| v.as_str()) else { continue };
        if name.is_empty() || name.len() > 256 || profiles.iter().any(|p: &WindowsVpnProfile| p.name == name) {
            continue;
        }
        let connected = value.get("ConnectionStatus").and_then(|v| v.as_str())
            .is_some_and(|status| status.eq_ignore_ascii_case("Connected"));
        profiles.push(WindowsVpnProfile { name: name.to_string(), connected });
    }
    Ok(profiles)
}

#[cfg(not(target_os = "windows"))]
fn local_profiles() -> Result<Vec<WindowsVpnProfile>, String> {
    Ok(Vec::new())
}

const WARP_PROFILE: &str = "Cloudflare WARP";

#[cfg(target_os = "windows")]
fn cloudflare_warp_cli() -> Option<std::path::PathBuf> {
    for key in ["ProgramFiles", "ProgramFiles(x86)"] {
        if let Some(dir) = std::env::var_os(key) {
            let candidate = std::path::PathBuf::from(dir)
                .join("Cloudflare").join("Cloudflare WARP").join("warp-cli.exe");
            if candidate.is_file() { return Some(candidate); }
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn warp_status(cli: &std::path::Path) -> Result<bool, String> {
    let output = super::hidden_windows_command(cli.to_string_lossy().as_ref())
        .arg("status").output()
        .map_err(|_| "Cloudflare WARP is installed but not responding.".to_string())?;
    if !output.status.success() {
        return Err("Cloudflare WARP could not report its status. Open WARP to finish its initial setup.".into());
    }
    let report = String::from_utf8_lossy(&output.stdout);
    Ok(warp_reports_connected(&report))
}

#[cfg(target_os = "windows")]
fn warp_reports_connected(output: &str) -> bool {
    output.lines().any(|line| {
        let normalized = line.trim().to_ascii_lowercase();
        (normalized.starts_with("status update:") || normalized.starts_with("status:"))
            && normalized.split_once(':').is_some_and(|(_, status)| status.trim().starts_with("connected"))
    })
}

#[cfg(target_os = "windows")]
fn connect_warp(cli: &std::path::Path) -> Result<VpnConnectionResult, String> {
    if warp_status(cli).unwrap_or(false) {
        return Ok(VpnConnectionResult {
            connected: true, profile: Some(WARP_PROFILE.into()),
            message: "Cloudflare WARP reports Connected. Network routing follows the WARP client settings.".into(),
        });
    }
    let result = super::hidden_windows_command(cli.to_string_lossy().as_ref())
        .arg("connect").output()
        .map_err(|_| "Cloudflare WARP could not start. Open its app and complete first-run setup.".to_string())?;
    if !result.status.success() {
        return Ok(VpnConnectionResult {
            connected: false, profile: Some(WARP_PROFILE.into()),
            message: "WARP connection failed. Open Cloudflare WARP, accept its terms and register the device first.".into(),
        });
    }
    let connected = warp_status(cli).unwrap_or(false);
    Ok(VpnConnectionResult {
        connected, profile: Some(WARP_PROFILE.into()),
        message: if connected {
            "Cloudflare WARP reports Connected. Routing follows the WARP client settings.".into()
        } else {
            "Cloudflare WARP has not confirmed a connection. Open its app to check setup and network access.".into()
        },
    })
}

#[tauri::command]
pub(crate) async fn list_windows_vpn_profiles() -> Result<Vec<WindowsVpnProfile>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let mut profiles = local_profiles()?;
        #[cfg(target_os = "windows")]
        if let Some(cli) = cloudflare_warp_cli() {
            profiles.insert(0, WindowsVpnProfile {
                name: WARP_PROFILE.into(),
                connected: warp_status(&cli).unwrap_or(false),
            });
        }
        Ok(profiles)
    }).await.map_err(|error| format!("VPN profile lookup failed: {error}"))?
}

#[cfg(target_os = "windows")]
fn connect_profile(requested: Option<String>) -> Result<VpnConnectionResult, String> {
    let profiles = local_profiles()?;
    let warp = cloudflare_warp_cli();
    if requested.as_deref() == Some(WARP_PROFILE) {
        return match warp {
            Some(ref cli) => connect_warp(cli),
            None => Ok(VpnConnectionResult {
                connected: false, profile: Some(WARP_PROFILE.into()),
                message: "Cloudflare WARP is not installed. Use Install Cloudflare WARP in Dusk Settings.".into(),
            }),
        };
    }
    // Automatic favors any existing connected VPN, then Cloudflare WARP, then a
    // single configured Windows VPN connection.
    if requested.as_deref().unwrap_or_default().trim().is_empty() {
        if let Some(ref cli) = warp {
            if warp_status(cli).unwrap_or(false) { return connect_warp(cli); }
        }
        if let Some(active) = profiles.iter().find(|p| p.connected) {
            return Ok(VpnConnectionResult {
                connected: true, profile: Some(active.name.clone()),
                message: "Windows reports an existing VPN connection.".into(),
            });
        }
        if let Some(ref cli) = warp { return connect_warp(cli); }
    }
    if profiles.is_empty() {
        return Ok(VpnConnectionResult {
            connected: false, profile: None,
            message: "No VPN provider is installed. Install Cloudflare WARP in Dusk Settings or configure a Windows VPN profile.".into(),
        });
    }
    let selected = if let Some(name) = requested.filter(|value| !value.trim().is_empty()) {
        profiles.iter().find(|p| p.name == name)
            .ok_or_else(|| "Selected VPN profile no longer exists in Windows.".to_string())?
    } else if let Some(connected) = profiles.iter().find(|p| p.connected) {
        connected
    } else if profiles.len() == 1 {
        &profiles[0]
    } else {
        return Ok(VpnConnectionResult {
            connected: false, profile: None,
            message: "Multiple Windows VPN profiles found. Select one in Dusk Settings to connect automatically.".into(),
        });
    };
    let name = selected.name.clone();
    if selected.connected {
        return Ok(VpnConnectionResult {
            connected: true, profile: Some(name),
            message: "Already connected through Windows VPN.".into(),
        });
    }
    // rasdial only uses a preconfigured Windows VPN connection and its saved
    // credentials. It never receives credentials or arbitrary shell commands.
    let status = super::hidden_windows_command("rasdial.exe")
        .arg(&name).output()
        .map_err(|_| "Could not start the Windows VPN connection.".to_string())?;
    if !status.status.success() {
        return Ok(VpnConnectionResult {
            connected: false, profile: Some(name),
            message: "Windows could not connect this VPN. Check its saved credentials and server in Windows Settings.".into(),
        });
    }
    // Do not claim protection until Windows reports the connection as active.
    let connected = local_profiles()?.iter().any(|p| p.name == name && p.connected);
    Ok(VpnConnectionResult {
        connected, profile: Some(name),
        message: if connected {
            "Windows reports the VPN is connected.".into()
        } else {
            "VPN connection was requested, but Windows has not confirmed it is active.".into()
        },
    })
}

#[cfg(not(target_os = "windows"))]
fn connect_profile(_requested: Option<String>) -> Result<VpnConnectionResult, String> {
    Ok(VpnConnectionResult {
        connected: false, profile: None,
        message: "Windows VPN profile integration is only available in the Windows build.".into(),
    })
}

#[tauri::command]
pub(crate) async fn prepare_windows_vpn(profile: Option<String>) -> Result<VpnConnectionResult, String> {
    tauri::async_runtime::spawn_blocking(move || connect_profile(profile))
        .await.map_err(|error| format!("Windows VPN setup failed: {error}"))?
}

#[tauri::command]
pub(crate) fn open_windows_vpn_settings() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        super::hidden_windows_command("explorer.exe")
            .arg("ms-settings:network-vpn")
            .spawn()
            .map_err(|_| "Could not open Windows VPN settings.".to_string())?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("Windows VPN settings are unavailable on this operating system.".into())
    }
}

#[tauri::command]
pub(crate) fn open_cloudflare_warp_setup() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        super::hidden_windows_command("explorer.exe")
            .arg("https://developers.cloudflare.com/warp-client/get-started/windows/")
            .spawn()
            .map_err(|_| "Could not open the official Cloudflare WARP setup guide.".to_string())?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("Cloudflare WARP setup is currently supported on Windows.".into())
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;
    #[test]
    fn recognizes_only_connected_warp_status() {
        assert!(warp_reports_connected("Status update: Connected\n"));
        assert!(!warp_reports_connected("Status update: Disconnected\n"));
        assert!(!warp_reports_connected("Connection: Connected\nStatus update: Connecting\n"));
    }
}
