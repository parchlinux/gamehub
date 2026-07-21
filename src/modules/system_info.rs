use std::process::Command;
use crate::modules::pacman::PacmanManager;

pub struct SystemInfo;

#[derive(Debug, Clone, Default)]
pub struct SystemStatus {
    pub gpu: String,
    pub driver: String,
    pub vulkan_supported: bool,
    pub chaotic_aur_installed: bool,
}

#[derive(Debug, Clone)]
pub struct InstalledLauncher {
    pub id: String,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub command: String,
}

impl SystemInfo {
    pub fn new() -> Self {
        Self
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
                    .and_then(|line| line.splitn(2, ':').nth(1).map(|s| s.trim().to_string()))
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
                    .and_then(|line| line.splitn(2, ':').nth(1).map(|s| s.trim().to_string()))
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

    pub fn gather_all(&self) -> SystemStatus {
        SystemStatus {
            gpu: self.get_gpu_info(),
            driver: self.get_driver_info(),
            vulkan_supported: self.check_vulkan(),
            chaotic_aur_installed: self.check_chaotic_aur(),
        }
    }

    pub fn get_installed_launchers(&self) -> Vec<InstalledLauncher> {
        let launchers = vec![
            ("steam", "Steam", "steam"),
            ("lutris", "Lutris", "lutris"),
            ("heroic-games-launcher-bin", "Heroic Games Launcher", "heroic"),
            ("bottles", "Bottles", "bottles"),
            ("minigalaxy", "Minigalaxy", "minigalaxy"),
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
