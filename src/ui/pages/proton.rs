use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;

use adw::prelude::*;

use crate::modules::proton_manager::{CompatTool, ProtonManager, ReleaseInfo};

struct UiState {
    tools: Vec<CompatTool>,
    all_releases: RefCell<Vec<Vec<ReleaseInfo>>>,
    selected_tool: usize,
    selected_version: usize,
}

pub fn new() -> gtk::Box {
    let tools = ProtonManager::available_tools();
    let tool_count = tools.len();

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

    let spinner = gtk::Spinner::builder()
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .margin_top(48)
        .build();
    spinner.start();
    content.append(&spinner);

    clamp.set_child(Some(&content));
    scroll.set_child(Some(&clamp));

    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    container.append(&scroll);

    let (tx, rx) = mpsc::channel::<(usize, Vec<ReleaseInfo>)>();
    let tools_for_thread = tools.clone();

    for (i, tool) in tools_for_thread.iter().enumerate() {
        let tx = tx.clone();
        let id = tool.id;
        let tools_ref = tools_for_thread.clone();
        std::thread::spawn(move || {
            let t = tools_ref.iter().find(|t| t.id == id).unwrap();
            let releases = ProtonManager::list_available_releases(t);
            let _ = tx.send((i, releases));
        });
    }
    drop(tx);

    let content_loading = content.clone();
    let spinner_loading = spinner.clone();
    let tools_loaded = Rc::new(RefCell::new(Some(tools.clone())));

    glib::idle_add_local(move || {
        let all_releases = RefCell::new(vec![vec![]; tool_count]);
        let mut ready = 0;

        while let Ok((i, rels)) = rx.try_recv() {
            all_releases.borrow_mut()[i] = rels;
            ready += 1;
        }

        if ready < tool_count {
            glib::ControlFlow::Continue
        } else {
            spinner_loading.stop();
            spinner_loading.set_visible(false);
            if let Some(t) = tools_loaded.borrow_mut().take() {
                build_page(&content_loading, t, all_releases.into_inner());
            }
            glib::ControlFlow::Break
        }
    });

    container
}

