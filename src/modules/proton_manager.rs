use std::path::PathBuf;
use std::time::{Duration, SystemTime};

#[derive(Clone)]
pub struct CompatTool {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub repo: &'static str,
    pub asset_filter: &'static str,
    pub arch_filter: Option<&'static str>,
    #[allow(dead_code)]
    pub install_subdir: &'static str,
    pub match_fn: fn(&str) -> bool,
}

#[derive(Clone)]
pub struct ReleaseInfo {
    pub version: String,
    pub asset_name: String,
    pub download_url: String,
    pub body: String,
}

pub struct InstalledVersion {
    pub version: String,
    pub path: PathBuf,
}

pub struct ProtonManager;

impl ProtonManager {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    pub fn available_tools() -> Vec<CompatTool> {
        vec![
            CompatTool {
                id: "ge-proton",
                name: "GE-Proton",
                description: "Custom Proton build by GloriousEggroll with improved game compatibility",
                repo: "GloriousEggroll/proton-ge-custom",
                asset_filter: r"GE-Proton.*\.tar\.gz",
                arch_filter: None,
                install_subdir: "compatibilitytools.d",
                match_fn: |n| n.starts_with("GE-Proton") && !n.to_lowercase().contains("rtsp"),
            },
            CompatTool {
                id: "proton-cachyos",
                name: "Proton-CachyOS",
                description: "Proton optimized by CachyOS with additional patches",
                repo: "CachyOS/proton-cachyos",
                asset_filter: r"proton-cachyos.*\.tar\.(?:xz|zst)",
                arch_filter: Some("-x86_64.tar"),
                install_subdir: "compatibilitytools.d",
                match_fn: |n| n.starts_with("proton-cachyos"),
            },
            CompatTool {
                id: "ge-proton-rtsp",
                name: "GE-Proton RTSP",
                description: "GE-Proton with RTSP patches for better video playback",
                repo: "SpookySkeletons/proton-ge-rtsp",
                asset_filter: r"GE-Proton.*\.tar\.gz",
                arch_filter: None,
                install_subdir: "compatibilitytools.d",
                match_fn: |n| {
                    let lower = n.to_lowercase();
                    lower.contains("ge-proton") && lower.contains("rtsp")
                },
            },
            CompatTool {
                id: "dxvk",
                name: "DXVK",
                description: "DirectX 9/10/11 to Vulkan translation layer",
                repo: "doitsujin/dxvk",
                asset_filter: r"dxvk-.*\.tar\.gz",
                arch_filter: None,
                install_subdir: "",
                match_fn: |n| n.starts_with("dxvk-"),
            },
            CompatTool {
                id: "vkd3d-proton",
                name: "VKD3D-Proton",
                description: "DirectX 12 to Vulkan translation layer",
                repo: "HansKristian-Work/vkd3d-proton",
                asset_filter: r"vkd3d-proton-.*\.tar\.zst",
                arch_filter: None,
                install_subdir: "",
                match_fn: |n| n.starts_with("vkd3d-proton-"),
            },
        ]
    }

