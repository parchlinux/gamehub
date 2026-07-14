use std::path::PathBuf;

use adw::prelude::*;

use crate::modules::launcher_manager::{LauncherInfo, LauncherManager};
use crate::ui::dialogs::terminal;

pub fn new() -> gtk::Box {
    let manager = LauncherManager::new();

    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .build();

    let clamp = adw::Clamp::builder()
        .maximum_size(1000)
        .tightening_threshold(600)
        .build();

    let flowbox = gtk::FlowBox::builder()
        .max_children_per_line(6)
        .min_children_per_line(2)
        .selection_mode(gtk::SelectionMode::None)
        .homogeneous(true)
        .column_spacing(12)
        .row_spacing(12)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(12)
        .margin_end(12)
        .build();

    for info in manager.all_launchers() {
        let icon_path = manager.cached_icon_path(&info);
        let card = create_launcher_card(info, icon_path);
        flowbox.append(&card);
    }

    clamp.set_child(Some(&flowbox));
    scroll.set_child(Some(&clamp));

    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    container.append(&scroll);
    container
}

fn create_launcher_card(info: LauncherInfo, icon_path: Option<PathBuf>) -> gtk::Box {
    let installed = LauncherManager::new().is_installed(&info.id);

    let icon: gtk::Image = if let Some(ref path) = icon_path {
        gtk::Image::builder()
            .file(path.to_string_lossy().as_ref())
            .pixel_size(64)
            .build()
    } else {
        gtk::Image::builder()
            .icon_name("applications-games")
            .pixel_size(64)
            .build()
    };

    let name_label = gtk::Label::builder()
        .label(info.name)
        .css_classes(["heading"])
        .wrap(true)
        .build();

    let desc_label = gtk::Label::builder()
        .label(info.description)
        .css_classes(["caption"])
        .wrap(true)
        .lines(2)
        .build();

    let status_label = gtk::Label::builder()
        .label(if installed {
            "Installed"
        } else {
            "Not installed"
        })
        .css_classes(["caption"])
        .build();

    let action_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .halign(gtk::Align::Center)
        .spacing(6)
        .build();

    let pkg_list: Vec<String> = info.packages.iter().map(|s| s.to_string()).collect();

    if installed {
        let launch_btn = gtk::Button::builder()
            .label("Launch")
            .css_classes(["suggested-action", "compact"])
            .build();
        let cmd = info.command.to_string();
        launch_btn.connect_clicked(move |_| {
            std::process::Command::new(&cmd)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .ok();
        });
        action_box.append(&launch_btn);

        let remove_btn = gtk::Button::builder()
            .label("Remove")
            .css_classes(["destructive-action", "compact"])
            .build();
        let remove_cmd = crate::modules::pacman::PacmanManager::remove_command(&pkg_list);
        let remove_title = info.name.to_string();
        remove_btn.connect_clicked(move |btn| {
            if let Some(root) = btn.root() {
                if let Some(window) = root.downcast_ref::<gtk::Window>() {
                    terminal::show(window, &remove_title, &remove_cmd, None);
                }
            }
        });
        action_box.append(&remove_btn);
    } else {
        let install_btn = gtk::Button::builder()
            .label("Install")
            .css_classes(["suggested-action", "compact"])
            .build();
        let install_cmd = crate::modules::pacman::PacmanManager::install_command(&pkg_list);
        let install_title = info.name.to_string();
        install_btn.connect_clicked(move |btn| {
            if let Some(root) = btn.root() {
                if let Some(window) = root.downcast_ref::<gtk::Window>() {
                    terminal::show(window, &install_title, &install_cmd, None);
                }
            }
        });
        action_box.append(&install_btn);
    }

    let inner = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(6)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(12)
        .margin_end(12)
        .valign(gtk::Align::Start)
        .build();

    inner.append(&icon);
    inner.append(&name_label);
    inner.append(&desc_label);
    inner.append(&status_label);
    inner.append(&action_box);

    let card = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .css_classes(["card"])
        .build();
    card.append(&inner);
    card
}
