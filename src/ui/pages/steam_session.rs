use adw::prelude::*;

use crate::ui::dialogs::terminal;

pub fn new() -> gtk::Box {
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

    // Steam Gaming Session section
    let session_group = adw::PreferencesGroup::builder()
        .title("Create Gaming Session")
        .description("Set up a dedicated Steam gaming session on Parch Linux")
        .build();

    // Install Steam
    let steam_row = adw::ActionRow::builder()
        .title("Install Steam")
        .subtitle("Installs Steam and Steam Native Runtime")
        .build();
    let install_steam_btn = gtk::Button::builder()
        .label("Install")
        .css_classes(["suggested-action"])
        .valign(gtk::Align::Center)
        .build();
    install_steam_btn.connect_clicked(|btn| {
        if let Some(root) = btn.root() {
            if let Some(window) = root.downcast_ref::<gtk::Window>() {
                terminal::show(
                    window,
                    "Install Steam",
                    "pkexec pacman -S --noconfirm steam steam-native-runtime",
                    None,
                );
            }
        }
    });
    steam_row.add_suffix(&install_steam_btn);
    session_group.add(&steam_row);

    // Install Gamescope
    let gamescope_row = adw::ActionRow::builder()
        .title("Install Gamescope")
        .subtitle("Micro-compositor for gamescope session (gaming-focused Wayland compositor)")
        .build();
    let install_gamescope_btn = gtk::Button::builder()
        .label("Install")
        .css_classes(["suggested-action"])
        .valign(gtk::Align::Center)
        .build();
    install_gamescope_btn.connect_clicked(|btn| {
        if let Some(root) = btn.root() {
            if let Some(window) = root.downcast_ref::<gtk::Window>() {
                terminal::show(
                    window,
                    "Install Gamescope",
                    "pkexec pacman -S --noconfirm gamescope",
                    None,
                );
            }
        }
    });
    gamescope_row.add_suffix(&install_gamescope_btn);
    session_group.add(&gamescope_row);

    // Create dedicated gaming user
    let user_row = adw::ActionRow::builder()
        .title("Create Gaming User (steam)")
        .subtitle("Creates a dedicated 'steam' user with auto-login for gaming mode")
        .build();
    let create_user_btn = gtk::Button::builder()
        .label("Create")
        .css_classes(["suggested-action"])
        .valign(gtk::Align::Center)
        .build();
    create_user_btn.connect_clicked(|btn| {
        if let Some(root) = btn.root() {
            if let Some(window) = root.downcast_ref::<gtk::Window>() {
                let cmd = "if ! id steam &>/dev/null; then \
                           pkexec useradd -m -G wheel -s /bin/bash steam && \
                           echo 'Created user: steam' && \
                           echo 'Set a password with: sudo passwd steam'; \
                           else echo 'User steam already exists'; fi"
                    .to_string();
                terminal::show(window, "Create Gaming User", &cmd, None);
            }
        }
    });
    user_row.add_suffix(&create_user_btn);
    session_group.add(&user_row);

    // Steam auto-start (Big Picture)
    let autostart_row = adw::ActionRow::builder()
        .title("Steam Auto-Start (Big Picture Mode)")
        .subtitle("Creates an autostart entry so Steam launches in Big Picture on login")
        .build();
    let autostart_btn = gtk::Button::builder()
        .label("Enable")
        .css_classes(["suggested-action"])
        .valign(gtk::Align::Center)
        .build();
    autostart_btn.connect_clicked(|btn| {
        if let Some(root) = btn.root() {
            if let Some(window) = root.downcast_ref::<gtk::Window>() {
                let cmd = r#"mkdir -p ~/.config/autostart && cat > ~/.config/autostart/steam-bigpicture.desktop << 'EOF'
[Desktop Entry]
Type=Application
Name=Steam Big Picture
Exec=/usr/bin/steam -silent -bigpicture
X-GNOME-Autostart-enabled=true
EOF
echo 'Steam autostart configured'"#
                    .to_string();
                terminal::show(window, "Enable Steam Auto-Start", &cmd, None);
            }
        }
    });
    autostart_row.add_suffix(&autostart_btn);
    session_group.add(&autostart_row);

    // Gaming optimizations
    let optimize_row = adw::ActionRow::builder()
        .title("Gaming Optimizations")
        .subtitle("Install GameMode, MangoHud, and performance tweaks")
        .build();
    let optimize_btn = gtk::Button::builder()
        .label("Optimize")
        .css_classes(["suggested-action"])
        .valign(gtk::Align::Center)
        .build();
    optimize_btn.connect_clicked(|btn| {
        if let Some(root) = btn.root() {
            if let Some(window) = root.downcast_ref::<gtk::Window>() {
                terminal::show(
                    window,
                    "Gaming Optimizations",
                    "pkexec pacman -S --noconfirm mangohud gamemode lib32-gamemode goverlay",
                    None,
                );
            }
        }
    });
    optimize_row.add_suffix(&optimize_btn);
    session_group.add(&optimize_row);

    content.append(&session_group);

    // info section
    let info_group = adw::PreferencesGroup::builder()
        .title("About Gaming Session")
        .description("A dedicated Steam gaming session creates an optimized environment for gaming on Parch Linux")
        .build();

    let info_row = adw::ActionRow::builder()
        .title("What this does")
        .subtitle(
            "Creates a dedicated 'steam' user, installs gaming packages, \
             configures auto-login and Steam Big Picture auto-start, \
             and applies performance tweaks for the best gaming experience."
        )
        .build();
    info_group.add(&info_row);

    let gpu_row = adw::ActionRow::builder()
        .title("GPU Drivers")
        .subtitle("Make sure your GPU drivers are installed (mesa, nvidia-dkms, or amdgpu)")
        .build();
    info_group.add(&gpu_row);

    content.append(&info_group);

    clamp.set_child(Some(&content));
    scroll.set_child(Some(&clamp));

    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    container.append(&scroll);
    container
}
