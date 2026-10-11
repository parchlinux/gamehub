use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use adw::prelude::*;
use gtk::glib;

use crate::modules::protondb::{
    CheckVerdict, ProtonDbClient, ProtonDbSummary, SteamGameMatch, SteamRequirements,
};

#[derive(Clone)]
struct GameFullData {
    game: SteamGameMatch,
    summary: Option<ProtonDbSummary>,
    reqs: Option<SteamRequirements>,
    cover_path: Option<PathBuf>,
}

pub fn new() -> gtk::Box {
    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();

    let toast_overlay = adw::ToastOverlay::new();
    container.append(&toast_overlay);

    let stack = gtk::Stack::builder()
        .transition_type(gtk::StackTransitionType::SlideLeftRight)
        .transition_duration(200)
        .vexpand(true)
        .build();
    toast_overlay.set_child(Some(&stack));

    // =========================================================================
    // Page 1: Search View
    // =========================================================================
    let search_scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .vexpand(true)
        .build();

    let search_page = adw::PreferencesPage::new();
    search_scrolled.set_child(Some(&search_page));

    // --- Search Input Group ---
    let search_group = adw::PreferencesGroup::builder()
        .title("Game Compatibility and ProtonDB")
        .description("Search Windows games to check Steam Deck / Linux ProtonDB ratings, cover art, and hardware checks.")
        .build();

    let search_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .margin_top(4)
        .margin_bottom(8)
        .build();

    let search_entry = gtk::SearchEntry::builder()
        .placeholder_text("Search game title (e.g. Cyberpunk 2077, Elden Ring, God of War)...")
        .hexpand(true)
        .build();

    let check_btn = gtk::Button::builder()
        .label("Check")
        .css_classes(["suggested-action", "pill"])
        .valign(gtk::Align::Center)
        .build();

    search_box.append(&search_entry);
    search_box.append(&check_btn);
    search_group.add(&search_box);
    search_page.add(&search_group);

    // --- System Specs Overview ---
    let (nominal_ram, usable_ram) = ProtonDbClient::get_system_ram_gb();
    let free_disk = ProtonDbClient::get_free_disk_gb();
    let cpu = ProtonDbClient::get_cpu_model();
    let gpu = ProtonDbClient::get_gpu_model();

    let specs_group = adw::PreferencesGroup::builder()
        .title("Detected Hardware")
        .description("Your hardware specifications used for game requirement comparisons")
        .build();

    let cpu_row = adw::ActionRow::builder()
        .title("Processor")
        .subtitle(&cpu)
        .build();
    cpu_row.add_prefix(&gtk::Image::from_icon_name("cpu-symbolic"));
    specs_group.add(&cpu_row);

    let gpu_row = adw::ActionRow::builder()
        .title("Graphics Card")
        .subtitle(&gpu)
        .build();
    gpu_row.add_prefix(&gtk::Image::from_icon_name("video-display-symbolic"));
    specs_group.add(&gpu_row);

    let ram_subtitle = if (nominal_ram - usable_ram).abs() > 0.05 {
        format!("{:.0} GB ({:.1} GB usable)", nominal_ram, usable_ram)
    } else {
        format!("{:.1} GB", usable_ram)
    };
    let ram_row = adw::ActionRow::builder()
        .title("Memory (RAM)")
        .subtitle(&ram_subtitle)
        .build();
    ram_row.add_prefix(&gtk::Image::from_icon_name("drive-multidisk-symbolic"));
    specs_group.add(&ram_row);

    let disk_row = adw::ActionRow::builder()
        .title("Available Disk Space")
        .subtitle(format!("{:.1} GB free in home directory", free_disk))
        .build();
    disk_row.add_prefix(&gtk::Image::from_icon_name("drive-harddisk-symbolic"));
    specs_group.add(&disk_row);

    search_page.add(&specs_group);

    // --- Status and Loading Indicator ---
    let status_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::Center)
        .margin_top(12)
        .margin_bottom(12)
        .build();

    let spinner = gtk::Spinner::builder()
        .spinning(false)
        .visible(false)
        .build();
    status_box.append(&spinner);

    let status_label = gtk::Label::builder()
        .label("Search for any game to view compatibility and hardware checks.")
        .css_classes(["dim-label"])
        .wrap(true)
        .build();
    status_box.append(&status_label);

    let status_group = adw::PreferencesGroup::new();
    status_group.add(&status_box);
    search_page.add(&status_group);

    // --- Search Results List ---
    let results_group = adw::PreferencesGroup::builder()
        .title("Search Results")
        .build();
    let results_list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();
    results_group.add(&results_list);
    results_group.set_visible(false);
    search_page.add(&results_group);

    stack.add_named(&search_scrolled, Some("search"));

    // =========================================================================
    // Page 2: Detail View
    // =========================================================================
    let detail_scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .vexpand(true)
        .build();

    let detail_page = adw::PreferencesPage::new();
    detail_scrolled.set_child(Some(&detail_page));

    stack.add_named(&detail_scrolled, Some("detail"));

    // State to hold current search results
    let current_results: Rc<RefCell<Vec<GameFullData>>> = Rc::new(RefCell::new(Vec::new()));

    // Search Trigger handler
    let trigger_search = {
        let entry = search_entry.clone();
        let btn = check_btn.clone();
        let spinner = spinner.clone();
        let status = status_label.clone();
        let r_list = results_list.clone();
        let r_group = results_group.clone();
        let stack = stack.clone();
        let toast = toast_overlay.clone();
        let detail_scrolled = detail_scrolled.clone();
        let current_results = current_results.clone();

        move || {
            let query = entry.text().trim().to_string();
            if query.is_empty() {
                return;
            }

            btn.set_sensitive(false);
            spinner.set_visible(true);
            spinner.start();
            status.set_label("Searching Steam and ProtonDB, fetching cover art...");
            r_group.set_visible(false);

            while let Some(child) = r_list.first_child() {
                r_list.remove(&child);
            }

            let (tx, rx) = std::sync::mpsc::channel::<Vec<GameFullData>>();

            std::thread::spawn(move || {
                let matches = ProtonDbClient::search_steam_games(&query);
                let mut data_list = Vec::new();

                for m in matches.into_iter().take(6) {
                    let summary = ProtonDbClient::get_protondb_summary(m.id);
                    let reqs = ProtonDbClient::get_steam_requirements(m.id);
                    let cover_path = ProtonDbClient::get_cover_image(m.id);

                    data_list.push(GameFullData {
                        game: m,
                        summary,
                        reqs,
                        cover_path,
                    });
                }

                let _ = tx.send(data_list);
            });

            let btn_clone = btn.clone();
            let spinner_clone = spinner.clone();
            let status_clone = status.clone();
            let r_list_clone = r_list.clone();
            let r_group_clone = r_group.clone();
            let stack_clone = stack.clone();
            let toast_clone = toast.clone();
            let detail_scrolled_clone = detail_scrolled.clone();
            let current_results_clone = current_results.clone();

            glib::timeout_add_local(std::time::Duration::from_millis(60), move || {
                if let Ok(data_list) = rx.try_recv() {
                    btn_clone.set_sensitive(true);
                    spinner_clone.stop();
                    spinner_clone.set_visible(false);

                    if data_list.is_empty() {
                        status_clone.set_label("No matches found on Steam.");
                        r_group_clone.set_visible(false);
                    } else {
                        status_clone.set_label(&format!("Found {} games. Click any game to view full details and hardware checks.", data_list.len()));
                        *current_results_clone.borrow_mut() = data_list.clone();

                        for item in &data_list {
                            let item_data = item.clone();
                            let stack_for_click = stack_clone.clone();
                            let toast_for_click = toast_clone.clone();
                            let detail_scrolled_for_click = detail_scrolled_clone.clone();

                            let row = create_search_result_row(&item_data);

                            row.connect_activated(move |_| {
                                populate_detail_page(
                                    &detail_scrolled_for_click,
                                    &item_data,
                                    &stack_for_click,
                                    &toast_for_click,
                                );
                                stack_for_click.set_visible_child_name("detail");
                            });

                            r_list_clone.append(&row);
                        }
                        r_group_clone.set_visible(true);
                    }
                    glib::ControlFlow::Break
                } else {
                    glib::ControlFlow::Continue
                }
            });
        }
    };

    let trigger_clone = trigger_search.clone();
    search_entry.connect_activate(move |_| {
        trigger_clone();
    });

    check_btn.connect_clicked(move |_| {
        trigger_search();
    });

    container
}

