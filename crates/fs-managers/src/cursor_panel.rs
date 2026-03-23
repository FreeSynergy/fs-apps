/// Cursor Manager panel — sidebar (cursor sets) + detail pane (slot previews + create form).
use dioxus::prelude::*;
use fs_manager_cursor::{CursorManager, CursorRepository, CursorSet, CursorSetDraft, CursorSlot};

// ── View state ────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum View {
    SetDetail(String),
    NewSet,
    Settings,
}

// ── Main panel ────────────────────────────────────────────────────────────────

#[component]
pub fn CursorManagerPanel() -> Element {
    let icons_root = std::path::PathBuf::from(
        std::env::var("FS_ICONS_ROOT").unwrap_or_else(|_| "../FreeSynergy.Icons".into()),
    );
    let mgr = CursorManager::new(icons_root, vec![
        CursorRepository {
            id:      "freesynergy-icons".into(),
            name:    "FreeSynergy Icons".into(),
            url:     "https://github.com/FreeSynergy/Icons".into(),
            enabled: true,
            builtin: true,
        },
    ]);
    let sets = mgr.sets();

    let first_view = sets
        .first()
        .map(|s| View::SetDetail(s.id.clone()))
        .unwrap_or(View::NewSet);

    let mut view     = use_signal(|| first_view);
    let mut active   = use_signal(|| sets.first().map(|s| s.id.clone()).unwrap_or_default());
    let mut feedback = use_signal(String::new);

    rsx! {
        div {
            style: "display: flex; height: 100%; width: 100%; overflow: hidden; \
                    background: var(--fs-color-bg-base);",

            // ── Sidebar ───────────────────────────────────────────────────────
            Sidebar {
                sets: sets.clone(),
                active_id: active.read().clone(),
                view: view.read().clone(),
                on_select_set: move |id: String| {
                    view.set(View::SetDetail(id));
                    feedback.set(String::new());
                },
                on_new_set:  move |_| {
                    view.set(View::NewSet);
                    feedback.set(String::new());
                },
                on_settings: move |_| {
                    view.set(View::Settings);
                    feedback.set(String::new());
                },
            }

            // ── Detail pane ───────────────────────────────────────────────────
            div { style: "flex: 1; overflow-y: auto;",
                match view.read().clone() {
                    View::SetDetail(id) => rsx! {
                        SetDetailView {
                            sets: sets.clone(),
                            set_id: id.clone(),
                            active_id: active.read().clone(),
                            icons_root: std::env::var("FS_ICONS_ROOT")
                                .unwrap_or_else(|_| "../FreeSynergy.Icons".into()),
                            on_activate: move |_| {
                                active.set(id.clone());
                                feedback.set(format!("Cursor set activated."));
                            },
                        }
                    },
                    View::NewSet => rsx! {
                        NewSetView {
                            icons_root: std::env::var("FS_ICONS_ROOT")
                                .unwrap_or_else(|_| "../FreeSynergy.Icons".into()),
                            feedback: feedback.read().clone(),
                            on_saved: move |id: String| {
                                feedback.set(format!("Cursor set \"{id}\" saved."));
                                view.set(View::SetDetail(id));
                            },
                        }
                    },
                    View::Settings => rsx! {
                        SettingsView {}
                    },
                }
            }
        }
    }
}

// ── Sidebar ───────────────────────────────────────────────────────────────────

