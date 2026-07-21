use adw::prelude::*;
use gtk::gio;

use crate::window;

pub fn run() {
    let app = adw::Application::builder()
        .application_id("com.parchlinux.gamehub")
        .resource_base_path("/com/parchlinux/gamehub")
        .build();

    app.connect_startup(|app| {
        setup_actions(app);
    });

    app.connect_activate(|app| {
        let win = window::create(app);
        win.present();
    });

    app.run();
}

fn setup_actions(app: &adw::Application) {
    let quit_action = gio::SimpleAction::new("quit", None);
    let app_weak = app.downgrade();
    quit_action.connect_activate(move |_, _| {
        if let Some(app) = app_weak.upgrade() {
            app.quit();
        }
    });
    app.add_action(&quit_action);

    let about_action = gio::SimpleAction::new("about", None);
    let app_weak = app.downgrade();
    about_action.connect_activate(move |_, _| {
        if let Some(app) = app_weak.upgrade() {
            show_about(&app);
        }
    });
    app.add_action(&about_action);

    let shortcuts_action = gio::SimpleAction::new("shortcuts", None);
    let app_weak = app.downgrade();
    shortcuts_action.connect_activate(move |_, _| {
        if let Some(app) = app_weak.upgrade() {
            show_shortcuts(&app);
        }
    });
    app.add_action(&shortcuts_action);

    app.set_accels_for_action("app.quit", &["<primary>q"]);
}

fn show_about(app: &adw::Application) {
    let about = adw::AboutDialog::builder()
        .application_name("Parch Linux Gamehub")
        .application_icon("com.parchlinux.gamehub")
        .version("0.1.1")
        .developers(["Parch Linux"])
        .website("https://parchlinux.com")
        .copyright("© 2026 Parch Linux")
        .license_type(gtk::License::Agpl30)
        .build();
    about.present(app.active_window().as_ref());
}

fn show_shortcuts(app: &adw::Application) {
    let builder = gtk::Builder::from_string(
        r#"<interface>
  <object class="GtkShortcutsWindow" id="shortcuts">
    <property name="modal">true</property>
    <child>
      <object class="GtkShortcutsSection">
        <property name="title">General</property>
        <child>
          <object class="GtkShortcutsGroup">
            <property name="title">Shortcuts</property>
            <child>
              <object class="GtkShortcutsShortcut">
                <property name="title">Quit</property>
                <property name="accelerator">&lt;primary&gt;q</property>
              </object>
            </child>
          </object>
        </child>
      </object>
    </child>
  </object>
</interface>"#,
    );
    let shortcuts: gtk::ShortcutsWindow = builder
        .object("shortcuts")
        .expect("Missing shortcuts window");
    shortcuts.set_transient_for(app.active_window().as_ref());
    shortcuts.present();
}