    pub fn steam_install_dirs() -> Vec<PathBuf> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let candidates = vec![
            PathBuf::from(&home).join(".steam/root/compatibilitytools.d"),
            PathBuf::from(&home).join(".steam/steam/compatibilitytools.d"),
            PathBuf::from(&home).join(".local/share/Steam/compatibilitytools.d"),
            PathBuf::from(&home).join(
                ".var/app/com.valvesoftware.Steam/data/Steam/compatibilitytools.d",
            ),
        ];
        candidates
            .into_iter()
            .filter(|p| p.exists())
            .collect()
    }

    #[allow(dead_code)]
    pub fn lutris_wine_dirs() -> Vec<PathBuf> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let candidates = vec![
            PathBuf::from(&home).join(".local/share/lutris/runners/wine"),
            PathBuf::from(&home).join(".var/app/net.lutris.Lutris/data/lutris/runners/wine"),
        ];
        candidates
            .into_iter()
            .filter(|p| p.exists())
            .collect()
    }

    fn cache_dir() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let dir = PathBuf::from(home).join(".cache/parch-gamehub/github_releases");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    fn read_cached_releases(repo: &str, max_age: Option<Duration>) -> Option<String> {
        let safe_name = repo.replace('/', "_");
        let path = Self::cache_dir().join(format!("{}.json", safe_name));
        if let Ok(meta) = std::fs::metadata(&path) {
            if let Some(ttl) = max_age {
                if let Ok(mtime) = meta.modified() {
                    if let Ok(age) = SystemTime::now().duration_since(mtime) {
                        if age > ttl {
                            return None;
                        }
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            if let Ok(content) = std::fs::read_to_string(&path) {
                if content.trim_start().starts_with('[') {
                    return Some(content);
                }
            }
        }
        None
    }

    fn write_cached_releases(repo: &str, data: &str) {
        let safe_name = repo.replace('/', "_");
        let path = Self::cache_dir().join(format!("{}.json", safe_name));
        let _ = std::fs::write(path, data);
    }

    pub fn list_available_releases(tool: &CompatTool) -> Vec<ReleaseInfo> {
        // 1. Try reading fresh cache (1 hour TTL)
        let body = if let Some(cached) = Self::read_cached_releases(tool.repo, Some(Duration::from_secs(3600))) {
            cached
        } else {
            // 2. Fetch from GitHub API via native ureq
            let url = format!("https://api.github.com/repos/{}/releases?per_page=30", tool.repo);
            let mut req = ureq::get(&url)
                .set("User-Agent", "ParchGameHub/0.2.0 (+https://github.com/parchlinux)")
                .timeout(Duration::from_secs(12));

            // Optional token support to avoid unauthenticated rate limits if configured
            if let Ok(token) = std::env::var("GITHUB_TOKEN").or_else(|_| std::env::var("GH_TOKEN")) {
                if !token.is_empty() {
                    req = req.set("Authorization", &format!("Bearer {}", token));
                }
            }

            match req.call() {
                Ok(resp) if resp.status() == 200 => {
                    if let Ok(text) = resp.into_string() {
                        if text.trim_start().starts_with('[') {
                            Self::write_cached_releases(tool.repo, &text);
                            text
                        } else {
                            Self::read_cached_releases(tool.repo, None).unwrap_or_default()
                        }
                    } else {
                        Self::read_cached_releases(tool.repo, None).unwrap_or_default()
                    }
                }
                _ => {
                    // Fallback to stale cache on network error, 403 rate limit, or 429
                    Self::read_cached_releases(tool.repo, None).unwrap_or_default()
                }
            }
        };

        if body.is_empty() {
            return vec![];
        }

        let releases: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap_or_default();
        let mut result = Vec::new();

        for release in releases {
            let tag = release["tag_name"].as_str().unwrap_or("");
            let release_body = release["body"].as_str().unwrap_or("").to_string();

            if let Some(assets) = release["assets"].as_array() {
                for asset in assets {
                    let name = asset["name"].as_str().unwrap_or("");
                    let url = asset["browser_download_url"].as_str().unwrap_or("");

                    let re = regex::Regex::new(tool.asset_filter).ok();
                    let matches = re.map(|r| r.is_match(name)).unwrap_or(false);

                    if !matches || url.is_empty() {
                        continue;
                    }
                    if let Some(arch) = tool.arch_filter {
                        if !name.contains(arch) {
                            continue;
                        }
                    }

                    let version = Self::extract_version(name, tag);
                    result.push(ReleaseInfo {
                        version,
                        asset_name: name.to_string(),
                        download_url: url.to_string(),
                        body: release_body.clone(),
                    });
                    break;
                }
            }
        }

        result
    }

    fn extract_version(asset_name: &str, tag: &str) -> String {
        for prefix in &["GE-Proton", "proton-cachyos", "dxvk", "vkd3d-proton"] {
            if let Some(rest) = asset_name.strip_prefix(prefix) {
                let v = rest
                    .trim_start_matches('-')
                    .trim_end_matches(".tar.gz")
                    .trim_end_matches(".tar.zst")
                    .trim_end_matches(".tar.xz");
                if !v.is_empty() {
                    return format!("{}-{}", prefix, v);
                }
            }
        }
        tag.trim_start_matches('v').to_string()
    }

    pub fn list_installed(tool: &CompatTool) -> Vec<InstalledVersion> {
        let dirs = Self::steam_install_dirs();
        let mut result = Vec::new();

        for dir in &dirs {
            if !dir.exists() {
                continue;
            }
            let entries = match std::fs::read_dir(dir) {
                Ok(e) => e,
                _ => continue,
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    if (tool.match_fn)(&name) {
                        result.push(InstalledVersion {
                            version: name,
                            path,
                        });
                    }
                }
            }
        }

        result
    }

    #[allow(dead_code)]
    pub fn install_version(
        tool: &CompatTool,
        release: &ReleaseInfo,
    ) -> Result<String, String> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let install_base = if tool.install_subdir.is_empty() {
            PathBuf::from(&home).join(".local/share/lutris/runtime")
        } else {
            let dirs = Self::steam_install_dirs();
            dirs.first().cloned().unwrap_or_else(|| {
                let steam_path = PathBuf::from(&home)
                    .join(".steam/steam/compatibilitytools.d");
                std::fs::create_dir_all(&steam_path).ok();
                steam_path
            })
        };

        std::fs::create_dir_all(&install_base).map_err(|e| e.to_string())?;

        let tmp_dir = std::env::var("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(&home).join(".cache"))
            .join("gamehub-proton");
        std::fs::create_dir_all(&tmp_dir).ok();
        let archive_path = tmp_dir.join(&release.asset_name);

        let status = std::process::Command::new("curl")
            .args([
                "-fsSL",
                "-o",
                &archive_path.to_string_lossy(),
                &release.download_url,
                "--max-time",
                "120",
            ])
            .status()
            .ok();

        match status {
            Some(s) if s.success() => {}
            _ => {
                let _ = std::fs::remove_file(&archive_path);
                return Err("Download failed".to_string());
            }
        }

        let decompress_flag = if release.asset_name.ends_with(".zst") {
            "--zstd"
        } else if release.asset_name.ends_with(".xz") {
            "--xz"
        } else {
            "-z"
        };

        let extract_status = std::process::Command::new("tar")
            .args([
                "-xf",
                &archive_path.to_string_lossy(),
                "-C",
                &install_base.to_string_lossy(),
                decompress_flag,
            ])
            .status()
            .ok();

        let _ = std::fs::remove_file(&archive_path);

        match extract_status {
            Some(s) if s.success() => Ok(format!(
                "Installed {} to {}",
                release.version,
                install_base.display()
            )),
            _ => Err("Extraction failed".to_string()),
        }
    }

    #[allow(dead_code)]
    pub fn remove_version(path: &PathBuf) -> Result<String, String> {
        if path.exists() {
            std::fs::remove_dir_all(path).map_err(|e| e.to_string())?;
            Ok(format!("Removed {}", path.display()))
        } else {
            Err("Directory not found".to_string())
        }
    }
}