#[component]
fn Sidebar(
    sets: Vec<CursorSet>,
    active_id: String,
    view: View,
    on_select_set: EventHandler<String>,
    on_new_set: EventHandler<()>,
    on_settings: EventHandler<()>,
) -> Element {
    let sidebar_style = "width: 220px; flex-shrink: 0; display: flex; flex-direction: column; \
                         overflow: hidden; \
                         background: var(--fs-color-bg-surface, #0f172a); \
                         border-right: 1px solid var(--fs-color-border-default, #334155);";

    rsx! {
        div { style: "{sidebar_style}",

            // Active set preview chip
            if !active_id.is_empty() {
                if let Some(set) = sets.iter().find(|s| s.id == active_id) {
                    div {
                        style: "padding: 10px 14px; \
                                border-bottom: 1px solid var(--fs-color-border-default, #334155); \
                                background: rgba(6,182,212,0.07);",
                        div {
                            style: "font-size: 10px; font-weight: 600; letter-spacing: 0.05em; \
                                    text-transform: uppercase; color: var(--fs-color-primary, #06b6d4); \
                                    margin-bottom: 4px;",
                            "Active Set"
                        }
                        div {
                            style: "font-size: 13px; font-weight: 500; \
                                    color: var(--fs-color-text-primary); \
                                    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                            "◉  {set.name}"
                        }
                    }
                }
            }

            // Set list (scrollable)
            div { style: "flex: 1; overflow-y: auto; padding: 8px 0;",
                div {
                    style: "padding: 8px 16px 4px; font-size: 11px; font-weight: 600; \
                            letter-spacing: 0.05em; text-transform: uppercase; \
                            color: var(--fs-color-text-muted);",
                    "Installed Sets"
                }

                if sets.is_empty() {
                    div {
                        style: "padding: 12px 16px; font-size: 13px; \
                                color: var(--fs-color-text-muted);",
                        "No cursor sets found."
                    }
                } else {
                    for set in &sets {
                        {
                            let set_id   = set.id.clone();
                            let is_sel   = view == View::SetDetail(set_id.clone());
                            let is_act   = set_id == active_id;
                            let bg = if is_sel {
                                "background: var(--fs-sidebar-active-bg, rgba(6,182,212,0.15)); \
                                 color: var(--fs-color-primary, #06b6d4);"
                            } else {
                                "background: transparent; color: var(--fs-color-text-primary);"
                            };
                            rsx! {
                                div {
                                    key: "{set_id}",
                                    style: "display: flex; align-items: center; gap: 10px; \
                                            padding: 9px 16px; cursor: pointer; \
                                            transition: background 100ms; {bg}",
                                    onclick: move |_| on_select_set.call(set_id.clone()),
                                    span { style: "font-size: 15px; flex-shrink: 0;",
                                        if is_act { "◉" } else { "○" }
                                    }
                                    div { style: "flex: 1; min-width: 0;",
                                        div {
                                            style: "font-size: 13px; font-weight: 500; \
                                                    white-space: nowrap; overflow: hidden; \
                                                    text-overflow: ellipsis;",
                                            "{set.name}"
                                        }
                                        div {
                                            style: "font-size: 11px; opacity: 0.55; margin-top: 1px;",
                                            "{set.present_slots.len()} / 31 slots"
                                        }
                                    }
                                    if set.builtin {
                                        span {
                                            style: "padding: 1px 5px; font-size: 9px; \
                                                    background: var(--fs-color-primary, #06b6d4); \
                                                    color: #fff; border-radius: 999px; flex-shrink: 0;",
                                            "✦"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // New Set nav item
                {
                    let is_sel = view == View::NewSet;
                    let bg = if is_sel {
                        "background: var(--fs-sidebar-active-bg, rgba(6,182,212,0.15)); \
                         color: var(--fs-color-primary, #06b6d4);"
                    } else {
                        "background: transparent; color: var(--fs-color-text-primary);"
                    };
                    rsx! {
                        div {
                            style: "display: flex; align-items: center; gap: 10px; \
                                    padding: 9px 16px; cursor: pointer; margin-top: 4px; \
                                    transition: background 100ms; {bg}",
                            onclick: move |_| on_new_set.call(()),
                            span { style: "font-size: 15px;", "＋" }
                            span { style: "font-size: 13px; font-weight: 500;", "New Set" }
                        }
                    }
                }
            }

            // Settings (pinned bottom)
            {
                let is_sel = view == View::Settings;
                let bg = if is_sel {
                    "background: var(--fs-sidebar-active-bg, rgba(6,182,212,0.15)); \
                     color: var(--fs-color-primary, #06b6d4);"
                } else {
                    "background: transparent; color: var(--fs-color-text-muted);"
                };
                rsx! {
                    div {
                        style: "border-top: 1px solid var(--fs-color-border-default, #334155);",
                        div {
                            style: "display: flex; align-items: center; gap: 10px; \
                                    padding: 10px 16px; cursor: pointer; \
                                    transition: background 100ms; {bg}",
                            onclick: move |_| on_settings.call(()),
                            span { style: "font-size: 14px;", "⚙" }
                            span { style: "font-size: 13px;", "Repositories" }
                        }
                    }
                }
            }
        }
    }
}

// ── Set detail view ───────────────────────────────────────────────────────────

#[component]
fn SetDetailView(
    sets: Vec<CursorSet>,
    set_id: String,
    active_id: String,
    icons_root: String,
    on_activate: EventHandler<()>,
) -> Element {
    let Some(set) = sets.iter().find(|s| s.id == set_id).cloned() else {
        return rsx! {
            div { style: "padding: 40px; color: var(--fs-color-text-muted);",
                "Set not found."
            }
        };
    };

    let is_active = set_id == active_id;
    let missing   = set.missing_required();

    // Key preview slots (the 6 most prominent ones)
    let preview_slots = [
        CursorSlot::Default,
        CursorSlot::Pointer,
        CursorSlot::Text,
        CursorSlot::Busy,
        CursorSlot::Grab,
        CursorSlot::NotAllowed,
    ];

    rsx! {
        div { style: "padding: 24px 28px; max-width: 720px;",

            // Header
            div {
                style: "display: flex; align-items: flex-start; justify-content: space-between; \
                        margin-bottom: 20px;",
                div {
                    h3 {
                        style: "margin: 0 0 4px; font-size: 17px; font-weight: 700; \
                                color: var(--fs-color-text-primary);",
                        "{set.name}"
                        if is_active {
                            span {
                                style: "margin-left: 10px; font-size: 10px; padding: 2px 8px; \
                                        background: var(--fs-color-primary, #06b6d4); \
                                        color: #fff; border-radius: 999px; vertical-align: middle;",
                                "ACTIVE"
                            }
                        }
                    }
                    if !set.description.is_empty() {
                        p {
                            style: "margin: 0; font-size: 13px; color: var(--fs-color-text-muted);",
                            "{set.description}"
                        }
                    }
                    div {
                        style: "margin-top: 6px; font-size: 12px; color: var(--fs-color-text-muted);",
                        "v{set.version}  ·  {set.author}"
                    }
                }
                if !is_active {
                    button {
                        style: "padding: 8px 20px; font-size: 13px; font-weight: 600; \
                                background: var(--fs-color-primary, #06b6d4); color: #fff; \
                                border: none; border-radius: var(--fs-radius-md, 6px); \
                                cursor: pointer; white-space: nowrap;",
                        onclick: move |_| on_activate.call(()),
                        "Activate"
                    }
                }
            }

            // Completeness badge
            if !missing.is_empty() {
                div {
                    style: "margin-bottom: 16px; padding: 10px 14px; font-size: 12px; \
                            background: rgba(234,179,8,0.1); \
                            border: 1px solid rgba(234,179,8,0.3); \
                            border-radius: var(--fs-radius-md, 6px); \
                            color: #ca8a04;",
                    "⚠  Missing {missing.len()} required slot(s): "
                    span { style: "opacity: 0.8;",
                        {missing.iter().map(|s| s.filename()).collect::<Vec<_>>().join(", ")}
                    }
                }
            }

            // Key preview row
            div {
                style: "margin-bottom: 24px; padding: 16px; \
                        background: var(--fs-color-bg-overlay, #1e293b); \
                        border-radius: var(--fs-radius-md, 6px);",
                div {
                    style: "font-size: 11px; font-weight: 600; letter-spacing: 0.05em; \
                            text-transform: uppercase; color: var(--fs-color-text-muted); \
                            margin-bottom: 12px;",
                    "Preview"
                }
                div {
                    style: "display: flex; gap: 12px; flex-wrap: wrap;",
                    for slot in &preview_slots {
                        {
                            let filename = slot.filename();
                            let svg_path = format!(
                                "file://{}/cursor-sets/{}/{}.svg",
                                icons_root, set_id, filename
                            );
                            let has_slot = set.present_slots.contains(slot);
                            rsx! {
                                div {
                                    key: "{filename}",
                                    style: "display: flex; flex-direction: column; align-items: center; \
                                            gap: 5px; padding: 10px 8px; width: 72px; \
                                            border-radius: var(--fs-radius-md, 6px); \
                                            background: var(--fs-color-bg-surface, #0f172a);",
                                    if has_slot {
                                        img {
                                            src: "{svg_path}",
                                            style: "width: 32px; height: 32px; object-fit: contain;",
                                            alt: "{filename}",
                                        }
                                    } else {
                                        div {
                                            style: "width: 32px; height: 32px; display: flex; \
                                                    align-items: center; justify-content: center; \
                                                    font-size: 18px; opacity: 0.25;",
                                            "✕"
                                        }
                                    }
                                    span {
                                        style: "font-size: 10px; color: var(--fs-color-text-muted); \
                                                text-align: center; word-break: break-all;",
                                        "{filename}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // All 31 slots grid
            div {
                style: "font-size: 11px; font-weight: 600; letter-spacing: 0.05em; \
                        text-transform: uppercase; color: var(--fs-color-text-muted); \
                        margin-bottom: 12px;",
                "All Slots  ({set.present_slots.len()} / 31)"
            }
            div {
                style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(80px, 1fr)); \
                        gap: 6px;",
                for slot in CursorSlot::all() {
                    {
                        let filename = slot.filename();
                        let svg_path = format!(
                            "file://{}/cursor-sets/{}/{}.svg",
                            icons_root, set_id, filename
                        );
                        let present = set.present_slots.contains(slot);
                        let opacity = if present { "1" } else { "0.3" };
                        rsx! {
                            div {
                                key: "{filename}",
                                style: "display: flex; flex-direction: column; align-items: center; \
                                        gap: 4px; padding: 8px 4px; \
                                        border-radius: var(--fs-radius-md, 6px); \
                                        background: var(--fs-color-bg-overlay, #1e293b); \
                                        opacity: {opacity};",
                                if present {
                                    img {
                                        src: "{svg_path}",
                                        style: "width: 28px; height: 28px; object-fit: contain;",
                                        alt: "{filename}",
                                    }
                                } else {
                                    div {
                                        style: "width: 28px; height: 28px; display: flex; \
                                                align-items: center; justify-content: center; \
                                                font-size: 14px;",
                                        "✕"
                                    }
                                }
                                span {
                                    style: "font-size: 9px; color: var(--fs-color-text-muted); \
                                            text-align: center; word-break: break-all; line-height: 1.2;",
                                    "{filename}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── New Set form ──────────────────────────────────────────────────────────────

#[component]
fn NewSetView(
    icons_root: String,
    feedback: String,
    on_saved: EventHandler<String>,
) -> Element {
    let mut f_id          = use_signal(String::new);
    let mut f_name        = use_signal(String::new);
    let mut f_description = use_signal(String::new);
    let mut f_author      = use_signal(String::new);
    let mut f_version     = use_signal(|| "1.0.0".to_string());
    let mut error         = use_signal(String::new);

    // Per-slot: SVG path + hotspot X/Y
    // Stored as parallel arrays indexed by CursorSlot::all() order.
    let slot_count = CursorSlot::all().len();
    let mut svg_paths  = use_signal(|| vec![String::new(); slot_count]);
    let mut hotspot_xs = use_signal(|| {
        CursorSlot::all().iter().map(|s| s.default_hotspot().0.to_string()).collect::<Vec<_>>()
    });
    let mut hotspot_ys = use_signal(|| {
        CursorSlot::all().iter().map(|s| s.default_hotspot().1.to_string()).collect::<Vec<_>>()
    });

    let icons_root_cloned = icons_root.clone();

    rsx! {
        div { style: "padding: 24px 28px; max-width: 680px;",

            h3 {
                style: "margin: 0 0 4px; font-size: 17px; font-weight: 700; \
                        color: var(--fs-color-text-primary);",
                "New Cursor Set"
            }
            p {
                style: "margin: 0 0 24px; font-size: 13px; color: var(--fs-color-text-muted);",
                "Fill in the metadata and provide SVG paths for each cursor slot."
            }

            // Feedback / error
            if !feedback.is_empty() {
                div {
                    style: "margin-bottom: 16px; padding: 10px 14px; font-size: 13px; \
                            background: rgba(6,182,212,0.1); \
                            border: 1px solid rgba(6,182,212,0.3); \
                            border-radius: var(--fs-radius-md, 6px); \
                            color: var(--fs-color-primary, #06b6d4);",
                    "✓  {feedback}"
                }
            }
            if !error.read().is_empty() {
                div {
                    style: "margin-bottom: 16px; padding: 10px 14px; font-size: 13px; \
                            background: rgba(239,68,68,0.1); \
                            border: 1px solid rgba(239,68,68,0.3); \
                            border-radius: var(--fs-radius-md, 6px); color: #ef4444;",
                    "{error}"
                }
            }

            // ── Metadata ──────────────────────────────────────────────────────
            div {
                style: "display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 24px;",

                FormField { label: "ID (slug) *",
                    input {
                        style: "{INPUT_STYLE}",
                        placeholder: "my-cursor-set",
                        value: "{f_id}",
                        oninput: move |e| f_id.set(e.value()),
                    }
                }
                FormField { label: "Name *",
                    input {
                        style: "{INPUT_STYLE}",
                        placeholder: "My Cursor Set",
                        value: "{f_name}",
                        oninput: move |e| f_name.set(e.value()),
                    }
                }
                FormField { label: "Author",
                    input {
                        style: "{INPUT_STYLE}",
                        placeholder: "Your Name",
                        value: "{f_author}",
                        oninput: move |e| f_author.set(e.value()),
                    }
                }
                FormField { label: "Version *",
                    input {
                        style: "{INPUT_STYLE}",
                        placeholder: "1.0.0",
                        value: "{f_version}",
                        oninput: move |e| f_version.set(e.value()),
                    }
                }
                div { style: "grid-column: 1 / -1;",
                    FormField { label: "Description",
                        textarea {
                            style: "{INPUT_STYLE} height: 60px; resize: vertical;",
                            placeholder: "Short description of this cursor set…",
                            value: "{f_description}",
                            oninput: move |e| f_description.set(e.value()),
                        }
                    }
                }
            }

            // ── Slot table ────────────────────────────────────────────────────
            div {
                style: "font-size: 11px; font-weight: 600; letter-spacing: 0.05em; \
                        text-transform: uppercase; color: var(--fs-color-text-muted); \
                        margin-bottom: 10px;",
                "Cursor Slots"
            }
            div {
                style: "margin-bottom: 4px; display: grid; \
                        grid-template-columns: 140px 1fr 56px 56px; \
                        gap: 6px; padding: 6px 8px; \
                        background: var(--fs-color-bg-overlay, #1e293b); \
                        border-radius: var(--fs-radius-md, 6px) var(--fs-radius-md, 6px) 0 0;",
                span { style: "font-size: 11px; color: var(--fs-color-text-muted); font-weight: 600;", "Slot" }
                span { style: "font-size: 11px; color: var(--fs-color-text-muted); font-weight: 600;", "SVG Path" }
                span { style: "font-size: 11px; color: var(--fs-color-text-muted); font-weight: 600;", "Hotspot X" }
                span { style: "font-size: 11px; color: var(--fs-color-text-muted); font-weight: 600;", "Hotspot Y" }
            }
            div {
                style: "border: 1px solid var(--fs-color-border-default, #334155); \
                        border-radius: 0 0 var(--fs-radius-md, 6px) var(--fs-radius-md, 6px); \
                        overflow: hidden;",
                for (i, slot) in CursorSlot::all().iter().enumerate() {
                    {
                        let filename = slot.filename();
                        let is_required = CursorSlot::minimum_required().contains(slot);
                        let row_bg = if i % 2 == 0 {
                            "background: var(--fs-color-bg-surface, #0f172a);"
                        } else {
                            "background: var(--fs-color-bg-overlay, #1e293b);"
                        };
                        rsx! {
                            div {
                                key: "{filename}",
                                style: "display: grid; grid-template-columns: 140px 1fr 56px 56px; \
                                        gap: 6px; padding: 6px 8px; align-items: center; {row_bg}",
                                div { style: "display: flex; align-items: center; gap: 6px;",
                                    span {
                                        style: "font-size: 12px; font-family: monospace; \
                                                color: var(--fs-color-text-primary);",
                                        "{filename}"
                                    }
                                    if is_required {
                                        span {
                                            style: "font-size: 9px; padding: 1px 4px; \
                                                    background: rgba(6,182,212,0.2); \
                                                    color: var(--fs-color-primary, #06b6d4); \
                                                    border-radius: 999px;",
                                            "req"
                                        }
                                    }
                                }
                                input {
                                    style: "{INPUT_STYLE} font-size: 11px; padding: 4px 8px;",
                                    placeholder: "/path/to/{filename}.svg",
                                    value: "{svg_paths.read()[i]}",
                                    oninput: move |e| {
                                        svg_paths.write()[i] = e.value();
                                    },
                                }
                                input {
                                    style: "{INPUT_STYLE} font-size: 11px; padding: 4px 6px; text-align: center;",
                                    r#type: "number",
                                    value: "{hotspot_xs.read()[i]}",
                                    oninput: move |e| {
                                        hotspot_xs.write()[i] = e.value();
                                    },
                                }
                                input {
                                    style: "{INPUT_STYLE} font-size: 11px; padding: 4px 6px; text-align: center;",
                                    r#type: "number",
                                    value: "{hotspot_ys.read()[i]}",
                                    oninput: move |e| {
                                        hotspot_ys.write()[i] = e.value();
                                    },
                                }
                            }
                        }
                    }
                }
            }

            // ── Save button ───────────────────────────────────────────────────
            div { style: "margin-top: 24px; display: flex; justify-content: flex-end;",
                button {
                    style: "padding: 9px 24px; font-size: 14px; font-weight: 600; \
                            background: var(--fs-color-primary, #06b6d4); color: #fff; \
                            border: none; border-radius: var(--fs-radius-md, 6px); \
                            cursor: pointer;",
                    onclick: move |_| {
                        let id      = f_id.read().trim().to_string();
                        let name    = f_name.read().trim().to_string();
                        let version = f_version.read().trim().to_string();

                        if id.is_empty() || name.is_empty() || version.is_empty() {
                            error.set("ID, Name, and Version are required.".into());
                            return;
                        }

                        // Build draft
                        let mut draft = CursorSetDraft {
                            id:          id.clone(),
                            name,
                            description: f_description.read().trim().to_string(),
                            author:      f_author.read().trim().to_string(),
                            version,
                            ..Default::default()
                        };

                        let paths  = svg_paths.read().clone();
                        let xs     = hotspot_xs.read().clone();
                        let ys     = hotspot_ys.read().clone();
                        let slots  = CursorSlot::all();

                        for (i, slot) in slots.iter().enumerate() {
                            let path = paths[i].trim().to_string();
                            if path.is_empty() { continue; }
                            match std::fs::read_to_string(&path) {
                                Ok(svg) => {
                                    draft.slots.push((*slot, svg));
                                    let x: u32 = xs[i].parse().unwrap_or(slot.default_hotspot().0);
                                    let y: u32 = ys[i].parse().unwrap_or(slot.default_hotspot().1);
                                    if (x, y) != slot.default_hotspot() {
                                        draft.hotspot_overrides.push((*slot, (x, y)));
                                    }
                                }
                                Err(e) => {
                                    error.set(format!("Cannot read {path}: {e}"));
                                    return;
                                }
                            }
                        }

                        let mgr = CursorManager::new(
                            std::path::PathBuf::from(&icons_root_cloned),
                            vec![],
                        );
                        match mgr.save_draft(&draft, "freesynergy-icons") {
                            Ok(_)  => {
                                error.set(String::new());
                                on_saved.call(id);
                            }
                            Err(e) => error.set(format!("{e}")),
                        }
                    },
                    "Save Cursor Set"
                }
            }
        }
    }
}

// ── Settings view ─────────────────────────────────────────────────────────────

#[component]
fn SettingsView() -> Element {
    // For now shows the builtin repo; editable repo management can be added later.
    rsx! {
        div { style: "padding: 24px 28px; max-width: 560px;",
            h3 {
                style: "margin: 0 0 4px; font-size: 17px; font-weight: 700; \
                        color: var(--fs-color-text-primary);",
                "Cursor Repositories"
            }
            p {
                style: "margin: 0 0 20px; font-size: 13px; color: var(--fs-color-text-muted);",
                "Repositories are sources for cursor sets. \
                 Builtin repositories (✦) can be disabled but not deleted."
            }

            // Builtin repo row
            div {
                style: "border: 1px solid var(--fs-color-border-default, #334155); \
                        border-radius: var(--fs-radius-md, 6px); overflow: hidden;",
                div {
                    style: "display: flex; align-items: center; gap: 12px; \
                            padding: 12px 16px; \
                            background: var(--fs-color-bg-overlay, #1e293b);",
                    span {
                        style: "font-size: 13px; padding: 2px 6px; \
                                background: var(--fs-color-primary, #06b6d4); \
                                color: #fff; border-radius: 999px; font-size: 10px;",
                        "✦"
                    }
                    div { style: "flex: 1;",
                        div {
                            style: "font-size: 13px; font-weight: 500; \
                                    color: var(--fs-color-text-primary);",
                            "FreeSynergy Icons"
                        }
                        div {
                            style: "font-size: 11px; color: var(--fs-color-text-muted); margin-top: 2px;",
                            "github.com/FreeSynergy/Icons"
                        }
                    }
                    span {
                        style: "font-size: 11px; padding: 3px 10px; \
                                background: rgba(6,182,212,0.15); \
                                color: var(--fs-color-primary, #06b6d4); \
                                border-radius: 999px;",
                        "enabled"
                    }
                }
            }

            // Add repo placeholder
            div { style: "margin-top: 16px;",
                button {
                    style: "padding: 8px 16px; font-size: 13px; \
                            background: transparent; \
                            border: 1px solid var(--fs-color-border-default, #334155); \
                            border-radius: var(--fs-radius-md, 6px); \
                            color: var(--fs-color-text-muted); cursor: pointer;",
                    "＋  Add Repository"
                }
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

#[component]
fn FormField(label: &'static str, children: Element) -> Element {
    rsx! {
        div {
            label {
                style: "display: block; font-size: 12px; font-weight: 500; \
                        color: var(--fs-color-text-muted); margin-bottom: 5px;",
                "{label}"
            }
            {children}
        }
    }
}

const INPUT_STYLE: &str =
    "width: 100%; box-sizing: border-box; padding: 7px 10px; font-size: 13px; \
     background: var(--fs-color-bg-overlay, #1e293b); \
     border: 1px solid var(--fs-color-border-default, #334155); \
     border-radius: var(--fs-radius-md, 6px); \
     color: var(--fs-color-text-primary); outline: none;";
