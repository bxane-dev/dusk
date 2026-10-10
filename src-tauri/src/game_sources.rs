//! Search public game listing pages on additional websites.
//! Dusk does not attempt to bypass download-host authentication or ad gates.
use regex::Regex;
use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GameSourceResult {
    title: String,
    url: String,
    description: String,
}

fn source_host(source: &str) -> Result<&'static str, String> {
    match source {
        "game3rb" => Ok("game3rb.com"),
        "fitgirl" => Ok("fitgirl-repacks.site"),
        _ => Err("Unsupported game discovery source.".into()),
    }
}

fn verified_listing(url: &str) -> Result<reqwest::Url, String> {
    let parsed = reqwest::Url::parse(url).map_err(|_| "Invalid game listing URL.".to_string())?;
    if parsed.scheme() != "https" || parsed.port().is_some()
        || parsed.username() != "" || parsed.password().is_some()
        || !matches!(parsed.host_str(), Some(
            "game3rb.com" | "www.game3rb.com" | "fitgirl-repacks.site" | "www.fitgirl-repacks.site"
        ))
    {
        return Err("Only Game3rb or FitGirl game listings are allowed.".into());
    }
    let path = parsed.path().trim_matches('/');
    // A real WordPress game listing is /game-slug/, not /category/,
    // /tag/, /wp-admin/ or any unrelated link.
    if path.is_empty() || path.contains('/') || path.starts_with("wp-") {
        return Err("Only individual game listing pages are supported.".into());
    }
    Ok(parsed)
}

fn unescape_html(value: &str) -> String {
    value.replace("&nbsp;", " ")
        .replace("&#8211;", "–")
        .replace("&#8217;", "'")
        .replace("&#038;", "&")
        .replace("&#39;", "'")
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

fn parse_results(source: &str, html: &str) -> Result<Vec<GameSourceResult>, String> {
    let pattern = match source {
        "game3rb" => r#"(?is)<h3\b[^>]*class="[^"]*\bentry-title\b[^"]*"[^>]*>\s*<a\b[^>]*href="([^"]+)"[^>]*>(.*?)</a>"#,
        "fitgirl" => r#"(?is)<h1\b[^>]*class="[^"]*\bentry-title\b[^"]*"[^>]*>\s*<a\b[^>]*href="([^"]+)"[^>]*>(.*?)</a>"#,
        _ => return Err("Unsupported game discovery source.".into()),
    };
    let hits = Regex::new(pattern).map_err(|e| e.to_string())?;
    let tags = Regex::new(r"(?s)<[^>]+>").map_err(|e| e.to_string())?;
    let expected_host = source_host(source)?;
    let mut results: Vec<GameSourceResult> = Vec::new();
    for hit in hits.captures_iter(html) {
        let raw_url = unescape_html(hit.get(1).unwrap().as_str());
        let Ok(parsed) = verified_listing(&raw_url) else { continue };
        if !matches!(parsed.host_str(), Some(host) if host == expected_host || host == format!("www.{expected_host}")) {
            continue;
        }
        let title = unescape_html(&tags.replace_all(hit.get(2).unwrap().as_str(), ""))
            .split_whitespace().collect::<Vec<_>>().join(" ");
        if title.is_empty() || results.iter().any(|entry| entry.url == raw_url) { continue; }
        results.push(GameSourceResult { title, url: raw_url, description: String::new() });
        if results.len() == 30 { break; }
    }
    Ok(results)
}

#[tauri::command]
pub(crate) async fn search_game_source(source: String, query: String) -> Result<Vec<GameSourceResult>, String> {
    let host = source_host(&source)?;
    let query = query.trim().to_string();
    if query.chars().count() < 3 || query.chars().count() > 120 {
        return Err("Enter a game title between 3 and 120 characters.".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let client = reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(24))
            .redirect(reqwest::redirect::Policy::custom(move |attempt| {
                let safe = attempt.url().scheme() == "https"
                    && attempt.url().host_str().map(|name| name == host || name == format!("www.{host}")).unwrap_or(false);
                if attempt.previous().len() >= 4 || !safe { attempt.stop() } else { attempt.follow() }
            }))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/131.0.0.0 Safari/537.36")
            .build().map_err(|e| e.to_string())?;
        let url = format!("https://{host}/");
        let response = client.get(url)
            .query(&[("s", query.as_str())])
            .send().map_err(|e| format!("Could not reach {host}: {e}"))?
            .error_for_status().map_err(|e| format!("{host} search returned an error: {e}"))?;
        let mut response = response.take(4_000_001);
        use std::io::Read;
        let mut bytes = Vec::new();
        response.read_to_end(&mut bytes).map_err(|e| format!("Could not read search page: {e}"))?;
        if bytes.len() > 4_000_000 { return Err("The search page is too large to inspect.".into()); }
        let html = String::from_utf8_lossy(&bytes);
        let results = parse_results(&source, &html)?;
        if results.is_empty() && !html.contains("entry-title") {
            return Err("This website did not return a normal game search page. Try again or open it in your browser.".into());
        }
        Ok(results)
    }).await.map_err(|e| e.to_string())?
}


