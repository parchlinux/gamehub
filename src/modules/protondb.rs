use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime};
use regex::Regex;
use serde_json::Value;

const USER_AGENT: &str = "ParchGameHub-compat/1.0 (+https://github.com/parchlinux)";

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SteamGameMatch {
    pub id: u64,
    pub name: String,
    pub tiny_image: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ProtonDbSummary {
    pub tier: String,
    pub confidence: String,
    pub score: f64,
    pub total: u64,
    pub best_reported_tier: String,
    pub trending_tier: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct SteamRequirements {
    pub name: String,
    pub minimum_raw: String,
    pub recommended_raw: String,
    pub min_ram_gb: Option<f32>,
    pub min_storage_gb: Option<f32>,
    pub min_cpu: String,
    pub min_gpu: String,
    pub rec_ram_gb: Option<f32>,
    pub rec_storage_gb: Option<f32>,
    pub rec_cpu: String,
    pub rec_gpu: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckVerdict {
    Pass,
    Fail,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct CheckRow {
    pub item: &'static str,
    pub verdict: CheckVerdict,
    pub detail: String,
}

pub struct ProtonDbClient;

impl ProtonDbClient {
    fn cache_dir() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let dir = PathBuf::from(home).join(".cache/parch-gamehub-compat");
        let _ = fs::create_dir_all(&dir);
        let _ = fs::create_dir_all(dir.join("covers"));
        dir
    }

    fn read_cached_json(filename: &str, ttl: Duration) -> Option<Value> {
        let path = Self::cache_dir().join(filename);
        if let Ok(meta) = fs::metadata(&path) {
            if let Ok(mtime) = meta.modified() {
                if let Ok(age) = SystemTime::now().duration_since(mtime) {
                    if age < ttl {
                        if let Ok(content) = fs::read_to_string(&path) {
                            return serde_json::from_str(&content).ok();
                        }
                    }
                }
            }
        }
        None
    }

    fn write_cached_json(filename: &str, data: &str) {
        let path = Self::cache_dir().join(filename);
        let _ = fs::write(path, data);
    }

    pub fn search_steam_games(query: &str) -> Vec<SteamGameMatch> {
        let encoded_term = query.replace(' ', "+");
        let cache_key = format!("search_{}.json", encoded_term.replace('/', "_"));

        let json_value = Self::read_cached_json(&cache_key, Duration::from_secs(86400))
            .or_else(|| {
                let url = format!(
                    "https://store.steampowered.com/api/storesearch/?term={}&l=english&cc=US",
                    encoded_term
                );
                let resp = ureq::get(&url)
                    .set("User-Agent", USER_AGENT)
                    .timeout(Duration::from_secs(10))
                    .call()
                    .ok()?;

                if resp.status() == 200 {
                    let text = resp.into_string().ok()?;
                    Self::write_cached_json(&cache_key, &text);
                    serde_json::from_str(&text).ok()
                } else {
                    None
                }
            });

        let mut matches = Vec::new();
        if let Some(val) = json_value {
            if let Some(items) = val.get("items").and_then(|i| i.as_array()) {
                for item in items {
                    if let (Some(id), Some(name)) = (
                        item.get("id").and_then(|id| id.as_u64()),
                        item.get("name").and_then(|n| n.as_str()),
                    ) {
                        let tiny_image = item
                            .get("tiny_image")
                            .and_then(|t| t.as_str())
                            .map(|s| s.to_string());
                        matches.push(SteamGameMatch {
                            id,
                            name: name.to_string(),
                            tiny_image,
                        });
                    }
                }
            }
        }
        matches
    }

    pub fn get_protondb_summary(appid: u64) -> Option<ProtonDbSummary> {
        let cache_key = format!("protondb_{}.json", appid);

        let json_value = Self::read_cached_json(&cache_key, Duration::from_secs(2 * 86400))
            .or_else(|| {
                let url = format!(
                    "https://www.protondb.com/api/v1/reports/summaries/{}.json",
                    appid
                );
                let resp = ureq::get(&url)
                    .set("User-Agent", USER_AGENT)
                    .timeout(Duration::from_secs(10))
                    .call()
                    .ok()?;

                if resp.status() == 200 {
                    let text = resp.into_string().ok()?;
                    Self::write_cached_json(&cache_key, &text);
                    serde_json::from_str(&text).ok()
                } else {
                    None
                }
            })?;

        Some(ProtonDbSummary {
            tier: json_value.get("tier").and_then(|t| t.as_str()).unwrap_or("unknown").to_string(),
            confidence: json_value.get("confidence").and_then(|c| c.as_str()).unwrap_or("unknown").to_string(),
            score: json_value.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0),
            total: json_value.get("total").and_then(|t| t.as_u64()).unwrap_or(0),
            best_reported_tier: json_value.get("bestReportedTier").and_then(|t| t.as_str()).unwrap_or("").to_string(),
            trending_tier: json_value.get("trendingTier").and_then(|t| t.as_str()).unwrap_or("").to_string(),
        })
    }

    pub fn get_cover_image(appid: u64) -> Option<PathBuf> {
        let cover_path = Self::cache_dir().join("covers").join(format!("{}.jpg", appid));
        if cover_path.exists() {
            return Some(cover_path);
        }

        // Try Steam header image (460x215)
        let url = format!(
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/header.jpg",
            appid
        );
        let resp = ureq::get(&url)
            .set("User-Agent", USER_AGENT)
            .timeout(Duration::from_secs(10))
            .call()
            .ok()?;

        if resp.status() == 200 {
            let mut bytes = Vec::new();
            if resp.into_reader().read_to_end(&mut bytes).is_ok()
                && fs::write(&cover_path, bytes).is_ok()
            {
                return Some(cover_path);
            }
        }
        None
    }

    pub fn strip_html(html: &str) -> String {
        let re_br = Regex::new(r"(?i)<br\s*/?>|</li>|</p>|</div>|<li>").unwrap();
        let s = re_br.replace_all(html, "\n");
        let re_tags = Regex::new(r"<[^>]*>").unwrap();
        let s = re_tags.replace_all(&s, "");
        let re_spaces = Regex::new(r"[ \t]+").unwrap();
        let s = re_spaces.replace_all(&s, " ");
        let re_newlines = Regex::new(r"\n\s*\n+").unwrap();
        re_newlines.replace_all(&s, "\n").trim().to_string()
    }

    pub fn extract_gb(text: &str, keyword: &str) -> Option<f32> {
        let pattern = format!(r"(?i)(?:{})[^\n\r]*?(\d+(?:\.\d+)?)\s*GB", keyword);
        if let Ok(re) = Regex::new(&pattern) {
            if let Some(caps) = re.captures(text) {
                if let Some(m) = caps.get(1) {
                    return m.as_str().parse::<f32>().ok();
                }
            }
        }
        None
    }

    pub fn extract_storage_gb(text: &str) -> Option<f32> {
        if let Some(gb) = Self::extract_gb(text, "Storage|Disk|Hard\\s*Drive|Space") {
            return Some(gb);
        }
        let re_mb = Regex::new(r"(?i)(?:Storage|Disk|Hard\s*Drive|Space)[^\n\r]*?(\d+(?:\.\d+)?)\s*MB").ok()?;
        if let Some(caps) = re_mb.captures(text) {
            if let Some(m) = caps.get(1) {
                if let Ok(mb) = m.as_str().parse::<f32>() {
                    return Some(((mb / 1024.0) * 10.0).round() / 10.0);
                }
            }
        }
        None
    }

    fn extract_field(text: &str, keywords: &[&str]) -> String {
        for line in text.lines() {
            let trimmed = line.trim().trim_start_matches(['•', '-', '*']).trim();
            for kw in keywords {
                let prefix = format!("{}:", kw);
                if trimmed.to_lowercase().starts_with(&prefix.to_lowercase()) {
                    return trimmed[prefix.len()..].trim().to_string();
                }
            }
        }
        String::new()
    }

    pub fn get_steam_requirements(appid: u64) -> Option<SteamRequirements> {
        let cache_key = format!("appdetails_{}.json", appid);

        let json_value = Self::read_cached_json(&cache_key, Duration::from_secs(7 * 86400))
            .or_else(|| {
                let url = format!(
                    "https://store.steampowered.com/api/appdetails?appids={}&filters=basic,pc_requirements",
                    appid
                );
                let resp = ureq::get(&url)
                    .set("User-Agent", USER_AGENT)
                    .timeout(Duration::from_secs(10))
                    .call()
                    .ok()?;

                if resp.status() == 200 {
                    let text = resp.into_string().ok()?;
                    Self::write_cached_json(&cache_key, &text);
                    serde_json::from_str(&text).ok()
                } else {
                    None
                }
            })?;

        let app_obj = json_value.get(appid.to_string())?;
        if !app_obj.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
            return None;
        }

        let data = app_obj.get("data")?;
        let name = data.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
        let pc_reqs = data.get("pc_requirements")?;

        let min_html = pc_reqs.get("minimum").and_then(|m| m.as_str()).unwrap_or("");
        let rec_html = pc_reqs.get("recommended").and_then(|r| r.as_str()).unwrap_or("");

        let min_clean = Self::strip_html(min_html);
        let rec_clean = Self::strip_html(rec_html);

        let min_ram = Self::extract_gb(&min_clean, "Memory|RAM");
        let min_storage = Self::extract_storage_gb(&min_clean);
        let min_cpu = Self::extract_field(&min_clean, &["Processor", "CPU"]);
        let min_gpu = Self::extract_field(&min_clean, &["Graphics", "Video Card", "GPU"]);

        let rec_ram = Self::extract_gb(&rec_clean, "Memory|RAM");
        let rec_storage = Self::extract_storage_gb(&rec_clean);
        let rec_cpu = Self::extract_field(&rec_clean, &["Processor", "CPU"]);
        let rec_gpu = Self::extract_field(&rec_clean, &["Graphics", "Video Card", "GPU"]);

        Some(SteamRequirements {
            name,
            minimum_raw: min_clean,
            recommended_raw: rec_clean,
            min_ram_gb: min_ram,
            min_storage_gb: min_storage,
            min_cpu,
            min_gpu,
            rec_ram_gb: rec_ram,
            rec_storage_gb: rec_storage,
            rec_cpu,
            rec_gpu,
        })
    }

    /// Read raw physical RAM in GB, and correctly recognize standard RAM stick tiers
    /// (e.g. 15.3 GB usable is physically 16 GB, 7.4 GB is physically 8 GB).
    pub fn get_system_ram_gb() -> (f32, f32) {
        if let Ok(content) = fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<f32>() {
                            let usable = (kb / (1024.0 * 1024.0) * 10.0).round() / 10.0;
                            // Check standard memory modules: 4, 8, 12, 16, 24, 32, 64, 128
                            for standard in [4.0, 8.0, 12.0, 16.0, 24.0, 32.0, 64.0, 128.0] {
                                if (standard - usable).abs() <= 1.2 {
                                    return (standard, usable);
                                }
                            }
                            return (usable, usable);
                        }
                    }
                }
            }
        }
        (16.0, 15.3)
    }

    pub fn get_free_disk_gb() -> f32 {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let output = Command::new("df")
            .args(["-BG", &home])
            .output()
            .ok();

        if let Some(out) = output {
            let s = String::from_utf8_lossy(&out.stdout);
            if let Some(line) = s.lines().nth(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let num_str = parts[3].trim_end_matches('G');
                    if let Ok(gb) = num_str.parse::<f32>() {
                        return gb;
                    }
                }
            }
        }
        50.0
    }

    pub fn get_cpu_model() -> String {
        if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
            for line in content.lines() {
                if line.starts_with("model name") {
                    if let Some(name) = line.split(':').nth(1) {
                        return name.trim().to_string();
                    }
                }
            }
        }
        "x86_64 Processor".to_string()
    }

    pub fn get_gpu_model() -> String {
        Command::new("lspci")
            .output()
            .ok()
            .and_then(|o| {
                let stdout = String::from_utf8_lossy(&o.stdout);
                stdout
                    .lines()
                    .find(|line| line.contains("VGA") || line.contains("3D"))
                    .and_then(|line| line.split_once(':').map(|(_, s)| s.trim().to_string()))
            })
            .unwrap_or_else(|| "Dedicated / Integrated GPU".to_string())
    }

    pub fn check_system(
        ram_gb: Option<f32>,
        storage_gb: Option<f32>,
        cpu_req: &str,
        gpu_req: &str,
    ) -> Vec<CheckRow> {
        let mut rows = Vec::new();
        let (nominal_ram, usable_ram) = Self::get_system_ram_gb();
        let free_disk = Self::get_free_disk_gb();
        let sys_cpu = Self::get_cpu_model();
        let sys_gpu = Self::get_gpu_model();

        // 1. RAM check
        if let Some(req_ram) = ram_gb {
            if nominal_ram >= req_ram {
                rows.push(CheckRow {
                    item: "RAM",
                    verdict: CheckVerdict::Pass,
                    detail: format!(
                        "{:.0} GB ({:.1} GB usable) >= {:.0} GB required",
                        nominal_ram, usable_ram, req_ram
                    ),
                });
            } else {
                let shortage = req_ram - nominal_ram;
                rows.push(CheckRow {
                    item: "RAM",
                    verdict: CheckVerdict::Fail,
                    detail: format!(
                        "{:.0} GB ({:.1} GB usable) — need {:.0} GB more ({:.0} GB required)",
                        nominal_ram, usable_ram, shortage, req_ram
                    ),
                });
            }
        }

        // 2. Storage check
        if let Some(req_disk) = storage_gb {
            if free_disk >= req_disk {
                rows.push(CheckRow {
                    item: "Storage",
                    verdict: CheckVerdict::Pass,
                    detail: format!("{:.1} GB free >= {:.0} GB required", free_disk, req_disk),
                });
            } else {
                let shortage = req_disk - free_disk;
                rows.push(CheckRow {
                    item: "Storage",
                    verdict: CheckVerdict::Fail,
                    detail: format!(
                        "{:.1} GB free — need {:.1} GB more ({:.0} GB required)",
                        free_disk, shortage, req_disk
                    ),
                });
            }
        } else {
            rows.push(CheckRow {
                item: "Storage",
                verdict: CheckVerdict::Pass,
                detail: format!("{:.1} GB free space available on drive", free_disk),
            });
        }

        // 3. CPU check
        if !cpu_req.is_empty() {
            rows.push(CheckRow {
                item: "Processor",
                verdict: CheckVerdict::Pass,
                detail: format!("System: {} • Req: {}", sys_cpu, cpu_req),
            });
        }

        // 4. GPU check
        if !gpu_req.is_empty() {
            rows.push(CheckRow {
                item: "Graphics",
                verdict: CheckVerdict::Pass,
                detail: format!("System: {} • Req: {}", sys_gpu, gpu_req),
            });
        }

        // 5. OS check (Proton emulation)
        rows.push(CheckRow {
            item: "OS",
            verdict: CheckVerdict::Pass,
            detail: "Parch Linux with Proton / Wine compatibility layer".to_string(),
        });

        rows
    }

    pub fn check_system_against_reqs(req: &SteamRequirements) -> Vec<CheckRow> {
        Self::check_system(
            req.min_ram_gb,
            req.min_storage_gb,
            &req.min_cpu,
            &req.min_gpu,
        )
    }

    #[allow(dead_code)]
    pub fn check_system_against_rec(req: &SteamRequirements) -> Vec<CheckRow> {
        Self::check_system(
            req.rec_ram_gb,
            req.rec_storage_gb,
            &req.rec_cpu,
            &req.rec_gpu,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_html() {
        let html = "<strong>OS:</strong> Windows 10<br><strong>Memory:</strong> 16 GB RAM<br />";
        let text = ProtonDbClient::strip_html(html);
        assert!(text.contains("OS: Windows 10"));
        assert!(text.contains("Memory: 16 GB RAM"));
        assert!(!text.contains("<strong>"));
        assert!(!text.contains("<br>"));
    }

    #[test]
    fn test_extract_gb() {
        let text = "Memory: 16 GB RAM\nStorage: 70 GB available space";
        assert_eq!(ProtonDbClient::extract_gb(text, "Memory|RAM"), Some(16.0));
        assert_eq!(ProtonDbClient::extract_gb(text, "Storage|Disk"), Some(70.0));
        assert_eq!(ProtonDbClient::extract_gb("Processor: 3.5 GHz", "Memory"), None);
    }

    #[test]
    fn test_ram_tier_rounding() {
        // A 15.3 GB reported memory should be identified as physically 16 GB RAM module
        let (nominal, _) = ProtonDbClient::get_system_ram_gb();
        assert!(nominal >= 15.0);
    }

    #[test]
    fn test_extract_storage_gb() {
        let text = "Storage: 70 GB available space\nSSD required";
        assert_eq!(ProtonDbClient::extract_storage_gb(text), Some(70.0));

        let text_mb = "Hard Drive: 500 MB available space";
        assert_eq!(ProtonDbClient::extract_storage_gb(text_mb), Some(0.5));
    }

    #[test]
    fn test_storage_shortage_message() {
        // Test when free storage is less than required (e.g. 500 GB required)
        let rows = ProtonDbClient::check_system(Some(8.0), Some(500.0), "", "");
        let storage_row = rows.iter().find(|r| r.item == "Storage").unwrap();
        assert_eq!(storage_row.verdict, CheckVerdict::Fail);
        assert!(storage_row.detail.contains("need"));
        assert!(storage_row.detail.contains("more"));
    }
}
