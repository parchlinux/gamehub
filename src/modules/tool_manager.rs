use crate::modules::pacman::PacmanManager;

pub struct ToolInfo {
    pub id: String,
    pub name: &'static str,
    pub packages: &'static [&'static str],
    pub description: &'static str,
    pub category: ToolCategory,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolCategory {
    WineCompatibility,
    Performance,
    Additional,
    ChaoticAur,
}

pub struct ToolManager {
    pacman: PacmanManager,
}

impl ToolManager {
    pub fn new() -> Self {
        Self {
            pacman: PacmanManager::new(),
        }
    }

    pub fn all_tools(&self) -> Vec<ToolInfo> {
        vec![
            ToolInfo {
                id: "wine".into(),
                name: "Wine",
                packages: &["wine"],
                description: "Run Windows applications on Linux",
                category: ToolCategory::WineCompatibility,
            },
            ToolInfo {
                id: "proton-ge-custom-bin".into(),
                name: "Proton-GE",
                packages: &["proton-ge-custom-bin"],
                description: "Custom Proton build for better game compatibility",
                category: ToolCategory::WineCompatibility,
            },
            ToolInfo {
                id: "wine-ge-custom".into(),
                name: "Wine-GE",
                packages: &["wine-ge-custom"],
                description: "GloriousEggroll's Wine build",
                category: ToolCategory::WineCompatibility,
            },
            ToolInfo {
                id: "winetricks".into(),
                name: "Winetricks",
                packages: &["winetricks"],
                description: "Install Windows components in Wine",
                category: ToolCategory::WineCompatibility,
            },
            ToolInfo {
                id: "mangohud".into(),
                name: "MangoHud",
                packages: &["mangohud"],
                description: "Vulkan and OpenGL overlay for monitoring FPS, CPU, GPU",
                category: ToolCategory::Performance,
            },
            ToolInfo {
                id: "gamemode".into(),
                name: "GameMode",
                packages: &["gamemode"],
                description: "Optimize system performance for gaming",
                category: ToolCategory::Performance,
            },
            ToolInfo {
                id: "goverlay".into(),
                name: "GOverlay",
                packages: &["goverlay"],
                description: "Graphical UI to configure MangoHud",
                category: ToolCategory::Performance,
            },
            ToolInfo {
                id: "corectrl".into(),
                name: "CoreCtrl",
                packages: &["corectrl"],
                description: "Control your CPU and GPU for better performance",
                category: ToolCategory::Performance,
            },
            ToolInfo {
                id: "discord".into(),
                name: "Discord",
                packages: &["discord"],
                description: "Voice and text chat for gamers",
                category: ToolCategory::Additional,
            },
            ToolInfo {
                id: "obs-studio".into(),
                name: "OBS Studio",
                packages: &["obs-studio"],
                description: "Stream and record your gameplay",
                category: ToolCategory::Additional,
            },
            ToolInfo {
                id: "protonup-qt".into(),
                name: "ProtonUp-Qt",
                packages: &["protonup-qt"],
                description: "Install and manage Proton-GE versions",
                category: ToolCategory::Additional,
            },
        ]
    }

    pub fn is_installed(&self, tool_id: &str) -> bool {
        let tools = self.all_tools();
        if let Some(info) = tools.iter().find(|t| t.id == tool_id) {
            self.pacman.is_installed(info.packages[0])
        } else {
            self.pacman.is_installed(tool_id)
        }
    }

    pub fn get_installed_by_category(&self, category: &ToolCategory) -> Vec<ToolInfo> {
        self.all_tools()
            .into_iter()
            .filter(|t| &t.category == category && self.is_installed(&t.id))
            .collect()
    }

    pub fn check_chaotic_aur(&self) -> bool {
        self.pacman.check_chaotic_aur()
    }

    pub fn install_command(&self, tool_id: &str) -> Option<String> {
        let tools = self.all_tools();
        tools
            .iter()
            .find(|t| t.id == tool_id)
            .map(|info| {
                let packages: Vec<String> = info.packages.iter().map(|s| s.to_string()).collect();
                PacmanManager::install_command(&packages)
            })
    }

    pub fn remove_command(&self, tool_id: &str) -> Option<String> {
        let tools = self.all_tools();
        tools
            .iter()
            .find(|t| t.id == tool_id)
            .map(|info| {
                let packages: Vec<String> = info.packages.iter().map(|s| s.to_string()).collect();
                PacmanManager::remove_command(&packages)
            })
    }

    pub fn get_version(&self, tool_id: &str) -> Option<String> {
        let tools = self.all_tools();
        tools
            .iter()
            .find(|t| t.id == tool_id)
            .and_then(|info| self.pacman.get_package_version(info.packages[0]))
    }
}