fn source_page_allowed(url: &reqwest::Url, expected_host: &str) -> bool {
    if url.scheme() != "https" || url.username() != "" || url.password().is_some() {
        return false;
    }
    // Allow supported file hosts while retaining the original source restriction.
    // In-page click handling rejects unrelated pop-ups and advertising hosts.
    url.host_str().map(|host| host == expected_host || host == format!("www.{expected_host}") ||
        matches!(host, "gofile.io" | "pixeldrain.com" | "mega.nz" | "1fichier.com" |
            "filecrypt.cc" | "filecrypt.co" | "rapidgator.net" | "multiup.io" |
            "qiwi.gg" | "datanodes.to" | "buzzheavier.com" | "vikingfile.com" |
            "filekeeper.net" | "fileditchfiles.st")).unwrap_or(false)
}

fn open_game_source_page(app: tauri::AppHandle, listing: reqwest::Url) -> Result<(), String> {
    let host = listing.host_str().unwrap_or_default().trim_start_matches("www.").to_string();
    let title = if host == "fitgirl-repacks.site" {
        "Dusk — FitGirl (offline game listings)"
    } else {
        "Dusk — Game3rb listings"
    };
    tauri::WebviewWindowBuilder::new(
        &app,
        format!("game-source-{}", uuid::Uuid::new_v4().simple()),
        tauri::WebviewUrl::External(listing)
    )
    .title(title)
    .inner_size(1150.0, 800.0)
    .accept_first_mouse(true)
    // The ad blocker is intentionally always enabled, including new-window
    // browser sessions for Game3rb and offline FitGirl listings.
    .initialization_script(include_str!("online_fix_adblock.js"))
    .initialization_script(include_str!("game_source_navigation.js"))
    .on_navigation(move |target| source_page_allowed(target, &host))
    .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)
    .on_download(|_window, event| match event {
        tauri::webview::DownloadEvent::Requested { url, destination } => {
            let Some(filename) = destination.file_name().and_then(|name| name.to_str()) else {
                return false;
            };
            if !super::is_safe_game_archive_download(&url, filename) { return false; }
            let Some(home) = std::env::var_os("USERPROFILE") else { return false; };
            let folder = std::path::PathBuf::from(home).join("Downloads");
            if !folder.is_dir() { return false; }
            let mut path = folder.join(filename);
            if path.exists() {
                // Avoid clobbering existing files; preserving multipart filenames
                // is more important than guessing an incompatible part name.
                if filename.to_ascii_lowercase().contains(".part")
                    || filename.to_ascii_lowercase().ends_with(".7z.001") { return false; }
                let fpath = std::path::Path::new(filename);
                let stem = fpath.file_stem().and_then(|part| part.to_str()).unwrap_or("game");
                let ext = fpath.extension().and_then(|part| part.to_str()).unwrap_or("zip");
                let Some(next) = (2..100).map(|i| folder.join(format!("{stem} ({i}).{ext}")))
                    .find(|candidate| !candidate.exists()) else { return false; };
                path = next;
            }
            *destination = path;
            true
        }
        _ => true
    })
    .build()
    .map_err(|error| format!("Could not open game listing inside Dusk: {error}"))?;
    Ok(())
}

