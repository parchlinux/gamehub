use std::path::PathBuf;

use crate::modules::pacman::PacmanManager;

pub fn apply_locale(cmd: &mut std::process::Command) {
    if let Ok(val) = std::env::var("LANG") {
        cmd.env("LANG", val);
    }
    if let Ok(val) = std::env::var("LANGUAGE") {
        let primary = val.split(':').next().filter(|s| !s.is_empty());
        if let Some(primary) = primary {
            cmd.env("LANGUAGE", primary);
        } else {
            cmd.env_remove("LANGUAGE");
        }
    }
    if let Ok(val) = std::env::var("LC_MESSAGES") {
        cmd.env("LC_MESSAGES", val);
    }
}

#[derive(Clone, Debug)]
pub struct LauncherInfo {
    pub id: String,
    pub name: &'static str,
    pub packages: &'static [&'static str],
    pub command: &'static str,
    pub icon_url: &'static str,
    pub description: &'static str,
    pub chaotic_aur: bool,
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
                packages: &["steam"],
                command: "steam",
                icon_url: "https://upload.wikimedia.org/wikipedia/commons/8/83/Steam_icon_logo.svg",
                description: "The ultimate entertainment platform",
                chaotic_aur: false,
            },
            LauncherInfo {
                id: "lutris".into(),
                name: "Lutris",
                packages: &["lutris"],
                command: "lutris",
                icon_url: "https://raw.githubusercontent.com/lutris/lutris/master/share/icons/hicolor/128x128/apps/net.lutris.Lutris.png",
                description: "Open source gaming platform",
                chaotic_aur: false,
            },
            LauncherInfo {
                id: "heroic-games-launcher-bin".into(),
                name: "Heroic Games Launcher",
                packages: &["heroic-games-launcher-bin"],
                command: "heroic",
                icon_url: "https://raw.githubusercontent.com/Heroic-Games-Launcher/HeroicGamesLauncher/main/flatpak/com.heroicgameslauncher.hgl.png",
                description: "Epic Games, GOG, and Amazon Games launcher",
                chaotic_aur: true,
            },
            LauncherInfo {
                id: "bottles".into(),
                name: "Bottles",
                packages: &["bottles"],
                command: "bottles",
                icon_url: "https://raw.githubusercontent.com/bottlesdevs/Bottles/main/data/icons/hicolor/scalable/apps/com.usebottles.bottles.svg",
                description: "Run Windows software and games",
                chaotic_aur: true,
            },
            LauncherInfo {
                id: "minigalaxy".into(),
                name: "Minigalaxy",
                packages: &["minigalaxy"],
                command: "minigalaxy",
                icon_url: "https://raw.githubusercontent.com/sharkwouter/minigalaxy/master/data/icons/128x128/io.github.sharkwouter.Minigalaxy.png",
                description: "Simple GOG client",
                chaotic_aur: true,
            },
            LauncherInfo {
                id: "retroarch".into(),
                name: "RetroArch",
                packages: &["retroarch"],
                command: "retroarch",
                icon_url: "https://raw.githubusercontent.com/libretro/RetroArch/master/media/icons/128x128/retroarch.png",
                description: "Cross-platform multi-system emulation game engine",
                chaotic_aur: false,
            },
            LauncherInfo {
                id: "prismlauncher".into(),
                name: "Prism Launcher",
                packages: &["prismlauncher"],
                command: "prismlauncher",
                icon_url: "https://raw.githubusercontent.com/PrismLauncher/PrismLauncher/develop/launcher/resources/multimc/128x128/apps/org.prismlauncher.PrismLauncher.png",
                description: "Modern open source Minecraft launcher and engine",
                chaotic_aur: false,
            },
            LauncherInfo {
                id: "cartridges".into(),
                name: "Cartridges",
                packages: &["cartridges"],
                command: "cartridges",
                icon_url: "https://raw.githubusercontent.com/kirbyfan64/cartridges/main/data/icons/page.kramo.Cartridges.svg",
                description: "Unified game library for all your launchers",
                chaotic_aur: false,
            },
            LauncherInfo {
                id: "parchdroid".into(),
                name: "Parchdroid",
                packages: &["parchdroid"],
                command: "parchdroid",
                icon_url: "https://raw.githubusercontent.com/parchlinux/parchdroid/main/data/icons/com.parchlinux.parchdroid.svg",
                description: "Android game and application manager for Parch Linux",
                chaotic_aur: false,
            },
            LauncherInfo {
                id: "itch-setup-bin".into(),
                name: "itch.io",
                packages: &["itch-setup-bin"],
                command: "itch-setup",
                icon_url: "https://raw.githubusercontent.com/itchio/itch/master/release/resources/itch.png",
                description: "Indie game marketplace and launcher",
                chaotic_aur: true,
            },
        ]
    }

    pub fn existing_icon_path(&self, id: &str) -> Option<PathBuf> {
        let cached = self.cache_dir.join(format!("{}.png", id));
        if cached.exists() {
            return Some(cached);
        }

        // Check system local icons
        let system_paths = [
            format!("/usr/share/icons/hicolor/scalable/apps/com.parchlinux.{}.svg", id),
            format!("/usr/share/icons/hicolor/scalable/apps/{}.svg", id),
            format!("/usr/share/icons/hicolor/128x128/apps/{}.png", id),
            format!("/usr/share/pixmaps/{}.png", id),
            format!("/usr/share/pixmaps/{}.svg", id),
        ];

        for path_str in system_paths {
            let p = PathBuf::from(path_str);
            if p.exists() {
                return Some(p);
            }
        }

        None
    }

    pub fn download_icon_by_url(&self, id: &str, icon_url: &str) -> Option<PathBuf> {
        if let Some(existing) = self.existing_icon_path(id) {
            return Some(existing);
        }

        let icon_path = self.cache_dir.join(format!("{}.png", id));
        let tmp_path = self.cache_dir.join(format!("{}.tmp", id));

        let user_agent = format!("ParchGameHub/{}", env!("CARGO_PKG_VERSION"));
        let resp = ureq::get(icon_url)
            .set("User-Agent", &user_agent)
            .timeout(std::time::Duration::from_secs(8))
            .call()
            .ok()?;

        if resp.status() != 200 {
            return None;
        }

        let mut bytes = Vec::new();
        if std::io::Read::read_to_end(&mut resp.into_reader(), &mut bytes).is_err() {
            return None;
        }

        if std::fs::write(&tmp_path, &bytes).is_err() {
            return None;
        }

        let is_svg = bytes.starts_with(b"<svg") || bytes.starts_with(b"<?xml");

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

    #[allow(dead_code)]
    pub fn cached_icon_path(&self, info: &LauncherInfo) -> Option<PathBuf> {
        self.existing_icon_path(&info.id)
            .or_else(|| self.download_icon_by_url(&info.id, info.icon_url))
    }

    pub fn is_installed(&self, launcher_id: &str) -> bool {
        let launchers = self.all_launchers();
        if let Some(info) = launchers.iter().find(|l| l.id == launcher_id) {
            self.pacman.is_installed(info.packages[0])
        } else {
            self.pacman.is_installed(launcher_id)
        }
    }

    #[allow(dead_code)]
    pub fn get_installed(&self) -> Vec<LauncherInfo> {
        self.all_launchers()
            .into_iter()
            .filter(|l| self.is_installed(&l.id))
            .collect()
    }

    #[allow(dead_code)]
    pub fn launch(&self, command: &str) -> bool {
        let mut cmd = std::process::Command::new(command);
        cmd.stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        apply_locale(&mut cmd);
        cmd.spawn().is_ok()
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn get_version(&self, launcher_id: &str) -> Option<String> {
        let launchers = self.all_launchers();
        launchers
            .iter()
            .find(|l| l.id == launcher_id)
            .and_then(|info| self.pacman.get_package_version(info.packages[0]))
    }
}
