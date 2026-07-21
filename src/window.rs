use adw::prelude::*;
use gtk::glib;
use gtk::gio;

use crate::ui::pages;

pub fn create(app: &adw::Application) -> adw::ApplicationWindow {
    let view_stack = adw::ViewStack::new();

    let dashboard = pages::dashboard::new(Some(view_stack.clone()));
    let page = view_stack.add_titled(&dashboard, Some("dashboard"), "Dashboard");
    page.set_icon_name(Some("view-grid"));

    let launchers = pages::launchers::new();
    let page = view_stack.add_titled(&launchers, Some("launchers"), "Launchers");
    page.set_icon_name(Some("applications-games"));

    let proton = pages::proton::new();
    let page = view_stack.add_titled(&proton, Some("proton"), "Proton");
    page.set_icon_name(Some("media-playback-start"));

    let steam_session = pages::steam_session::new();
    let page = view_stack.add_titled(&steam_session, Some("steam-session"), "Steam Session");
    page.set_icon_name(Some("computer"));

    let tools = pages::tools::new();
    let page = view_stack.add_titled(&tools, Some("tools"), "Tools");
    page.set_icon_name(Some("applications-utilities"));

    let settings = pages::settings::new();
    let page = view_stack.add_titled(&settings, Some("settings"), "Settings");
    page.set_icon_name(Some("preferences-system"));

    let view_switcher = adw::ViewSwitcher::builder()
        .stack(&view_stack)
        .build();

    let header = adw::HeaderBar::builder()
        .title_widget(&view_switcher)
        .build();

    let menu_button = gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .tooltip_text("Main Menu")
        .build();

    let menu = gio::Menu::new();
    menu.append(Some("About Gamehub"), Some("app.about"));
    menu.append(Some("Keyboard Shortcuts"), Some("app.shortcuts"));
    menu_button.set_menu_model(Some(&menu));
    header.pack_end(&menu_button);

    let main_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    main_box.append(&header);
    main_box.append(&view_stack);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(950)
        .default_height(650)
        .title("Parch Linux Gamehub")
        .content(&main_box)
        .build();

    let app_weak = app.downgrade();
    window.connect_close_request(move |_| {
        if let Some(app) = app_weak.upgrade() {
            app.quit();
        }
        glib::Propagation::Proceed
    });

    window
}
