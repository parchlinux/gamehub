use std::process::Command;

pub struct PacmanManager;

impl PacmanManager {
    pub fn new() -> Self {
        Self
    }

    pub fn is_installed(&self, package: &str) -> bool {
        Command::new("/usr/bin/pacman")
            .args(["-Q", package])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn get_installed_packages(&self) -> Vec<String> {
        Command::new("/usr/bin/pacman")
            .arg("-Q")
            .output()
            .ok()
            .map(|o| {
                let stdout = String::from_utf8_lossy(&o.stdout);
                stdout
                    .lines()
                    .filter_map(|line| line.split_whitespace().next().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_package_version(&self, package: &str) -> Option<String> {
        Command::new("/usr/bin/pacman")
            .args(["-Q", package])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    stdout.split_whitespace().nth(1).map(String::from)
                } else {
                    None
                }
            })
    }

    pub fn check_chaotic_aur(&self) -> bool {
        if self.is_installed("chaotic-mirrorlist") {
            return true;
        }
        std::fs::read_to_string("/etc/pacman.conf")
            .map(|content| content.contains("chaotic-aur"))
            .unwrap_or(false)
    }

    pub fn install_command(packages: &[String]) -> String {
        let pkgs = packages.join(" ");
        format!("pkexec pacman -S --noconfirm {}", pkgs)
    }

    pub fn remove_command(packages: &[String]) -> String {
        let pkgs = packages.join(" ");
        format!("pkexec pacman -Rns --noconfirm {}", pkgs)
    }
}
