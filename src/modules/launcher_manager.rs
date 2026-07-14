use std::path::PathBuf;

use crate::modules::pacman::PacmanManager;

pub struct LauncherInfo {
    pub id: String,
    pub name: &'static str,
    pub packages: &'static [&'static str],
    pub command: &'static str,
    pub icon_url: &'static str,
    pub description: &'static str,
}

pub struct LauncherManager {
    pacman: PacmanManager,
    cache_dir: PathBuf,
}

impl LauncherManager {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let cache_dir = PathBuf::from(home).join(".local/share/gamehub/icons");
        std::fs::create_dir_all(&cache_dir).ok();
        Self {
            pacman: PacmanManager::new(),
            cache_dir,
        }
    }

    pub fn all_launchers(&self) -> Vec<LauncherInfo> {
        vec![
            LauncherInfo {
                id: "steam".into(),
                name: "Steam",
                packages: &["steam", "steam-native-runtime"],
                command: "steam",
                icon_url: "https://upload.wikimedia.org/wikipedia/commons/8/83/Steam_icon_logo.svg",
                description: "The ultimate entertainment platform",
            },
            LauncherInfo {
                id: "lutris".into(),
                name: "Lutris",
                packages: &["lutris"],
                command: "lutris",
                icon_url: "https://raw.githubusercontent.com/lutris/lutris/master/share/icons/hicolor/128x128/apps/net.lutris.Lutris.png",
                description: "Open source gaming platform",
            },
            LauncherInfo {
                id: "heroic-games-launcher-bin".into(),
                name: "Heroic Games Launcher",
                packages: &["heroic-games-launcher-bin"],
                command: "heroic",
                icon_url: "https://raw.githubusercontent.com/Heroic-Games-Launcher/HeroicGamesLauncher/main/flatpak/com.heroicgameslauncher.hgl.png",
                description: "Epic Games, GOG, and Amazon Games launcher",
            },
            LauncherInfo {
                id: "bottles".into(),
                name: "Bottles",
                packages: &["bottles"],
                command: "bottles",
                icon_url: "https://raw.githubusercontent.com/bottlesdevs/Bottles/main/data/icons/hicolor/scalable/apps/com.usebottles.bottles.svg",
                description: "Run Windows software and games",
            },
            LauncherInfo {
                id: "minigalaxy".into(),
                name: "Minigalaxy",
                packages: &["minigalaxy"],
                command: "minigalaxy",
                icon_url: "https://raw.githubusercontent.com/sharkwouter/minigalaxy/master/data/icons/128x128/io.github.sharkwouter.Minigalaxy.png",
                description: "Simple GOG client",
            },
        ]
    }

    pub fn cached_icon_path(&self, info: &LauncherInfo) -> Option<PathBuf> {
        let icon_path = self.cache_dir.join(format!("{}.png", info.id));
        if icon_path.exists() {
            return Some(icon_path);
        }

        let tmp_path = self.cache_dir.join(format!("{}.tmp", info.id));

        let ok = std::process::Command::new("curl")
            .args([
                "-fsSL",
                "-A",
                "Mozilla/5.0 (X11; Linux x86_64)",
                "-o",
                &tmp_path.to_string_lossy(),
                info.icon_url,
                "--max-time",
                "10",
            ])
            .status()
            .ok()
            .map(|s| s.success())
            .unwrap_or(false);

        if !ok || !tmp_path.exists() {
            let _ = std::fs::remove_file(&tmp_path);
            return None;
        }

        let is_svg = std::fs::read(&tmp_path)
            .ok()
            .map(|bytes| bytes.starts_with(b"<svg") || bytes.starts_with(b"<?xml"))
            .unwrap_or(false);

        if is_svg {
            let converted = std::process::Command::new("rsvg-convert")
                .args([
                    "-w",
                    "128",
                    "-h",
                    "128",
                    "-o",
                    &icon_path.to_string_lossy(),
                    &tmp_path.to_string_lossy(),
                ])
                .status()
                .ok()
                .map(|s| s.success())
                .unwrap_or(false);
            let _ = std::fs::remove_file(&tmp_path);
            if converted && icon_path.exists() {
                Some(icon_path)
            } else {
                None
            }
        } else {
            let _ = std::fs::rename(&tmp_path, &icon_path);
            if icon_path.exists() {
                Some(icon_path)
            } else {
                None
            }
        }
    }

    pub fn is_installed(&self, launcher_id: &str) -> bool {
        let launchers = self.all_launchers();
        if let Some(info) = launchers.iter().find(|l| l.id == launcher_id) {
            self.pacman.is_installed(info.packages[0])
        } else {
            self.pacman.is_installed(launcher_id)
        }
    }

    pub fn get_installed(&self) -> Vec<LauncherInfo> {
        self.all_launchers()
            .into_iter()
            .filter(|l| self.is_installed(&l.id))
            .collect()
    }

    pub fn launch(&self, command: &str) -> bool {
        std::process::Command::new(command)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .is_ok()
    }

    pub fn install_command(&self, launcher_id: &str) -> Option<String> {
        let launchers = self.all_launchers();
        launchers
            .iter()
            .find(|l| l.id == launcher_id)
            .map(|info| {
                let packages: Vec<String> = info.packages.iter().map(|s| s.to_string()).collect();
                PacmanManager::install_command(&packages)
            })
    }

    pub fn remove_command(&self, launcher_id: &str) -> Option<String> {
        let launchers = self.all_launchers();
        launchers
            .iter()
            .find(|l| l.id == launcher_id)
            .map(|info| {
                let packages: Vec<String> = info.packages.iter().map(|s| s.to_string()).collect();
                PacmanManager::remove_command(&packages)
            })
    }

    pub fn get_version(&self, launcher_id: &str) -> Option<String> {
        let launchers = self.all_launchers();
        launchers
            .iter()
            .find(|l| l.id == launcher_id)
            .and_then(|info| self.pacman.get_package_version(info.packages[0]))
    }
}
