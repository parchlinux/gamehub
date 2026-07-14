use adw::prelude::*;
use std::io::BufRead;
use std::process::{Command, Stdio};

pub fn show(
    parent: &gtk::Window,
    title: &str,
    command: &str,
    mut on_complete: Option<Box<dyn FnOnce(bool) + 'static>>,
) {
    let dialog = adw::Window::builder()
        .transient_for(parent)
        .modal(true)
        .default_width(800)
        .default_height(500)
        .title(title)
        .build();

    let box_ = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(0)
        .build();

    let header = adw::HeaderBar::new();

    let close_btn = gtk::Button::with_label("Close");
    close_btn.set_sensitive(false);
    let dialog_close = dialog.clone();
    close_btn.connect_clicked(move |_| {
        dialog_close.close();
    });
    header.pack_end(&close_btn);

    box_.append(&header);

    let text_view = gtk::TextView::builder()
        .vexpand(true)
        .hexpand(true)
        .editable(false)
        .monospace(true)
        .build();

    text_view.set_cursor_visible(false);

    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Automatic)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .vexpand(true)
        .child(&text_view)
        .build();

    let terminal_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .vexpand(true)
        .build();
    terminal_box.append(&scrolled);
    box_.append(&terminal_box);

    let status_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(12)
        .margin_end(12)
        .build();

    let status_label = gtk::Label::builder()
        .label("Ready")
        .halign(gtk::Align::Start)
        .hexpand(true)
        .build();
    status_box.append(&status_label);

    let spinner = gtk::Spinner::builder()
        .halign(gtk::Align::End)
        .build();
    status_box.append(&spinner);

    box_.append(&status_box);

    dialog.set_content(Some(&box_));
    dialog.present();

    status_label.set_label("Running command...");
    spinner.start();

    // Use channel for sending output lines from thread
    let (line_tx, line_rx) = std::sync::mpsc::channel::<String>();
    let (done_tx, done_rx) = std::sync::mpsc::channel::<bool>();

    let cmd = command.to_string();

    std::thread::spawn(move || {
        let wrapped = format!(
            "{}\necho ''\necho '--- EXIT CODE: $? ---'\n",
            cmd
        );

        let mut child = match Command::new("/bin/bash")
            .args(["-c", &wrapped])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                let _ = line_tx.send(format!("Error: {}", e));
                let _ = done_tx.send(false);
                return;
            }
        };

        if let Some(stdout) = child.stdout.take() {
            let tx = line_tx.clone();
            std::thread::spawn(move || {
                let reader = std::io::BufReader::new(stdout);
                for line in reader.lines().map_while(|l| l.ok()) {
                    if tx.send(format!("{}\n", line)).is_err() {
                        break;
                    }
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let tx = line_tx.clone();
            std::thread::spawn(move || {
                let reader = std::io::BufReader::new(stderr);
                for line in reader.lines().map_while(|l| l.ok()) {
                    if tx.send(format!("{}\n", line)).is_err() {
                        break;
                    }
                }
            });
        }

        // Wait for command
        let success = child
            .wait()
            .ok()
            .map(|s| s.success())
            .unwrap_or(false);

        let _ = done_tx.send(success);
    });

    // Poll for output updates on main thread
    let buffer = text_view.buffer();
    let status_clone = status_label.clone();
    let spinner_clone = spinner.clone();
    let close_clone = close_btn.clone();

    glib::idle_add_local(move || {
        // Drain available output lines
        while let Ok(line) = line_rx.try_recv() {
            let mut end = buffer.end_iter();
            buffer.insert(&mut end, &line);
        }

        // Check if done
        if let Ok(success) = done_rx.try_recv() {
            spinner_clone.stop();
            if success {
                status_clone.set_label("✓ Command completed successfully");
            } else {
                status_clone.set_label("✗ Command failed");
            }
            close_clone.set_sensitive(true);

            if let Some(cb) = on_complete.take() {
                cb(success);
            }

            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
}