/// Create a clean, compact search result row that opens the full detail page on click.
fn create_search_result_row(info: &GameFullData) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title(&info.game.name)
        .subtitle(format!("Steam AppID: {}", info.game.id))
        .activatable(true)
        .build();

    // Cover thumbnail on the left
    if let Some(ref cover) = info.cover_path {
        let pic = gtk::Picture::for_filename(cover);
        pic.set_content_fit(gtk::ContentFit::Cover);
        pic.set_size_request(80, 38);
        pic.set_css_classes(&["rounded"]);
        row.add_prefix(&pic);
    } else {
        let icon = gtk::Image::from_icon_name("applications-games");
        icon.set_pixel_size(36);
        row.add_prefix(&icon);
    }

    // ProtonDB Badge on the right
    let tier_str = info.summary.as_ref().map(|s| s.tier.as_str()).unwrap_or("unknown");
    let (badge_text, badge_class) = match tier_str {
        "platinum" => ("PLATINUM", "success"),
        "gold" => ("GOLD", "accent"),
        "silver" => ("SILVER", "card"),
        "bronze" => ("BRONZE", "warning"),
        "borked" => ("BORKED", "destructive-action"),
        _ => ("NO DATA", "card"),
    };

    let badge = gtk::Label::builder()
        .label(badge_text)
        .css_classes([badge_class, "pill", "caption"])
        .valign(gtk::Align::Center)
        .build();
    row.add_suffix(&badge);

    // Chevron to show it's clickable
    let chevron = gtk::Image::from_icon_name("go-next-symbolic");
    chevron.set_css_classes(&["dim-label"]);
    chevron.set_valign(gtk::Align::Center);
    row.add_suffix(&chevron);

    row
}

