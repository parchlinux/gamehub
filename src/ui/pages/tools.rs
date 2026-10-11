use adw::prelude::*;

use crate::modules::launcher_manager::apply_locale;
use crate::modules::tool_manager::{ToolCategory, ToolInfo, ToolManager};
use crate::ui::dialogs::terminal;

pub fn new() -> gtk::Box {
    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .build();

    let clamp = adw::Clamp::builder()
        .maximum_size(850)
        .tightening_threshold(600)
        .build();

    let page_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .build();

    let search_entry = gtk::SearchEntry::builder()
        .placeholder_text("Search gaming tools, drivers, and kernels...")
        .margin_top(16)
        .margin_start(16)
        .margin_end(16)
        .margin_bottom(4)
        .build();

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(24)
        .margin_top(12)
        .margin_bottom(24)
        .margin_start(12)
        .margin_end(12)
        .build();

    let c = content.clone();
    let se = search_entry.clone();
    search_entry.connect_search_changed(move |entry| {
        let text = entry.text().to_string();
        populate_tools(&c, &text, &se);
    });

    populate_tools(&content, "", &search_entry);

    page_box.append(&search_entry);
    page_box.append(&content);

    clamp.set_child(Some(&page_box));
    scroll.set_child(Some(&clamp));

    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    container.append(&scroll);
    container
}

fn populate_tools(content: &gtk::Box, filter: &str, search_entry: &gtk::SearchEntry) {
    while let Some(child) = content.first_child() {
        content.remove(&child);
    }

    let manager = ToolManager::new();
    let content_ref = content.clone();
    let se_ref = search_entry.clone();

    // Drivers and Kernel Management
    let driver_group = create_tool_category_group(
        &manager,
        ToolCategory::DriverKernel,
        "Drivers and Kernel Management",
        "Hardware graphics drivers and gaming kernels for Parch Linux",
        filter,
        {
            let c = content_ref.clone();
            let se = se_ref.clone();
            move || {
                let text = se.text().to_string();
                populate_tools(&c, &text, &se);
            }
        },
    );
    if driver_group.first_child().is_some() {
        content.append(&driver_group);
    }

    // Wine and Compatibility
    let wine_group = create_tool_category_group(
        &manager,
        ToolCategory::WineCompatibility,
        "Wine and Compatibility",
        "Windows compatibility layers and tools",
        filter,
        {
            let c = content_ref.clone();
            let se = se_ref.clone();
            move || {
                let text = se.text().to_string();
                populate_tools(&c, &text, &se);
            }
        },
    );
    if wine_group.first_child().is_some() {
        content.append(&wine_group);
    }

    // Performance Tools
    let perf_group = create_tool_category_group(
        &manager,
        ToolCategory::Performance,
        "Performance Tools",
        "Gaming performance utilities and overlays",
        filter,
        {
            let c = content_ref.clone();
            let se = se_ref.clone();
            move || {
                let text = se.text().to_string();
                populate_tools(&c, &text, &se);
            }
        },
    );
    if perf_group.first_child().is_some() {
        content.append(&perf_group);
    }

    // Additional and Hardware Tools
    let add_group = create_tool_category_group(
        &manager,
        ToolCategory::Additional,
        "Hardware and Additional Tools",
        "Peripherals, RGB lighting, streaming, and game utilities",
        filter,
        {
            let c = content_ref.clone();
            let se = se_ref.clone();
            move || {
                let text = se.text().to_string();
                populate_tools(&c, &text, &se);
            }
        },
    );
    if add_group.first_child().is_some() {
        content.append(&add_group);
    }

    // Chaotic AUR
    if filter.is_empty() || "chaotic aur".contains(&filter.to_lowercase()) {
        let chaotic_group = create_chaotic_group(&manager);
        content.append(&chaotic_group);
    }
}

