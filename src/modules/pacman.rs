use std::process::Command;

pub struct PacmanManager;

fn is_valid_package_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 256
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '+' | '@'))
}

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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
        let pkgs: Vec<&str> = packages
            .iter()
            .map(|s| s.as_str())
            .filter(|s| is_valid_package_name(s))
            .collect();
        format!("pkexec pacman -S --noconfirm {}", pkgs.join(" "))
    }

    pub fn remove_command(packages: &[String]) -> String {
        let pkgs: Vec<&str> = packages
            .iter()
            .map(|s| s.as_str())
            .filter(|s| is_valid_package_name(s))
            .collect();
        format!("pkexec pacman -Rns --noconfirm {}", pkgs.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_package_names() {
        assert!(is_valid_package_name("steam"));
        assert!(is_valid_package_name("steam-native-runtime"));
        assert!(is_valid_package_name("lib32-vulkan-radeon"));
        assert!(is_valid_package_name("proton_ge_custom.bin"));
        assert!(is_valid_package_name("game+tool@arch"));
    }

    #[test]
    fn test_invalid_package_names() {
        assert!(!is_valid_package_name(""));
        assert!(!is_valid_package_name("pkg; rm -rf /"));
        assert!(!is_valid_package_name("pkg && echo hacked"));
        assert!(!is_valid_package_name("pkg | sh"));
        assert!(!is_valid_package_name("`reboot`"));
        assert!(!is_valid_package_name("$(whoami)"));
    }
}
