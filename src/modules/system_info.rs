use std::process::Command;
use crate::modules::pacman::PacmanManager;

pub struct SystemInfo;

#[derive(Debug, Clone, Default)]
pub struct SystemStatus {
    pub kernel: String,
    pub gpu: String,
    pub driver: String,
    pub vulkan_supported: bool,
    pub chaotic_aur_installed: bool,
    #[allow(dead_code)]
    pub kernel_manager_installed: bool,
    #[allow(dead_code)]
    pub driver_manager_installed: bool,
}

#[derive(Debug, Clone)]
pub struct InstalledLauncher {
    pub id: String,
    pub name: Option<String>,
    #[allow(dead_code)]
    pub icon: Option<String>,
    pub command: String,
}

impl SystemInfo {
    pub fn new() -> Self {
        Self
    }

    pub fn get_kernel_info(&self) -> String {
        Command::new("uname")
            .arg("-r")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|| "Linux (unknown)".to_string())
    }

    pub fn get_gpu_info(&self) -> String {
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
            .unwrap_or_else(|| "No GPU detected".to_string())
    }

    pub fn get_driver_info(&self) -> String {
        Command::new("glxinfo")
            .output()
            .ok()
            .and_then(|o| {
                let stdout = String::from_utf8_lossy(&o.stdout);
                stdout
                    .lines()
                    .find(|line| line.contains("OpenGL renderer string"))
                    .and_then(|line| line.split_once(':').map(|(_, s)| s.trim().to_string()))
            })
            .unwrap_or_else(|| "Error detecting driver (install mesa-utils?)".to_string())
    }

    pub fn check_vulkan(&self) -> bool {
        Command::new("vulkaninfo")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn check_chaotic_aur(&self) -> bool {
        PacmanManager::new().check_chaotic_aur()
    }

    pub fn check_kernel_manager(&self) -> bool {
        Command::new("which")
            .arg("parch-kernel-manager")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn check_driver_manager(&self) -> bool {
        Command::new("which")
            .arg("parch-driver-manager")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn gather_all(&self) -> SystemStatus {
        SystemStatus {
            kernel: self.get_kernel_info(),
            gpu: self.get_gpu_info(),
            driver: self.get_driver_info(),
            vulkan_supported: self.check_vulkan(),
            chaotic_aur_installed: self.check_chaotic_aur(),
            kernel_manager_installed: self.check_kernel_manager(),
            driver_manager_installed: self.check_driver_manager(),
        }
    }

    pub fn get_installed_launchers(&self) -> Vec<InstalledLauncher> {
        let launchers = vec![
            ("steam", "Steam", "steam"),
            ("lutris", "Lutris", "lutris"),
            ("heroic-games-launcher-bin", "Heroic Games Launcher", "heroic"),
            ("bottles", "Bottles", "bottles"),
            ("minigalaxy", "Minigalaxy", "minigalaxy"),
            ("retroarch", "RetroArch", "retroarch"),
            ("prismlauncher", "Prism Launcher", "prismlauncher"),
            ("cartridges", "Cartridges", "cartridges"),
            ("parchdroid", "Parchdroid", "parchdroid"),
            ("itch-bin", "itch.io", "itch"),
        ];

        launchers
            .into_iter()
            .filter_map(|(id, name, command)| {
                Command::new("which")
                    .arg(command)
                    .output()
                    .ok()
                    .filter(|o| o.status.success())
                    .map(|_| InstalledLauncher {
                        id: id.to_string(),
                        name: Some(name.to_string()),
                        icon: None,
                        command: command.to_string(),
                    })
            })
            .collect()
    }
}
