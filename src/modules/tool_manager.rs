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
    DriverKernel,
    WineCompatibility,
    Performance,
    Additional,
    #[allow(dead_code)]
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
            // Drivers and Kernel Management
            ToolInfo {
                id: "parch-driver-manager".into(),
                name: "Parch Driver Manager",
                packages: &["pdm"],
                description: "Detect, configure, and manage hardware and GPU drivers",
                category: ToolCategory::DriverKernel,
            },
            ToolInfo {
                id: "parch-kernel-manager".into(),
                name: "Parch Kernel Manager",
                packages: &["parch-kernel-manager"],
                description: "Manage, install, and switch Linux kernels on Parch Linux",
                category: ToolCategory::DriverKernel,
            },
            ToolInfo {
                id: "vulkan-radeon".into(),
                name: "AMD RADV Vulkan",
                packages: &["vulkan-radeon", "lib32-vulkan-radeon"],
                description: "High-performance open source AMD Vulkan driver (recommended)",
                category: ToolCategory::DriverKernel,
            },
            ToolInfo {
                id: "amdvlk".into(),
                name: "AMD AMDVLK Vulkan",
                packages: &["amdvlk", "lib32-amdvlk"],
                description: "AMD official open-source Vulkan driver",
                category: ToolCategory::DriverKernel,
            },
            ToolInfo {
                id: "nvidia-utils".into(),
                name: "NVIDIA Graphics Drivers",
                packages: &["nvidia-utils", "lib32-nvidia-utils"],
                description: "NVIDIA proprietary Vulkan and OpenGL driver utilities",
                category: ToolCategory::DriverKernel,
            },
            ToolInfo {
                id: "vulkan-intel".into(),
                name: "Intel Vulkan (ANV)",
                packages: &["vulkan-intel", "lib32-vulkan-intel"],
                description: "Vulkan driver for Intel Arc, Xe, and HD/UHD graphics",
                category: ToolCategory::DriverKernel,
            },
            ToolInfo {
                id: "linux-zen".into(),
                name: "Linux Zen Kernel",
                packages: &["linux-zen", "linux-zen-headers"],
                description: "Desktop and gaming responsiveness tuned kernel",
                category: ToolCategory::DriverKernel,
            },
            ToolInfo {
                id: "linux-cachyos".into(),
                name: "Linux CachyOS Kernel",
                packages: &["linux-cachyos", "linux-cachyos-headers"],
                description: "Performance gaming kernel with BORE scheduler and patches",
                category: ToolCategory::DriverKernel,
            },

            // Wine and Compatibility
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

            // Performance
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
                packages: &["gamemode", "lib32-gamemode"],
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

            // Additional and Hardware
            ToolInfo {
                id: "openrgb".into(),
                name: "OpenRGB",
                packages: &["openrgb"],
                description: "RGB lighting control for gaming peripherals and motherboards",
                category: ToolCategory::Additional,
            },
            ToolInfo {
                id: "piper".into(),
                name: "Piper",
                packages: &["piper"],
                description: "GTK application to configure gaming mice (DPI, buttons)",
                category: ToolCategory::Additional,
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

    #[allow(dead_code)]
    pub fn get_installed_by_category(&self, category: &ToolCategory) -> Vec<ToolInfo> {
        self.all_tools()
            .into_iter()
            .filter(|t| &t.category == category && self.is_installed(&t.id))
            .collect()
    }

    pub fn check_chaotic_aur(&self) -> bool {
        self.pacman.check_chaotic_aur()
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn get_version(&self, tool_id: &str) -> Option<String> {
        let tools = self.all_tools();
        tools
            .iter()
            .find(|t| t.id == tool_id)
            .and_then(|info| self.pacman.get_package_version(info.packages[0]))
    }
}
