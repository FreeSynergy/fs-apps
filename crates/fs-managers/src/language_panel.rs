// language_panel.rs — Language Manager panel.
//
// Layout: LayoutB (sidebar + detail pane).
//
// Left sidebar:
//   - Active language chip
//   - List of subscribed languages (with flag SVG + direction indicator)
//   - "＋ Subscribe" nav item — opens inline subscription picker
//   - "⚙ Formats" (pinned bottom) — opens locale format settings
//
// Right detail pane:
//   Three tabs consistent with ManagerView:
//   - Info    : language metadata (name, direction, script, family, continent,
//               installed pack count), [Set Active] button
//   - Config  : locale format settings (date / time / number / fallback / auto-update)
//   - Builder : translation editor placeholder (with contributor SSH status badge)
//
// OOP conventions:
//   - View enum replaces match-on-string navigation
//   - Tab enum replaces match-on-index tab tracking
//   - Sub-components for Sidebar, each tab, and the subscribe picker

use dioxus::prelude::*;
use fs_i18n;
use fs_manager_language::{
    DateFormat, FormatVariant, HasFlag, LanguageManager, NumberFormat, TimeFormat,
};

// ── View / Tab enums ──────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
enum View {
    /// Detail pane for a subscribed language.
    Language(String),
    /// Inline subscription picker — add a new language.
    Subscribe,
    /// Global locale format settings (pinned bottom).
    Formats,
}

#[derive(Clone, PartialEq, Debug, Default)]
enum Tab {
    #[default]
    Info,
    Config,
    Builder,
}

impl Tab {
    fn label(&self) -> String {
        match self {
            Self::Info    => fs_i18n::t("managers.language.tab_info").to_string(),
            Self::Config  => fs_i18n::t("managers.language.tab_config").to_string(),
            Self::Builder => fs_i18n::t("managers.language.tab_builder").to_string(),
        }
    }
}

// ── Root component ────────────────────────────────────────────────────────────

#[component]
pub fn LanguageManagerPanel() -> Element {
    let mgr       = LanguageManager::new();
    let active_id = mgr.active().id.clone();
    let subscribed = mgr.available(); // subscribed + built-ins + installed packs

    // Select the active language as the initial view
    let initial_view = if subscribed.iter().any(|l| l.id == active_id) {
        View::Language(active_id.clone())
    } else {
        subscribed.first().map(|l| View::Language(l.id.clone())).unwrap_or(View::Subscribe)
    };

    let mut view     = use_signal(|| initial_view);
    let mut active   = use_signal(|| active_id);
    let mut feedback = use_signal(String::new);

    rsx! {
        div {
            style: "display: flex; height: 100%; width: 100%; overflow: hidden; \
                    background: var(--fs-color-bg-base);",

            // ── Left sidebar ─────────────────────────────────────────────────
            LanguageSidebar {
                subscribed: subscribed.clone(),
                active_id:  active.read().clone(),
                view:       view.read().clone(),
                on_select:  move |id: String| {
                    view.set(View::Language(id));
                    feedback.set(String::new());
                },
                on_subscribe: move |_| {
                    view.set(View::Subscribe);
                    feedback.set(String::new());
                },
                on_formats: move |_| {
                    view.set(View::Formats);
                    feedback.set(String::new());
                },
            }

            // ── Right detail pane ────────────────────────────────────────────
            div { style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",
                match view.read().clone() {
                    View::Language(ref id) => {
                        let lang = subscribed.iter().find(|l| &l.id == id).cloned();
                        let active_id_clone = active.read().clone();
                        let id_clone = id.clone();
                        rsx! {
                            LanguageDetailPane {
                                lang,
                                active_id: active_id_clone,
                                feedback: feedback.read().clone(),
                                on_set_active: move |_| {
                                    let _ = LanguageManager::new().set_active(&id_clone);
                                    active.set(id_clone.clone());
                                    feedback.set(
                                        fs_i18n::t("managers.saved").to_string()
                                    );
                                },
                                on_unsubscribe: move |code: String| {
                                    let _ = LanguageManager::new().unsubscribe(&code);
                                    view.set(View::Subscribe);
                                },
                            }
                        }
                    },
                    View::Subscribe => rsx! {
                        SubscribeView {
                            subscribed_ids: subscribed.iter().map(|l| l.id.clone()).collect(),
                            on_subscribed: move |code: String| {
                                let _ = LanguageManager::new().subscribe(&code);
                                view.set(View::Language(code));
                            },
                        }
                    },
                    View::Formats => rsx! {
                        FormatsView {}
                    },
                }
            }
        }
    }
}

