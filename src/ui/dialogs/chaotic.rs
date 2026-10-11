use adw::prelude::*;
use crate::modules::launcher_manager::apply_locale;

pub fn prompt_chaotic_aur(parent: &gtk::Window, item_name: &str) {
    let dialog = adw::Window::builder()
        .transient_for(parent)
        .modal(true)
        .default_width(450)
        .default_height(240)
        .title("Chaotic-AUR Required")
        .build();

    let vbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(16)
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(24)
        .margin_end(24)
        .build();

    let header_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .build();

    let icon = gtk::Image::builder()
        .icon_name("dialog-warning-symbolic")
        .pixel_size(36)
        .css_classes(["warning"])
        .build();

    let title_label = gtk::Label::builder()
        .label("Chaotic-AUR Repository Required")
        .css_classes(["title-3"])
        .halign(gtk::Align::Start)
        .wrap(true)
        .build();

    header_box.append(&icon);
    header_box.append(&title_label);
    vbox.append(&header_box);

    let message_label = gtk::Label::builder()
        .label(format!(
            "<b>{}</b> is provided by the <b>Chaotic-AUR</b> repository.\n\nPlease enable Chaotic-AUR using Parch Repository Manager (mirrorman) before installing this package.",
            glib::markup_escape_text(item_name)
        ))
        .use_markup(true)
        .wrap(true)
        .halign(gtk::Align::Start)
        .build();
    vbox.append(&message_label);

    let btn_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::End)
        .margin_top(8)
        .build();

    let cancel_btn = gtk::Button::builder()
        .label("Cancel")
        .css_classes(["flat"])
        .build();
    let dlg_cancel = dialog.clone();
    cancel_btn.connect_clicked(move |_| {
        dlg_cancel.close();
    });

    let open_btn = gtk::Button::builder()
        .label("Open Repository Manager")
        .css_classes(["suggested-action", "pill"])
        .build();
    let dlg_open = dialog.clone();
    open_btn.connect_clicked(move |_| {
        let mut child = std::process::Command::new("mirrorman");
        child.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
        apply_locale(&mut child);
        if let Ok(mut c) = child.spawn() {
            std::thread::spawn(move || {
                let _ = c.wait();
            });
        }
        dlg_open.close();
    });

    btn_box.append(&cancel_btn);
    btn_box.append(&open_btn);
    vbox.append(&btn_box);

    dialog.set_content(Some(&vbox));
    dialog.present();
}
