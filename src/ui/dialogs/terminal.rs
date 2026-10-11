use adw::prelude::*;
use std::io::BufRead;
use std::process::{Command, Stdio};

use crate::modules::launcher_manager::apply_locale;

enum TermMsg {
    Output(String),
    Done(bool),
}

pub fn show(
    parent: &gtk::Window,
    title: &str,
    command: &str,
    on_complete: Option<Box<dyn FnOnce(bool) + 'static>>,
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
    let (tx, rx) = std::sync::mpsc::channel::<TermMsg>();

    let cmd = command.to_string();

    std::thread::spawn(move || {
        let wrapped = format!(
            "{}\necho ''\necho '--- EXIT CODE: $? ---'\n",
            cmd
        );

        let mut child = {
            let mut cmd = Command::new("/bin/bash");
            cmd.args(["-c", &wrapped])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            apply_locale(&mut cmd);
            match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(TermMsg::Output(format!("Error: {}\n", e)));
                    let _ = tx.send(TermMsg::Done(false));
                    return;
                }
            }
        };

        if let Some(stdout) = child.stdout.take() {
            let tx = tx.clone();
            std::thread::spawn(move || {
                let reader = std::io::BufReader::new(stdout);
                for line in reader.lines().map_while(|l| l.ok()) {
                    if tx.send(TermMsg::Output(format!("{}\n", line))).is_err() {
                        break;
                    }
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let tx = tx.clone();
            std::thread::spawn(move || {
                let reader = std::io::BufReader::new(stderr);
                for line in reader.lines().map_while(|l| l.ok()) {
                    if tx.send(TermMsg::Output(format!("{}\n", line))).is_err() {
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

        let _ = tx.send(TermMsg::Done(success));
    });

    // Poll for output updates on main thread
    let buffer = text_view.buffer();
    let text_view_scroll = text_view.clone();
    let mut on_complete = on_complete;

    glib::idle_add_local(move || {
        // Drain available output
        while let Ok(msg) = rx.try_recv() {
            match msg {
                TermMsg::Output(line) => {
                    let mut end = buffer.end_iter();
                    buffer.insert(&mut end, &line);
                    // Auto-scroll to the latest output
                    text_view_scroll.scroll_to_iter(&mut buffer.end_iter(), 0.0, false, 0.0, 0.0);
                }
                TermMsg::Done(success) => {
                    spinner.stop();
                    if success {
                        status_label.set_label("\u{2713} Command completed successfully");
                    } else {
                        status_label.set_label("\u{2717} Command failed");
                    }
                    close_btn.set_sensitive(true);

                    if let Some(cb) = on_complete.take() {
                        cb(success);
                    }

                    return glib::ControlFlow::Break;
                }
            }
        }
        glib::ControlFlow::Continue
    });
}
