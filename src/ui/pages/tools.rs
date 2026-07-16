use adw::prelude::*;

use crate::modules::launcher_manager::apply_locale;
use crate::modules::tool_manager::{ToolCategory, ToolManager};
use crate::ui::dialogs::terminal;

pub fn new() -> gtk::Box {
    let manager = ToolManager::new();

    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .build();

    let clamp = adw::Clamp::builder()
        .maximum_size(800)
        .tightening_threshold(600)
        .build();

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(24)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(12)
        .margin_end(12)
        .build();

    // Wine &amp; Compatibility
    let wine_group = create_tool_category_group(
        &manager,
        ToolCategory::WineCompatibility,
        "Wine &amp; Compatibility",
        "Windows compatibility layers",
    );
    content.append(&wine_group);

    // Performance Tools
    let perf_group = create_tool_category_group(
        &manager,
        ToolCategory::Performance,
        "Performance Tools",
        "Gaming performance utilities",
    );
    content.append(&perf_group);

    // Additional Tools
    let add_group = create_tool_category_group(&manager, ToolCategory::Additional, "Additional Tools", "");
    content.append(&add_group);

    // Chaotic AUR
    let chaotic_group = create_chaotic_group(&manager);
    content.append(&chaotic_group);

    clamp.set_child(Some(&content));
    scroll.set_child(Some(&clamp));

    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    container.append(&scroll);
    container
}

fn create_tool_category_group(
    manager: &ToolManager,
    category: ToolCategory,
    title: &str,
    description: &str,
) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title(title)
        .description(description)
        .build();

    for tool in manager.all_tools().into_iter().filter(|t| t.category == category) {
        let row = adw::ActionRow::builder()
            .title(tool.name)
            .subtitle(tool.description)
            .build();
        row.add_prefix(&gtk::Image::from_icon_name("application-x-executable"));

        let installed = manager.is_installed(&tool.id);

        if installed {
            row.set_subtitle(&format!("{} • Installed", tool.description));

            let remove_btn = gtk::Button::builder()
                .icon_name("user-trash-symbolic")
                .valign(gtk::Align::Center)
                .tooltip_text("Uninstall")
                .build();
            let pkg_list: Vec<String> = tool.packages.iter().map(|s| s.to_string()).collect();
            let cmd = crate::modules::pacman::PacmanManager::remove_command(&pkg_list);
            let title = tool.name.to_string();
            remove_btn.connect_clicked(move |btn| {
                if let Some(root) = btn.root() {
                    if let Some(window) = root.downcast_ref::<gtk::Window>() {
                        terminal::show(window, &title, &cmd, None);
                    }
                }
            });
            row.add_suffix(&remove_btn);
        } else {
            let install_btn = gtk::Button::builder()
                .label("Install")
                .valign(gtk::Align::Center)
                .css_classes(["suggested-action"])
                .build();
            let pkg_list: Vec<String> = tool.packages.iter().map(|s| s.to_string()).collect();
            let cmd = crate::modules::pacman::PacmanManager::install_command(&pkg_list);
            let title = tool.name.to_string();
            install_btn.connect_clicked(move |btn| {
                if let Some(root) = btn.root() {
                    if let Some(window) = root.downcast_ref::<gtk::Window>() {
                        terminal::show(window, &title, &cmd, None);
                    }
                }
            });
            row.add_suffix(&install_btn);
        }

        group.add(&row);
    }

    group
}

fn create_chaotic_group(manager: &ToolManager) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Chaotic AUR")
        .description("Access to pre-built AUR packages")
        .build();

    let installed = manager.check_chaotic_aur();

    let row = adw::ActionRow::builder()
        .title("Chaotic AUR Repository")
        .subtitle(if installed { "Installed" } else { "Not installed" })
        .build();
    row.add_prefix(&gtk::Image::from_icon_name("package"));

    if !installed {
        let open_btn = gtk::Button::builder()
            .label("Open Parch Repository Manager")
            .valign(gtk::Align::Center)
            .css_classes(["suggested-action"])
            .tooltip_text("Launch Parch Repository Manager (mirrorman) to enable Chaotic AUR")
            .build();
        open_btn.connect_clicked(move |_| {
            let mut child = std::process::Command::new("mirrorman");
            child.stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());
            apply_locale(&mut child);
            child.spawn().ok();
        });
        row.add_suffix(&open_btn);
    } else {
        let status_icon = gtk::Image::from_icon_name("emblem-ok-symbolic");
        status_icon.set_valign(gtk::Align::Center);
        row.add_suffix(&status_icon);
    }

    group.add(&row);
    group
}
