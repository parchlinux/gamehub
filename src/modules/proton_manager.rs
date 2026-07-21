use std::path::PathBuf;

#[derive(Clone)]
pub struct CompatTool {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub repo: &'static str,
    pub asset_filter: &'static str,
    pub arch_filter: Option<&'static str>,
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

    pub fn list_available_releases(tool: &CompatTool) -> Vec<ReleaseInfo> {
        let url = format!("https://api.github.com/repos/{}/releases?per_page=30", tool.repo);
        let output = std::process::Command::new("curl")
            .args([
                "-sSL",
                "-A",
                "Mozilla/5.0 (X11; Linux x86_64) gamehub",
                "--max-time",
                "15",
                &url,
            ])
            .output()
            .ok();

        let body = match output {
            Some(o) => String::from_utf8_lossy(&o.stdout).to_string(),
            _ => return vec![],
        };

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

        if result.is_empty() && !body.is_empty() {
            if body.contains("API rate limit") || body.contains("rate limit") {
                eprintln!("GitHub API rate limit exceeded: {}", body);
            } else {
                eprintln!("No releases parsed. Response: {}", body);
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

        let tmp_dir = PathBuf::from("/tmp/gamehub-proton");
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

    pub fn remove_version(path: &PathBuf) -> Result<String, String> {
        if path.exists() {
            std::fs::remove_dir_all(path).map_err(|e| e.to_string())?;
            Ok(format!("Removed {}", path.display()))
        } else {
            Err("Directory not found".to_string())
        }
    }
}