fn build_page(content: &gtk::Box, tools: Vec<CompatTool>, all_releases: Vec<Vec<ReleaseInfo>>) {
    while let Some(child) = content.first_child() {
        content.remove(&child);
    }

    let state = Rc::new(RefCell::new(UiState {
        tools,
        all_releases: RefCell::new(all_releases),
        selected_tool: 0,
        selected_version: 0,
    }));

    // Tool selector
    let selector_group = adw::PreferencesGroup::builder()
        .title("Compatibility Tool")
        .description("Select a tool to manage")
        .build();

    let tool_store = gtk::StringList::new(&[]);
    for t in &state.borrow().tools {
        tool_store.append(t.name);
    }

    let tool_dropdown = gtk::DropDown::builder()
        .model(&tool_store)
        .selected(0)
        .valign(gtk::Align::Center)
        .build();
    let tool_row = adw::ActionRow::builder()
        .title("Tool")
        .subtitle("Choose a compatibility tool")
        .build();
    tool_row.add_suffix(&tool_dropdown);
    selector_group.add(&tool_row);

    let refresh_btn = gtk::Button::builder()
        .label("Refresh")
        .valign(gtk::Align::Center)
        .build();
    let refresh_row = adw::ActionRow::builder()
        .title("Releases")
        .subtitle("Fetch latest releases from GitHub")
        .build();
    refresh_row.add_suffix(&refresh_btn);
    selector_group.add(&refresh_row);

    content.append(&selector_group);

    // Version list
    let version_group = adw::PreferencesGroup::builder()
        .title("Available Versions")
        .description("Select a version to install")
        .build();

    let version_list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::Single)
        .build();

    let version_scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .max_content_height(300)
        .propagate_natural_height(true)
        .child(&version_list)
        .build();

    version_group.add(&version_scroll);
    content.append(&version_group);

    // Installed versions
    let installed_group = adw::PreferencesGroup::builder()
        .title("Installed Versions")
        .description("Currently installed versions for the selected tool")
        .build();
    let installed_list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::Single)
        .build();
    let installed_scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .max_content_height(150)
        .propagate_natural_height(true)
        .child(&installed_list)
        .build();
    installed_group.add(&installed_scroll);
    content.append(&installed_group);

    // Description
    let desc_group = adw::PreferencesGroup::builder()
        .title("Release Notes")
        .build();
    let desc_view = gtk::TextView::builder()
        .editable(false)
        .wrap_mode(gtk::WrapMode::Word)
        .vexpand(true)
        .height_request(120)
        .build();
    let desc_scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .propagate_natural_height(true)
        .child(&desc_view)
        .build();
    desc_group.add(&desc_scroll);
    content.append(&desc_group);

    // Action buttons
    let action_group = adw::PreferencesGroup::builder().build();
    let install_btn = gtk::Button::builder()
        .label("Install Selected Version")
        .css_classes(["suggested-action"])
        .hexpand(true)
        .build();
    let remove_btn = gtk::Button::builder()
        .label("Remove Selected")
        .css_classes(["destructive-action"])
        .hexpand(true)
        .build();

    let btn_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .homogeneous(true)
        .build();
    btn_box.append(&install_btn);
    btn_box.append(&remove_btn);

    let action_row = adw::ActionRow::builder().build();
    action_row.add_suffix(&btn_box);
    action_group.add(&action_row);
    content.append(&action_group);

    // Populate initial version list
    populate_view(&state, &version_list, &installed_list, &desc_view, 0);

    // Tool selector handler
    let state_s = state.clone();
    let vl = version_list.clone();
    let il = installed_list.clone();
    let dv = desc_view.clone();
    tool_dropdown.connect_selected_notify(move |dd| {
        let idx = dd.selected() as usize;
        state_s.borrow_mut().selected_tool = idx;
        state_s.borrow_mut().selected_version = 0;
        populate_view(&state_s, &vl, &il, &dv, idx);
    });

    // Install button
    let state_i = state.clone();
    install_btn.connect_clicked(move |btn| {
        let s = state_i.borrow();
        let tool_idx = s.selected_tool;
        let ver_idx = s.selected_version;
        let rels = s.all_releases.borrow();
        if let Some(release) = rels[tool_idx].get(ver_idx) {
            if let Some(root) = btn.root() {
                if let Some(window) = root.downcast_ref::<gtk::Window>() {
                    let cmd = format!(
                        "mkdir -p ~/.steam/steam/compatibilitytools.d && \
                         cd /tmp && \
                         curl -fsSL -o '{}' '{}' && \
                         tar -xf '{}' -C ~/.steam/steam/compatibilitytools.d/ && \
                         rm -f '{}'",
                        release.asset_name,
                        release.download_url,
                        release.asset_name,
                        release.asset_name
                    );
                    let tool_name = s.tools[tool_idx].name;
                    let title = format!("Install {} - {}", tool_name, release.version);
                    crate::ui::dialogs::terminal::show(window, &title, &cmd, None);
                }
            }
        }
    });

    // Refresh button
    let state_r = state.clone();
    let vl_r = version_list.clone();
    let il_r = installed_list.clone();
    let dv_r = desc_view.clone();
    refresh_btn.connect_clicked(move |_| {
        let (tx, rx) = mpsc::channel::<(usize, Vec<ReleaseInfo>)>();
        let tools_ref = state_r.borrow().tools.clone();
        let tool_count = tools_ref.len();

        for (i, tool) in tools_ref.iter().enumerate() {
            let tx = tx.clone();
            let id = tool.id;
            let t_ref = tools_ref.clone();
            std::thread::spawn(move || {
                let t = t_ref.iter().find(|t| t.id == id).unwrap();
                let rels = ProtonManager::list_available_releases(t);
                let _ = tx.send((i, rels));
            });
        }
        drop(tx);

        let state_r2 = state_r.clone();
        let vl_r2 = vl_r.clone();
        let il_r2 = il_r.clone();
        let dv_r2 = dv_r.clone();

        glib::idle_add_local(move || {
            let mut ready = 0;
            while let Ok((i, rels)) = rx.try_recv() {
                state_r2.borrow().all_releases.borrow_mut()[i] = rels;
                ready += 1;
            }

            if ready >= tool_count {
                // Refresh complete, update view
                let idx = state_r2.borrow().selected_tool;
                populate_view(&state_r2, &vl_r2, &il_r2, &dv_r2, idx);
                glib::ControlFlow::Break
            } else if ready == 0 && rx.try_recv().is_err() {
                match rx.try_recv() {
                    Err(mpsc::TryRecvError::Disconnected) => {
                        glib::ControlFlow::Break
                    }
                    _ => glib::ControlFlow::Continue,
                }
            } else {
                glib::ControlFlow::Continue
            }
        });
    });
}

fn populate_view(
    state: &Rc<RefCell<UiState>>,
    version_list: &gtk::ListBox,
    installed_list: &gtk::ListBox,
    desc_view: &gtk::TextView,
    tool_idx: usize,
) {
    let s = state.borrow();
    let rels = s.all_releases.borrow();
    let tool = &s.tools[tool_idx];
    let releases = &rels[tool_idx];

    // Clear version list
    while let Some(child) = version_list.first_child() {
        version_list.remove(&child);
    }
    while let Some(child) = installed_list.first_child() {
        installed_list.remove(&child);
    }
    desc_view.buffer().set_text("");

    // Populate versions
    for r in releases {
        let row = adw::ActionRow::builder()
            .title(&r.version)
            .subtitle(&r.asset_name)
            .activatable(true)
            .build();
        version_list.append(&row);
    }
    if releases.is_empty() {
        let row = adw::ActionRow::builder()
            .title("No releases available")
            .subtitle("Check internet connection")
            .build();
        version_list.append(&row);
    }

    // Populate installed
    let installed = ProtonManager::list_installed(tool);
    for v in &installed {
        let row = adw::ActionRow::builder()
            .title(&v.version)
            .subtitle("Installed")
            .build();
        installed_list.append(&row);
    }
    if installed.is_empty() {
        let row = adw::ActionRow::builder()
            .title("None installed")
            .build();
        installed_list.append(&row);
    }

    // Show first version description
    if let Some(first) = releases.first() {
        desc_view.buffer().set_text(&first.body);
    }
}
