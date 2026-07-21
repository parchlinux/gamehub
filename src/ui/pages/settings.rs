use std::rc::Rc;

use adw::prelude::*;
use gtk::glib;

use crate::config::AppConfig;
use crate::ui::dialogs::terminal;

pub fn new() -> gtk::Box {
    let config = Rc::new(AppConfig::new());

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

    // System group
    let system_group = adw::PreferencesGroup::builder()
        .title("System")
        .build();

    // Auto-update
    let update_row = adw::ActionRow::builder()
        .title("Auto-update game launchers")
        .subtitle("Check for updates on startup")
        .build();
    let update_switch = gtk::Switch::builder()
        .valign(gtk::Align::Center)
        .active(config.auto_update())
        .build();
    let update_config = Rc::clone(&config);
    update_switch.connect_state_set(move |_, state| {
        update_config.set_auto_update(state);
        glib::Propagation::Proceed
    });
    update_row.add_suffix(&update_switch);
    update_row.set_activatable_widget(Some(&update_switch));
    system_group.add(&update_row);

    // Notifications
    let notif_row = adw::ActionRow::builder()
        .title("Show notifications")
        .subtitle("Display system notifications for installs and updates")
        .build();
    let notif_switch = gtk::Switch::builder()
        .valign(gtk::Align::Center)
        .active(config.show_notifications())
        .build();
    let notif_config = Rc::clone(&config);
    notif_switch.connect_state_set(move |_, state| {
        notif_config.set_show_notifications(state);
        glib::Propagation::Proceed
    });
    notif_row.add_suffix(&notif_switch);
    notif_row.set_activatable_widget(Some(&notif_switch));
    system_group.add(&notif_row);

    content.append(&system_group);

    // Gaming Optimizations
    let tweaks_group = adw::PreferencesGroup::builder()
        .title("Gaming Optimizations")
        .description("System tweaks for better gaming performance")
        .build();

    // CPU Governor
    let governor_row = adw::ActionRow::builder()
        .title("Performance CPU Governor")
        .subtitle("Set CPU to performance mode for gaming")
        .build();
    let governor_switch = gtk::Switch::builder()
        .valign(gtk::Align::Center)
        .active(config.performance_cpu_governor())
        .build();
    let governor_config = Rc::clone(&config);
    governor_switch.connect_state_set(move |switch, state| {
        governor_config.set_performance_cpu_governor(state);
        if let Some(window) = switch.root().and_then(|r| r.downcast_ref::<gtk::Window>().cloned()) {
            let cmd = if state {
                "pkexec bash -c 'echo performance | tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor'"
            } else {
                "pkexec bash -c 'echo powersave | tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor'"
            };
            terminal::show(&window, "CPU Governor", cmd, None);
        }
        glib::Propagation::Proceed
    });
    governor_row.add_suffix(&governor_switch);
    governor_row.set_activatable_widget(Some(&governor_switch));
    tweaks_group.add(&governor_row);

    // Kernel Parameters
    let kernel_row = adw::ActionRow::builder()
        .title("Gaming Kernel Parameters")
        .subtitle("Apply recommended kernel parameters")
        .build();
    let kernel_switch = gtk::Switch::builder()
        .valign(gtk::Align::Center)
        .active(config.gaming_kernel_parameters())
        .build();
    let kernel_config = Rc::clone(&config);
    kernel_switch.connect_state_set(move |switch, state| {
        kernel_config.set_gaming_kernel_parameters(state);
        if let Some(window) = switch.root().and_then(|r| r.downcast_ref::<gtk::Window>().cloned()) {
            let cmd = if state {
                "pkexec bash -c 'echo \"vm.max_map_count=2147483642\" > /etc/sysctl.d/99-gaming.conf && sysctl --system'"
            } else {
                "pkexec rm -f /etc/sysctl.d/99-gaming.conf && sudo sysctl --system"
            };
            terminal::show(&window, "Kernel Parameters", cmd, None);
        }
        glib::Propagation::Proceed
    });
    kernel_row.add_suffix(&kernel_switch);
    kernel_row.set_activatable_widget(Some(&kernel_switch));
    tweaks_group.add(&kernel_row);

    // Esync/Fsync
    let sync_row = adw::ActionRow::builder()
        .title("Enable Esync/Fsync")
        .subtitle("Improve game performance with sync optimizations")
        .build();
    let sync_switch = gtk::Switch::builder()
        .valign(gtk::Align::Center)
        .active(config.esync_fsync())
        .build();
    let sync_config = Rc::clone(&config);
    sync_switch.connect_state_set(move |switch, state| {
        sync_config.set_esync_fsync(state);
        if let Some(window) = switch.root().and_then(|r| r.downcast_ref::<gtk::Window>().cloned()) {
            let cmd = if state {
                "pkexec bash -c 'echo \"* hard nofile 1048576\" >> /etc/security/limits.d/99-esync.conf && echo \"* soft nofile 1048576\" >> /etc/security/limits.d/99-esync.conf'"
            } else {
                "pkexec rm -f /etc/security/limits.d/99-esync.conf"
            };
            terminal::show(&window, "Esync/Fsync", cmd, None);
        }
        glib::Propagation::Proceed
    });
    sync_row.add_suffix(&sync_switch);
    sync_row.set_activatable_widget(Some(&sync_switch));
    tweaks_group.add(&sync_row);

    content.append(&tweaks_group);

    // About
    let about_group = adw::PreferencesGroup::builder()
        .title("About")
        .build();

    let app_row = adw::ActionRow::builder()
        .title("Parch Linux Gamehub")
        .subtitle("Version 0.1.1")
        .build();
    app_row.add_prefix(&gtk::Image::from_icon_name("applications-games"));
    about_group.add(&app_row);

    let credits_row = adw::ActionRow::builder()
        .title("Credits")
        .subtitle("Built for Parch Linux")
        .activatable(true)
        .build();
    credits_row.add_suffix(&gtk::Image::from_icon_name("go-next"));
    about_group.add(&credits_row);

    content.append(&about_group);

    clamp.set_child(Some(&content));
    scroll.set_child(Some(&clamp));

    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    container.append(&scroll);
    container
}