fn create_tool_category_group<F: Fn() + Clone + 'static>(
    manager: &ToolManager,
    category: ToolCategory,
    title: &str,
    description: &str,
    filter: &str,
    on_refresh: F,
) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title(title)
        .description(description)
        .build();

    let filter_lower = filter.trim().to_lowercase();

    let tools: Vec<ToolInfo> = manager
        .all_tools()
        .into_iter()
        .filter(|t| t.category == category)
        .filter(|t| {
            if filter_lower.is_empty() {
                true
            } else {
                t.name.to_lowercase().contains(&filter_lower)
                    || t.description.to_lowercase().contains(&filter_lower)
            }
        })
        .collect();

    for tool in tools {
        let row = adw::ActionRow::builder()
            .title(tool.name)
            .subtitle(tool.description)
            .build();

        let icon_name = match tool.category {
            ToolCategory::DriverKernel => "preferences-system",
            ToolCategory::WineCompatibility => "application-x-executable",
            ToolCategory::Performance => "emblem-system",
            _ => "applications-utilities",
        };
        row.add_prefix(&gtk::Image::from_icon_name(icon_name));

        let installed = manager.is_installed(&tool.id);

        let subtitle_text = if installed {
            if tool.chaotic_aur {
                format!("{} • Installed [Chaotic-AUR]", tool.description)
            } else {
                format!("{} • Installed", tool.description)
            }
        } else if tool.chaotic_aur {
            format!("{} • [Chaotic-AUR]", tool.description)
        } else {
            tool.description.to_string()
        };
        row.set_subtitle(&subtitle_text);

        if installed {
            let launch_cmd = match tool.id.as_str() {
                "parch-driver-manager" => Some("parch-driver-manager"),
                "parch-kernel-manager" => Some("parch-kernel-manager"),
                "openrgb" => Some("openrgb"),
                "piper" => Some("piper"),
                "goverlay" => Some("goverlay"),
                "corectrl" => Some("corectrl"),
                "discord" => Some("discord"),
                "obs-studio" => Some("obs"),
                "protonup-qt" => Some("protonup-qt"),
                _ => None,
            };

            if let Some(cmd) = launch_cmd {
                let open_btn = gtk::Button::builder()
                    .label("Open")
                    .valign(gtk::Align::Center)
                    .css_classes(["flat", "compact"])
                    .tooltip_text(format!("Launch {}", tool.name))
                    .build();
                let cmd_str = cmd.to_string();
                open_btn.connect_clicked(move |_| {
                    let mut child = std::process::Command::new(&cmd_str);
                    child.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
                    apply_locale(&mut child);
                    if let Ok(mut c) = child.spawn() {
                        std::thread::spawn(move || { let _ = c.wait(); });
                    }
                });
                row.add_suffix(&open_btn);
            }

            let remove_btn = gtk::Button::builder()
                .icon_name("user-trash-symbolic")
                .valign(gtk::Align::Center)
                .tooltip_text("Uninstall")
                .build();
            let pkg_list: Vec<String> = tool.packages.iter().map(|s| s.to_string()).collect();
            let cmd = crate::modules::pacman::PacmanManager::remove_command(&pkg_list);
            let title = tool.name.to_string();
            let refresh = on_refresh.clone();
            remove_btn.connect_clicked(move |btn| {
                if let Some(root) = btn.root() {
                    if let Some(window) = root.downcast_ref::<gtk::Window>() {
                        let ref_cb = refresh.clone();
                        terminal::show(
                            window,
                            &title,
                            &cmd,
                            Some(Box::new(move |success| {
                                if success {
                                    ref_cb();
                                }
                            })),
                        );
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
            let refresh = on_refresh.clone();
            let is_chaotic = tool.chaotic_aur;
            let item_name = tool.name.to_string();
            install_btn.connect_clicked(move |btn| {
                if let Some(root) = btn.root() {
                    if let Some(window) = root.downcast_ref::<gtk::Window>() {
                        if is_chaotic && !crate::modules::pacman::PacmanManager::new().check_chaotic_aur() {
                            crate::ui::dialogs::chaotic::prompt_chaotic_aur(window, &item_name);
                            return;
                        }
                        let ref_cb = refresh.clone();
                        terminal::show(
                            window,
                            &title,
                            &cmd,
                            Some(Box::new(move |success| {
                                if success {
                                    ref_cb();
                                }
                            })),
                        );
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
            if let Ok(mut c) = child.spawn() {
                std::thread::spawn(move || {
                    let _ = c.wait();
                });
            }
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