#[tauri::command]
pub(crate) async fn open_game_source_listing(app: tauri::AppHandle, url: String) -> Result<(), String> {
    open_game_source_page(app, verified_listing(&url)?)
}

// A native WebView search fallback for sites that reject the reqwest client.
// Search stays on the selected site's HTTPS origin with the same ad blocker.
#[tauri::command]
pub(crate) async fn open_game_source_search(app: tauri::AppHandle, source: String, query: String) -> Result<(), String> {
    let host = source_host(&source)?;
    let query = query.trim();
    if !(3..=120).contains(&query.chars().count()) {
        return Err("Enter a game title between 3 and 120 characters.".into());
    }
    let mut url = reqwest::Url::parse(&format!("https://{host}/"))
        .map_err(|e| e.to_string())?;
    url.query_pairs_mut().append_pair("s", query);
    open_game_source_page(app, url)
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub(crate) fn open_game_source_browser(url: String) -> Result<(), String> {
    verified_listing(&url)?;
    super::hidden_windows_command("rundll32")
        .args(["url.dll,FileProtocolHandler", &url])
        .spawn()
        .map_err(|e| format!("Could not open listing in browser: {e}"))?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub(crate) fn open_game_source_browser(url: String) -> Result<(), String> {
    verified_listing(&url)?;
    Err("External browser fallback is currently supported on Windows.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_game3rb_game_cards_not_sidebar_links() {
        let html = r#"<article class="post-hentry">
            <h3 class="g1-gamma entry-title">
            <a href="https://game3rb.com/stardew-valley-online/" rel="bookmark">Download Stardew Valley v1.6.15 + OnLine</a>
            </h3></article>
            <a href="https://game3rb.com/category/online/">Do not include</a>"#;
        let results = parse_results("game3rb", html).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].title.contains("Stardew Valley"));
    }

    #[test]
    fn parses_fitgirl_post_titles() {
        let html = r#"<article><header><h1 class="entry-title"><a href="https://fitgirl-repacks.site/stardew-valley/" rel="bookmark">Stardew Valley – v1.6.0 Build 24079</a></h1></header></article>"#;
        let results = parse_results("fitgirl", html).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].title.contains("Stardew Valley"));
    }

    #[test]
    fn embedded_browser_stays_on_chosen_source() {
        let host = "fitgirl-repacks.site";
        assert!(source_page_allowed(&reqwest::Url::parse("https://www.fitgirl-repacks.site/stardew-valley/").unwrap(), host));
        assert!(source_page_allowed(&reqwest::Url::parse("https://fitgirl-repacks.site/?s=example").unwrap(), host));
        assert!(!source_page_allowed(&reqwest::Url::parse("https://ads.example/gate").unwrap(), host));
        assert!(!source_page_allowed(&reqwest::Url::parse("https://fitgirl-repacks.site.evil.org/").unwrap(), host));
    }

    #[test]
    fn blocks_third_party_sites_and_unrelated_paths() {
        assert!(verified_listing("https://fitgirl-repacks.site/stardew-valley/").is_ok());
        assert!(verified_listing("https://game3rb.com/stardew-valley-online/").is_ok());
        assert!(verified_listing("https://game3rb.com.evil.example/foo/").is_err());
        assert!(verified_listing("http://game3rb.com/foo/").is_err());
        assert!(verified_listing("https://fitgirl-repacks.site/category/games/").is_err());
    }
}