// ── Sidebar ───────────────────────────────────────────────────────────────────

#[component]
fn LanguageSidebar(
    subscribed:   Vec<fs_manager_language::Language>,
    active_id:    String,
    view:         View,
    on_select:    EventHandler<String>,
    on_subscribe: EventHandler<()>,
    on_formats:   EventHandler<()>,
) -> Element {
    let sidebar_style =
        "width: 220px; flex-shrink: 0; display: flex; flex-direction: column; \
         overflow: hidden; \
         background: var(--fs-color-bg-surface, #0f172a); \
         border-right: 1px solid var(--fs-color-border-default, #334155);";

    rsx! {
        div { style: "{sidebar_style}",

            // Active language chip
            if !active_id.is_empty() {
                if let Some(lang) = subscribed.iter().find(|l| l.id == active_id) {
                    div {
                        style: "padding: 10px 14px; \
                                border-bottom: 1px solid var(--fs-color-border-default, #334155); \
                                background: rgba(6,182,212,0.07);",
                        div {
                            style: "font-size: 10px; font-weight: 600; letter-spacing: 0.05em; \
                                    text-transform: uppercase; \
                                    color: var(--fs-color-primary, #06b6d4); margin-bottom: 4px;",
                            {fs_i18n::t("managers.language.active_label")}
                        }
                        div {
                            style: "display: flex; align-items: center; gap: 8px;",
                            if !lang.flag_svg().is_empty() {
                                span {
                                    style: "flex-shrink: 0; width: 20px; height: 12px; \
                                            display: inline-flex; align-items: center; \
                                            border-radius: 2px; overflow: hidden;",
                                    dangerous_inner_html: "{lang.flag_svg()}",
                                }
                            }
                            span {
                                style: "font-size: 13px; font-weight: 500; \
                                        color: var(--fs-color-text-primary);",
                                "{lang.display_name}"
                            }
                        }
                    }
                }
            }

            // Subscribed list (scrollable)
            div { style: "flex: 1; overflow-y: auto; padding: 8px 0;",
                div {
                    style: "padding: 8px 16px 4px; font-size: 11px; font-weight: 600; \
                            letter-spacing: 0.05em; text-transform: uppercase; \
                            color: var(--fs-color-text-muted);",
                    {fs_i18n::t("managers.language.subscribed_section")}
                }

                if subscribed.is_empty() {
                    div {
                        style: "padding: 12px 16px; font-size: 13px; \
                                color: var(--fs-color-text-muted);",
                        {fs_i18n::t("managers.language.no_subscriptions")}
                    }
                } else {
                    for lang in &subscribed {
                        {
                            let lang_id   = lang.id.clone();
                            let is_sel    = view == View::Language(lang_id.clone());
                            let is_active = lang_id == active_id;
                            let flag_html = lang.flag_svg().to_string();
                            let name      = lang.display_name.clone();
                            let dir_rtl   = lang.meta().map(|m| m.is_rtl()).unwrap_or(false);
                            let bg = if is_sel {
                                "background: var(--fs-sidebar-active-bg, rgba(6,182,212,0.15)); \
                                 color: var(--fs-color-primary, #06b6d4);"
                            } else {
                                "background: transparent; color: var(--fs-color-text-primary);"
                            };
                            rsx! {
                                div {
                                    key: "{lang_id}",
                                    style: "display: flex; align-items: center; gap: 10px; \
                                            padding: 9px 16px; cursor: pointer; \
                                            transition: background 100ms; {bg}",
                                    onclick: move |_| on_select.call(lang_id.clone()),

                                    // Active indicator dot
                                    span {
                                        style: "font-size: 13px; flex-shrink: 0;",
                                        if is_active { "◉" } else { "○" }
                                    }

                                    // Flag
                                    if !flag_html.is_empty() {
                                        span {
                                            style: "flex-shrink: 0; width: 20px; height: 12px; \
                                                    display: inline-flex; align-items: center; \
                                                    border-radius: 2px; overflow: hidden;",
                                            dangerous_inner_html: "{flag_html}",
                                        }
                                    }

                                    // Name
                                    div { style: "flex: 1; min-width: 0;",
                                        div {
                                            style: "font-size: 13px; font-weight: 500; \
                                                    white-space: nowrap; overflow: hidden; \
                                                    text-overflow: ellipsis;",
                                            "{name}"
                                        }
                                        if dir_rtl {
                                            div {
                                                style: "font-size: 10px; color: var(--fs-color-text-muted); \
                                                        margin-top: 1px;",
                                                "RTL"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Subscribe nav item
                {
                    let is_sel = view == View::Subscribe;
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
                                    border-top: 1px solid var(--fs-color-border-default, #334155); \
                                    transition: background 100ms; {bg}",
                            onclick: move |_| on_subscribe.call(()),
                            span { style: "font-size: 14px;", "＋" }
                            span {
                                style: "font-size: 13px; font-weight: 500;",
                                {fs_i18n::t("managers.language.subscribe_btn")}
                            }
                        }
                    }
                }
            }

            // Formats (pinned bottom)
            {
                let is_sel = view == View::Formats;
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
                            onclick: move |_| on_formats.call(()),
                            span { style: "font-size: 14px;", "⚙" }
                            span { style: "font-size: 13px;",
                                {fs_i18n::t("managers.language.formats_title")}
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Language detail pane ──────────────────────────────────────────────────────

#[component]
fn LanguageDetailPane(
    lang:           Option<fs_manager_language::Language>,
    active_id:      String,
    feedback:       String,
    on_set_active:  EventHandler<()>,
    on_unsubscribe: EventHandler<String>,
) -> Element {
    let Some(lang) = lang else {
        return rsx! {
            div {
                style: "display: flex; align-items: center; justify-content: center; \
                        height: 100%; color: var(--fs-color-text-muted); font-size: 13px;",
                "Language not found."
            }
        };
    };

    let mut active_tab = use_signal(Tab::default);
    let is_active      = lang.id == active_id;
    let has_flag       = !lang.flag_svg().is_empty();
    let flag_html      = lang.flag_svg().to_string();
    let meta           = lang.meta();

    rsx! {
        div { style: "display: flex; flex-direction: column; height: 100%; overflow: hidden;",

            // ── Tab bar ───────────────────────────────────────────────────────
            div {
                style: "flex-shrink: 0; display: flex; gap: 0; \
                        border-bottom: 1px solid var(--fs-color-border-default, #334155); \
                        background: var(--fs-color-bg-surface, #0f172a);",

                for tab in [Tab::Info, Tab::Config, Tab::Builder] {
                    {
                        let tab_clone = tab.clone();
                        let is_sel    = *active_tab.read() == tab;
                        let style = if is_sel {
                            "padding: 10px 20px; font-size: 13px; font-weight: 500; \
                             cursor: pointer; background: none; \
                             border: none; border-bottom: 2px solid var(--fs-color-primary, #06b6d4); \
                             color: var(--fs-color-primary, #06b6d4); user-select: none;"
                        } else {
                            "padding: 10px 20px; font-size: 13px; font-weight: 500; \
                             cursor: pointer; background: none; \
                             border: none; border-bottom: 2px solid transparent; \
                             color: var(--fs-color-text-muted); user-select: none;"
                        };
                        rsx! {
                            button {
                                key: "{tab_clone:?}",
                                style: "{style}",
                                onclick: move |_| active_tab.set(tab_clone.clone()),
                                "{tab.label()}"
                            }
                        }
                    }
                }
            }

            // ── Tab content ───────────────────────────────────────────────────
            div { style: "flex: 1; overflow-y: auto; padding: 24px 28px;",

                // Feedback strip
                if !feedback.is_empty() {
                    div {
                        style: "margin-bottom: 16px; padding: 8px 14px; font-size: 12px; \
                                background: rgba(6,182,212,0.1); \
                                border: 1px solid rgba(6,182,212,0.3); \
                                border-radius: var(--fs-radius-md, 6px); \
                                color: var(--fs-color-primary, #06b6d4);",
                        "✓  {feedback}"
                    }
                }

                match *active_tab.read() {
                    Tab::Info => rsx! {
                        InfoTab {
                            lang: lang.clone(),
                            is_active,
                            has_flag,
                            flag_html: flag_html.clone(),
                            meta_name:      meta.map(|m| m.name).unwrap_or(""),
                            meta_script:    meta.map(|m| m.script).unwrap_or(""),
                            meta_family:    meta.map(|m| m.family).unwrap_or(""),
                            meta_continent: meta.map(|m| m.continent).unwrap_or(""),
                            pack_count: LanguageManager::new().registry()
                                            .packs_for_lang(&lang.id).len(),
                            on_set_active: on_set_active.clone(),
                            on_unsubscribe: on_unsubscribe.clone(),
                        }
                    },
                    Tab::Config => rsx! {
                        ConfigTab {}
                    },
                    Tab::Builder => rsx! {
                        BuilderTab {
                            lang_id: lang.id.clone(),
                        }
                    },
                }
            }
        }
    }
}

// ── Info tab ──────────────────────────────────────────────────────────────────

#[component]
fn InfoTab(
    lang:           fs_manager_language::Language,
    is_active:      bool,
    has_flag:       bool,
    flag_html:      String,
    meta_name:      &'static str,
    meta_script:    &'static str,
    meta_family:    &'static str,
    meta_continent: &'static str,
    pack_count:     usize,
    on_set_active:  EventHandler<()>,
    on_unsubscribe: EventHandler<String>,
) -> Element {
    let mgr           = LanguageManager::new();
    let registry      = mgr.registry();
    let installed     = registry.packs_for_lang(&lang.id);
    let lang_id_unsub = lang.id.clone();

    rsx! {
        div {
            // ── Header ────────────────────────────────────────────────────────
            div {
                style: "display: flex; align-items: flex-start; justify-content: space-between; \
                        margin-bottom: 24px; padding-bottom: 20px; \
                        border-bottom: 1px solid var(--fs-color-border-default, #334155);",

                div { style: "display: flex; align-items: center; gap: 16px;",
                    // Flag
                    if has_flag {
                        div {
                            style: "width: 48px; height: 32px; border-radius: 4px; \
                                    overflow: hidden; flex-shrink: 0; \
                                    background: var(--fs-color-bg-overlay, #1e293b); \
                                    display: flex; align-items: center; justify-content: center;",
                            span {
                                style: "width: 100%; height: 100%; display: flex; \
                                        align-items: center; justify-content: center;",
                                dangerous_inner_html: "{flag_html}",
                            }
                        }
                    } else {
                        div {
                            style: "width: 48px; height: 32px; border-radius: 4px; \
                                    background: var(--fs-color-bg-overlay, #1e293b); \
                                    display: flex; align-items: center; justify-content: center; \
                                    font-size: 20px; flex-shrink: 0;",
                            "🌐"
                        }
                    }

                    div {
                        div {
                            style: "display: flex; align-items: center; gap: 8px; margin-bottom: 2px;",
                            h2 {
                                style: "margin: 0; font-size: 18px; font-weight: 700; \
                                        color: var(--fs-color-text-primary);",
                                "{lang.display_name}"
                            }
                            if is_active {
                                span {
                                    style: "font-size: 10px; padding: 2px 8px; \
                                            background: var(--fs-color-primary, #06b6d4); \
                                            color: #fff; border-radius: 999px;",
                                    {fs_i18n::t("managers.language.active_badge")}
                                }
                            }
                        }
                        div {
                            style: "font-size: 13px; color: var(--fs-color-text-muted);",
                            "{lang.id}  ·  {lang.locale}"
                        }
                    }
                }

                // Action buttons
                div { style: "display: flex; gap: 8px; flex-shrink: 0;",
                    if !is_active {
                        button {
                            style: "padding: 7px 16px; font-size: 12px; font-weight: 600; \
                                    background: var(--fs-color-primary, #06b6d4); color: #fff; \
                                    border: none; border-radius: var(--fs-radius-md, 6px); \
                                    cursor: pointer;",
                            onclick: move |_| on_set_active.call(()),
                            {fs_i18n::t("managers.language.set_active_btn")}
                        }
                    }
                    button {
                        style: "padding: 7px 16px; font-size: 12px; font-weight: 500; \
                                background: transparent; color: var(--fs-color-text-muted); \
                                border: 1px solid var(--fs-color-border-default, #334155); \
                                border-radius: var(--fs-radius-md, 6px); cursor: pointer;",
                        onclick: move |_| on_unsubscribe.call(lang_id_unsub.clone()),
                        {fs_i18n::t("managers.language.unsubscribe_btn")}
                    }
                }
            }

            // ── Metadata grid ─────────────────────────────────────────────────
            div {
                style: "display: grid; grid-template-columns: 150px 1fr; \
                        gap: 10px 20px; font-size: 13px; margin-bottom: 28px;",

                MetaRow {
                    label: fs_i18n::t("managers.language.name_label").to_string(),
                    value: if meta_name.is_empty() { lang.display_name.clone() } else { meta_name.to_string() },
                }
                MetaRow {
                    label: fs_i18n::t("managers.language.direction_label").to_string(),
                    value: lang.direction_label().to_string(),
                }
                if !meta_script.is_empty() {
                    MetaRow {
                        label: fs_i18n::t("managers.language.script_label").to_string(),
                        value: meta_script.to_string(),
                    }
                }
                if !meta_family.is_empty() {
                    MetaRow {
                        label: fs_i18n::t("managers.language.family_label").to_string(),
                        value: meta_family.to_string(),
                    }
                }
                if !meta_continent.is_empty() {
                    MetaRow {
                        label: fs_i18n::t("managers.language.continent_label").to_string(),
                        value: meta_continent.to_string(),
                    }
                }
                MetaRow {
                    label: fs_i18n::t("managers.language.packs_section").to_string(),
                    value: pack_count.to_string(),
                }
            }

            // ── Installed packs list ──────────────────────────────────────────
            h3 {
                style: "font-size: 11px; font-weight: 600; letter-spacing: 0.06em; \
                        text-transform: uppercase; color: var(--fs-color-text-muted); \
                        margin: 0 0 10px;",
                {fs_i18n::t("managers.language.packs_section")}
            }

            if installed.is_empty() {
                div {
                    style: "padding: 14px 16px; font-size: 13px; \
                            color: var(--fs-color-text-muted); \
                            background: var(--fs-color-bg-overlay, #1e293b); \
                            border-radius: var(--fs-radius-md, 6px);",
                    {fs_i18n::t("managers.language.no_packs")}
                }
            } else {
                div {
                    style: "border: 1px solid var(--fs-color-border-default, #334155); \
                            border-radius: var(--fs-radius-md, 6px); overflow: hidden;",
                    for pack in &installed {
                        div {
                            key: "{pack.package_id}",
                            style: "display: flex; align-items: center; gap: 12px; \
                                    padding: 10px 14px; font-size: 13px; \
                                    border-bottom: 1px solid var(--fs-color-border-default, #334155);",
                            span { style: "color: var(--fs-color-primary, #06b6d4);", "✓" }
                            span {
                                style: "flex: 1; color: var(--fs-color-text-primary); \
                                        font-family: monospace; font-size: 12px;",
                                "{pack.package_id}"
                            }
                            span {
                                style: "font-size: 11px; color: var(--fs-color-text-muted); \
                                        padding: 1px 6px; \
                                        background: var(--fs-color-bg-overlay, #1e293b); \
                                        border-radius: 999px;",
                                "v{pack.version}"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Metadata row helper ───────────────────────────────────────────────────────

#[component]
fn MetaRow(label: String, value: String) -> Element {
    rsx! {
        span { style: "color: var(--fs-color-text-muted); font-weight: 500;", "{label}" }
        span { style: "color: var(--fs-color-text-primary);", "{value}" }
    }
}

// ── Config tab (locale formats) ───────────────────────────────────────────────

#[component]
fn ConfigTab() -> Element {
    let mgr      = LanguageManager::new();
    let settings = mgr.effective_settings();

    let mut date_fmt   = use_signal(|| settings.date_format.clone());
    let mut time_fmt   = use_signal(|| settings.time_format.clone());
    let mut num_fmt    = use_signal(|| settings.number_format.clone());
    let mut auto_upd   = use_signal(|| settings.auto_update_packs);
    let mut saved      = use_signal(|| false);

    // Example values for preview
    let date_example = date_fmt.read().format(2026, 3, 19);
    let time_example = time_fmt.read().format(14, 30);
    let num_example  = num_fmt.read().format_decimal(1234.56, 2);

    rsx! {
        div { style: "max-width: 540px;",

            h3 {
                style: "font-size: 12px; font-weight: 600; letter-spacing: 0.06em; \
                        text-transform: uppercase; color: var(--fs-color-text-muted); \
                        margin: 0 0 20px;",
                {fs_i18n::t("managers.language.formats_title")}
            }

            // ── Date format ───────────────────────────────────────────────────
            FormatField {
                label: fs_i18n::t("managers.language.date_format_label").to_string(),
                preview: date_example,
                children: rsx! {
                    div { style: "display: flex; gap: 8px; flex-wrap: wrap;",
                        for variant in DateFormat::all() {
                            {
                                let v = variant.clone();
                                let is_sel = *date_fmt.read() == *variant;
                                rsx! {
                                    FormatBtn {
                                        key: "{variant:?}",
                                        label: variant.label().to_string(),
                                        active: is_sel,
                                        onclick: move |_| {
                                            date_fmt.set(v.clone());
                                            saved.set(false);
                                        },
                                    }
                                }
                            }
                        }
                    }
                },
            }

            // ── Time format ───────────────────────────────────────────────────
            FormatField {
                label: fs_i18n::t("managers.language.time_format_label").to_string(),
                preview: time_example,
                children: rsx! {
                    div { style: "display: flex; gap: 8px;",
                        for variant in TimeFormat::all() {
                            {
                                let v = variant.clone();
                                let is_sel = *time_fmt.read() == *variant;
                                rsx! {
                                    FormatBtn {
                                        key: "{variant:?}",
                                        label: variant.label().to_string(),
                                        active: is_sel,
                                        onclick: move |_| {
                                            time_fmt.set(v.clone());
                                            saved.set(false);
                                        },
                                    }
                                }
                            }
                        }
                    }
                },
            }

            // ── Number format ─────────────────────────────────────────────────
            FormatField {
                label: fs_i18n::t("managers.language.number_format_label").to_string(),
                preview: num_example,
                children: rsx! {
                    div { style: "display: flex; gap: 8px; flex-wrap: wrap;",
                        for variant in NumberFormat::all() {
                            {
                                let v = variant.clone();
                                let is_sel = *num_fmt.read() == *variant;
                                rsx! {
                                    FormatBtn {
                                        key: "{variant:?}",
                                        label: variant.label().to_string(),
                                        active: is_sel,
                                        onclick: move |_| {
                                            num_fmt.set(v.clone());
                                            saved.set(false);
                                        },
                                    }
                                }
                            }
                        }
                    }
                },
            }

            // ── Auto-update checkbox ──────────────────────────────────────────
            div {
                style: "display: flex; align-items: center; gap: 10px; \
                        padding: 14px 0; margin-bottom: 20px; \
                        border-bottom: 1px solid var(--fs-color-border-default, #334155);",
                input {
                    r#type: "checkbox",
                    checked: "{auto_upd}",
                    style: "width: 16px; height: 16px; accent-color: var(--fs-color-primary, #06b6d4); cursor: pointer;",
                    onchange: move |e| {
                        auto_upd.set(e.checked());
                        saved.set(false);
                    },
                }
                label {
                    style: "font-size: 13px; color: var(--fs-color-text-primary); cursor: pointer;",
                    {fs_i18n::t("managers.language.auto_update_label")}
                }
            }

            // ── Save button ───────────────────────────────────────────────────
            div { style: "display: flex; align-items: center; gap: 12px;",
                button {
                    style: "padding: 8px 24px; background: var(--fs-color-primary, #06b6d4); \
                            color: white; border: none; \
                            border-radius: var(--fs-radius-md, 6px); \
                            cursor: pointer; font-size: 13px; font-weight: 600;",
                    onclick: move |_| {
                        let mut inv = fs_manager_language::LocaleSettings::load_inventory();
                        inv.date_format   = Some(date_fmt.read().clone());
                        inv.time_format   = Some(time_fmt.read().clone());
                        inv.number_format = Some(num_fmt.read().clone());
                        inv.auto_update_packs = Some(*auto_upd.read());
                        let _ = inv.save_inventory();
                        saved.set(true);
                    },
                    {fs_i18n::t("actions.apply")}
                }
                if *saved.read() {
                    span {
                        style: "font-size: 12px; color: var(--fs-color-text-muted);",
                        {fs_i18n::t("managers.saved")}
                    }
                }
            }
        }
    }
}

// ── FormatField helper ────────────────────────────────────────────────────────

#[component]
fn FormatField(label: String, preview: String, children: Element) -> Element {
    rsx! {
        div {
            style: "padding: 16px 0; \
                    border-bottom: 1px solid var(--fs-color-border-default, #334155);",
            div {
                style: "display: flex; align-items: center; justify-content: space-between; \
                        margin-bottom: 10px;",
                span {
                    style: "font-size: 13px; font-weight: 500; color: var(--fs-color-text-primary);",
                    "{label}"
                }
                span {
                    style: "font-size: 12px; color: var(--fs-color-primary, #06b6d4); \
                            font-family: monospace;",
                    "{preview}"
                }
            }
            {children}
        }
    }
}

// ── FormatBtn helper ──────────────────────────────────────────────────────────

#[component]
fn FormatBtn(label: String, active: bool, onclick: EventHandler<()>) -> Element {
    let style = if active {
        "padding: 5px 12px; font-size: 12px; font-weight: 600; border-radius: 999px; \
         border: 1.5px solid var(--fs-color-primary, #06b6d4); \
         background: rgba(6,182,212,0.15); color: var(--fs-color-primary, #06b6d4); cursor: pointer;"
    } else {
        "padding: 5px 12px; font-size: 12px; border-radius: 999px; \
         border: 1px solid var(--fs-color-border-default, #334155); \
         background: transparent; color: var(--fs-color-text-muted); cursor: pointer;"
    };
    rsx! {
        button {
            style: "{style}",
            onclick: move |_| onclick.call(()),
            "{label}"
        }
    }
}

// ── Builder tab ───────────────────────────────────────────────────────────────

#[component]
fn BuilderTab(lang_id: String) -> Element {
    use fs_manager_language::GitContributorCheck;

    let status = use_memo(move || {
        GitContributorCheck::cached().unwrap_or(
            fs_manager_language::ContributorStatus::Unknown
        )
    });

    rsx! {
        div { style: "max-width: 600px;",

            // SSH contributor badge
            div {
                style: "display: flex; align-items: center; justify-content: space-between; \
                        margin-bottom: 20px;",
                h3 {
                    style: "font-size: 17px; font-weight: 700; margin: 0; \
                            color: var(--fs-color-text-primary);",
                    {fs_i18n::t("managers.language.tab_builder")}
                }
                match status() {
                    fs_manager_language::ContributorStatus::Authenticated { github_user } => rsx! {
                        span {
                            style: "font-size: 12px; padding: 3px 10px; \
                                    background: rgba(34,197,94,0.1); \
                                    border: 1px solid rgba(34,197,94,0.3); \
                                    border-radius: 999px; color: #22c55e;",
                            "✓ GitHub: @{github_user}"
                        }
                    },
                    fs_manager_language::ContributorStatus::NotAuthenticated => rsx! {
                        span {
                            style: "font-size: 12px; padding: 3px 10px; \
                                    background: var(--fs-color-bg-overlay, #1e293b); \
                                    border: 1px solid var(--fs-color-border-default, #334155); \
                                    border-radius: 999px; color: var(--fs-color-text-muted);",
                            "✕ No SSH key"
                        }
                    },
                    _ => rsx! {
                        span {
                            style: "font-size: 12px; padding: 3px 10px; \
                                    background: var(--fs-color-bg-overlay, #1e293b); \
                                    border: 1px solid var(--fs-color-border-default, #334155); \
                                    border-radius: 999px; color: var(--fs-color-text-muted);",
                            "… Checking"
                        }
                    },
                }
            }

            // Placeholder
            div {
                style: "padding: 40px 24px; text-align: center; \
                        border: 1px solid var(--fs-color-border-default, #334155); \
                        border-radius: var(--fs-radius-md, 6px); \
                        background: var(--fs-color-bg-overlay, #1e293b);",
                span {
                    style: "display: block; font-size: 36px; margin-bottom: 12px;",
                    "🌍"
                }
                p {
                    style: "margin: 0; font-size: 13px; color: var(--fs-color-text-muted);",
                    {fs_i18n::t("managers.language.builder_placeholder")}
                }
                p {
                    style: "margin: 8px 0 0; font-size: 12px; \
                            color: var(--fs-color-text-muted); opacity: 0.6;",
                    "Language: {lang_id}"
                }
            }
        }
    }
}

// ── Subscribe view ────────────────────────────────────────────────────────────

#[component]
fn SubscribeView(
    subscribed_ids: Vec<String>,
    on_subscribed:  EventHandler<String>,
) -> Element {
    let all_languages = fs_i18n::all_languages();
    let mut search    = use_signal(String::new);

    rsx! {
        div { style: "padding: 24px 28px; max-width: 560px;",

            h3 {
                style: "margin: 0 0 4px; font-size: 17px; font-weight: 700; \
                        color: var(--fs-color-text-primary);",
                {fs_i18n::t("managers.language.subscribe_title")}
            }
            p {
                style: "margin: 0 0 16px; font-size: 13px; color: var(--fs-color-text-muted);",
                {fs_i18n::t("managers.language.subscribe_hint")}
            }

            // Search
            input {
                style: "width: 100%; box-sizing: border-box; padding: 7px 12px; \
                        font-size: 13px; margin-bottom: 12px; \
                        background: var(--fs-color-bg-overlay, #1e293b); \
                        border: 1px solid var(--fs-color-border-default, #334155); \
                        border-radius: var(--fs-radius-md, 6px); \
                        color: var(--fs-color-text-primary); outline: none;",
                placeholder: "Search…",
                value: "{search}",
                oninput: move |e| search.set(e.value()),
            }

            // Language list
            div {
                style: "border: 1px solid var(--fs-color-border-default, #334155); \
                        border-radius: var(--fs-radius-md, 6px); overflow: hidden; \
                        max-height: 480px; overflow-y: auto;",

                for meta in all_languages.iter() {
                    {
                        let q = search.read().to_lowercase();
                        let matches = q.is_empty()
                            || meta.name.to_lowercase().contains(&q)
                            || meta.native_name.to_lowercase().contains(&q)
                            || meta.code.to_lowercase().contains(&q);

                        if !matches {
                            rsx! {}
                        } else {
                            let code       = meta.code.to_string();
                            let is_already = subscribed_ids.contains(&code);
                            let lang       = fs_manager_language::Language::from_code(meta.code);
                            let flag_html  = lang.flag_svg().to_string();

                            rsx! {
                                div {
                                    key: "{meta.code}",
                                    style: "display: flex; align-items: center; gap: 12px; \
                                            padding: 10px 14px; \
                                            border-bottom: 1px solid var(--fs-color-border-default, #334155); \
                                            background: var(--fs-color-bg-base);",

                                    // Flag
                                    if !flag_html.is_empty() {
                                        span {
                                            style: "flex-shrink: 0; width: 24px; height: 14px; \
                                                    display: inline-flex; align-items: center; \
                                                    border-radius: 2px; overflow: hidden;",
                                            dangerous_inner_html: "{flag_html}",
                                        }
                                    } else {
                                        span {
                                            style: "flex-shrink: 0; width: 24px; text-align: center; \
                                                    font-size: 14px;",
                                            "🌐"
                                        }
                                    }

                                    // Names
                                    div { style: "flex: 1; min-width: 0;",
                                        div {
                                            style: "font-size: 13px; font-weight: 500; \
                                                    color: var(--fs-color-text-primary);",
                                            "{meta.native_name}"
                                        }
                                        div {
                                            style: "font-size: 11px; color: var(--fs-color-text-muted);",
                                            "{meta.name}  ·  {meta.code}"
                                        }
                                    }

                                    // RTL badge
                                    if meta.is_rtl() {
                                        span {
                                            style: "font-size: 10px; padding: 1px 6px; \
                                                    background: var(--fs-color-bg-overlay, #1e293b); \
                                                    color: var(--fs-color-text-muted); \
                                                    border-radius: 999px; flex-shrink: 0;",
                                            "RTL"
                                        }
                                    }

                                    // Subscribe / Already subscribed
                                    if is_already {
                                        span {
                                            style: "font-size: 11px; padding: 3px 10px; \
                                                    background: rgba(6,182,212,0.1); \
                                                    color: var(--fs-color-primary, #06b6d4); \
                                                    border-radius: 999px; flex-shrink: 0;",
                                            {fs_i18n::t("managers.language.already_subscribed")}
                                        }
                                    } else {
                                        button {
                                            style: "padding: 4px 12px; font-size: 12px; \
                                                    background: var(--fs-color-primary, #06b6d4); \
                                                    color: #fff; border: none; \
                                                    border-radius: var(--fs-radius-md, 6px); \
                                                    cursor: pointer; flex-shrink: 0;",
                                            onclick: move |_| on_subscribed.call(code.clone()),
                                            "＋"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Formats view (global locale settings) ────────────────────────────────────

#[component]
fn FormatsView() -> Element {
    // Reuse ConfigTab — formats view IS the config tab, but without a specific language context.
    rsx! {
        div { style: "padding: 24px 28px;",
            ConfigTab {}
        }
    }
}