/// Populate the dedicated Game Detail page with full cover art, requirements, and actions.
fn populate_detail_page(
    detail_scrolled: &gtk::ScrolledWindow,
    info: &GameFullData,
    stack: &gtk::Stack,
    toast_overlay: &adw::ToastOverlay,
) {
    let page = adw::PreferencesPage::new();

    // --- Header & Navigation Bar ---
    let nav_group = adw::PreferencesGroup::new();
    let nav_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .margin_top(8)
        .margin_bottom(12)
        .build();

    let back_btn = gtk::Button::builder()
        .label(" Back to Search")
        .icon_name("go-previous-symbolic")
        .css_classes(["flat"])
        .valign(gtk::Align::Center)
        .build();

    let stack_back = stack.clone();
    back_btn.connect_clicked(move |_| {
        stack_back.set_visible_child_name("search");
    });
    nav_box.append(&back_btn);
    nav_group.add(&nav_box);
    page.add(&nav_group);

    // --- Main Game Header Card ---
    let header_group = adw::PreferencesGroup::new();
    let card = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(0)
        .css_classes(["card"])
        .build();

    // Top Cover banner
    if let Some(ref cover) = info.cover_path {
        let pic = gtk::Picture::for_filename(cover);
        pic.set_content_fit(gtk::ContentFit::Cover);
        pic.set_size_request(460, 215);
        pic.set_css_classes(&["rounded"]);
        card.append(&pic);
    }

    // Game Title & ProtonDB Tier Section
    let meta_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .margin_start(16)
        .margin_end(16)
        .margin_top(16)
        .margin_bottom(16)
        .build();

    let title_label = gtk::Label::builder()
        .label(&info.game.name)
        .css_classes(["title-1"])
        .halign(gtk::Align::Start)
        .wrap(true)
        .build();
    meta_box.append(&title_label);

    let tier_str = info.summary.as_ref().map(|s| s.tier.as_str()).unwrap_or("unknown");
    let (badge_text, badge_class, desc_text) = match tier_str {
        "platinum" => ("ProtonDB: PLATINUM", "success", "Runs flawlessly out of the box with zero tweaks needed."),
        "gold" => ("ProtonDB: GOLD", "accent", "Runs perfectly after minor tweaks or launch options."),
        "silver" => ("ProtonDB: SILVER", "card", "Runs with minor issues or playable performance tweaks."),
        "bronze" => ("ProtonDB: BRONZE", "warning", "Runs but frequently crashes, severe glitches, or performance drops."),
        "borked" => ("ProtonDB: BORKED", "destructive-action", "Game does not run or is unplayable."),
        _ => ("ProtonDB: NO DATA", "card", "No community reports submitted for this game yet."),
    };

    let badge_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();

    let badge = gtk::Label::builder()
        .label(badge_text)
        .css_classes([badge_class, "pill", "heading"])
        .halign(gtk::Align::Start)
        .build();
    badge_box.append(&badge);

    let appid_label = gtk::Label::builder()
        .label(format!("Steam AppID: {}", info.game.id))
        .css_classes(["dim-label"])
        .valign(gtk::Align::Center)
        .build();
    badge_box.append(&appid_label);
    meta_box.append(&badge_box);

    let tier_desc = gtk::Label::builder()
        .label(desc_text)
        .css_classes(["dim-label"])
        .halign(gtk::Align::Start)
        .wrap(true)
        .build();
    meta_box.append(&tier_desc);

    if let Some(ref s) = info.summary {
        let stats_str = format!(
            "{} community reports • Confidence: {} • Trending: {}",
            s.total,
            s.confidence,
            if s.trending_tier.is_empty() { "stable" } else { &s.trending_tier }
        );
        let stats_label = gtk::Label::builder()
            .label(&stats_str)
            .css_classes(["dim-label", "caption"])
            .halign(gtk::Align::Start)
            .build();
        meta_box.append(&stats_label);
    }

    card.append(&meta_box);

    // Action buttons bar
    let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
    card.append(&sep);

    let actions_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .margin_start(16)
        .margin_end(16)
        .margin_top(12)
        .margin_bottom(16)
        .build();

    // Copy Launch Option
    let copy_btn = gtk::Button::builder()
        .label("Copy Launch Option")
        .icon_name("edit-copy-symbolic")
        .css_classes(["suggested-action", "pill"])
        .tooltip_text("Copy 'gamemoderun %command%' for Steam launch options")
        .build();
    let toast_clone = toast_overlay.clone();
    copy_btn.connect_clicked(move |_| {
        if let Some(display) = gtk::gdk::Display::default() {
            display.clipboard().set_text("gamemoderun %command%");
        }
        toast_clone.add_toast(adw::Toast::new("Copied 'gamemoderun %command%' to clipboard"));
    });
    actions_box.append(&copy_btn);

    // Open ProtonDB in browser
    let appid = info.game.id;
    let protondb_btn = gtk::Button::builder()
        .label("Open ProtonDB")
        .icon_name("insert-link-symbolic")
        .css_classes(["pill"])
        .tooltip_text("View player reports, proton fixes, and Deck compatibility")
        .build();
    protondb_btn.connect_clicked(move |_| {
        let url = format!("https://www.protondb.com/app/{}", appid);
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    });
    actions_box.append(&protondb_btn);

    // Open Steam Store in browser
    let steam_btn = gtk::Button::builder()
        .label("Steam Store")
        .icon_name("insert-link-symbolic")
        .css_classes(["pill"])
        .tooltip_text("Open official Steam store page")
        .build();
    steam_btn.connect_clicked(move |_| {
        let url = format!("https://store.steampowered.com/app/{}", appid);
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    });
    actions_box.append(&steam_btn);

    card.append(&actions_box);
    header_group.add(&card);
    page.add(&header_group);

    // --- Minimum Requirements Comparison Group ---
    let min_group = adw::PreferencesGroup::builder()
        .title("Minimum Requirements Comparison")
        .description("Hardware requirements comparison against your detected system")
        .build();

    if let Some(ref req) = info.reqs {
        let min_checks = ProtonDbClient::check_system_against_reqs(req);
        let list_box = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["boxed-list"])
            .build();

        for check in min_checks {
            let row = adw::ActionRow::builder()
                .title(check.item)
                .subtitle(&check.detail)
                .build();

            let (icon_name, icon_class) = match check.verdict {
                CheckVerdict::Pass => ("emblem-ok-symbolic", "success"),
                CheckVerdict::Fail => ("dialog-error-symbolic", "error"),
                CheckVerdict::Unknown => ("dialog-information-symbolic", "dim-label"),
            };

            let icon = gtk::Image::from_icon_name(icon_name);
            icon.set_css_classes(&[icon_class]);
            row.add_prefix(&icon);
            list_box.append(&row);
        }
        min_group.add(&list_box);
    } else {
        let no_req = adw::ActionRow::builder()
            .title("No Requirements Found")
            .subtitle("Steam store does not provide structured hardware requirements for this title.")
            .build();
        no_req.add_prefix(&gtk::Image::from_icon_name("dialog-information-symbolic"));
        let list_box = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["boxed-list"])
            .build();
        list_box.append(&no_req);
        min_group.add(&list_box);
    }
    page.add(&min_group);

    // --- Recommended Requirements Comparison Group (if available) ---
    if let Some(ref req) = info.reqs {
        if req.rec_ram_gb.is_some() || !req.rec_cpu.is_empty() || !req.rec_gpu.is_empty() {
            let rec_group = adw::PreferencesGroup::builder()
                .title("Recommended Requirements Comparison")
                .description("Hardware targets recommended for optimal performance")
                .build();

            let rec_checks = ProtonDbClient::check_system_against_rec(req);
            let list_box = gtk::ListBox::builder()
                .selection_mode(gtk::SelectionMode::None)
                .css_classes(["boxed-list"])
                .build();

            for check in rec_checks {
                let row = adw::ActionRow::builder()
                    .title(check.item)
                    .subtitle(&check.detail)
                    .build();

                let (icon_name, icon_class) = match check.verdict {
                    CheckVerdict::Pass => ("emblem-ok-symbolic", "success"),
                    CheckVerdict::Fail => ("dialog-error-symbolic", "error"),
                    CheckVerdict::Unknown => ("dialog-information-symbolic", "dim-label"),
                };

                let icon = gtk::Image::from_icon_name(icon_name);
                icon.set_css_classes(&[icon_class]);
                row.add_prefix(&icon);
                list_box.append(&row);
            }
            rec_group.add(&list_box);
            page.add(&rec_group);
        }
    }

    // --- Raw Requirements Text Expander ---
    if let Some(ref req) = info.reqs {
        if !req.minimum_raw.is_empty() || !req.recommended_raw.is_empty() {
            let raw_group = adw::PreferencesGroup::builder()
                .title("Steam Store Requirements Text")
                .build();

            if !req.minimum_raw.is_empty() {
                let exp = adw::ExpanderRow::builder()
                    .title("Raw Minimum Requirements")
                    .subtitle("Original text published on Steam store")
                    .build();
                exp.add_prefix(&gtk::Image::from_icon_name("document-properties-symbolic"));

                let text_label = gtk::Label::builder()
                    .label(&req.minimum_raw)
                    .wrap(true)
                    .halign(gtk::Align::Start)
                    .margin_start(16)
                    .margin_end(16)
                    .margin_top(12)
                    .margin_bottom(12)
                    .build();
                exp.add_row(&text_label);
                raw_group.add(&exp);
            }

            if !req.recommended_raw.is_empty() {
                let exp = adw::ExpanderRow::builder()
                    .title("Raw Recommended Requirements")
                    .subtitle("Original text published on Steam store")
                    .build();
                exp.add_prefix(&gtk::Image::from_icon_name("document-properties-symbolic"));

                let text_label = gtk::Label::builder()
                    .label(&req.recommended_raw)
                    .wrap(true)
                    .halign(gtk::Align::Start)
                    .margin_start(16)
                    .margin_end(16)
                    .margin_top(12)
                    .margin_bottom(12)
                    .build();
                exp.add_row(&text_label);
                raw_group.add(&exp);
            }

            page.add(&raw_group);
        }
    }

    detail_scrolled.set_child(Some(&page));
}
