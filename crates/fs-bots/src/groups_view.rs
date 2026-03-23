/// Groups view — manage room collections and filter/bulk-act on rooms.
use dioxus::prelude::*;

use crate::context::GroupsContext;
use crate::model::{CachedRoom, GroupsConfig};

// ── Demo data ──────────────────────────────────────────────────────────────────

fn demo_rooms() -> Vec<CachedRoom> {
    vec![
        CachedRoom { platform: "Telegram".into(), room_id: "-1001".into(), room_name: "FreeSynergy Community".into(), member_count: Some(1240) },
        CachedRoom { platform: "Telegram".into(), room_id: "-1002".into(), room_name: "Dev Chat".into(),              member_count: Some(87)   },
        CachedRoom { platform: "Matrix".into(),   room_id: "!abc:fsn".into(), room_name: "#general:freesynergy.net".into(), member_count: Some(43) },
        CachedRoom { platform: "Matrix".into(),   room_id: "!def:fsn".into(), room_name: "#dev:freesynergy.net".into(),     member_count: Some(12) },
        CachedRoom { platform: "Discord".into(),  room_id: "111".into(),      room_name: "announcements".into(),           member_count: Some(520) },
    ]
}

// ── RoomPredicate + RoomFilter ────────────────────────────────────────────────

/// Predicate trait for filtering rooms — enables testable, composable filter logic.
trait RoomPredicate {
    fn matches(&self, room: &CachedRoom) -> bool;
}

#[derive(Clone, Default)]
struct RoomFilter {
    platform:    String,
    name:        String,
    min_members: String,
    max_members: String,
}

impl RoomPredicate for RoomFilter {
    fn matches(&self, room: &CachedRoom) -> bool {
        if !self.platform.is_empty() && room.platform != self.platform { return false; }
        if !self.name.is_empty() && !room.room_name.to_lowercase().contains(&self.name.to_lowercase()) { return false; }
        if let (Ok(min), Some(cnt)) = (self.min_members.parse::<u32>(), room.member_count) {
            if cnt < min { return false; }
        }
        if let (Ok(max), Some(cnt)) = (self.max_members.parse::<u32>(), room.member_count) {
            if cnt > max { return false; }
        }
        true
    }
}

// ── GroupsView ─────────────────────────────────────────────────────────────────

/// Groups & Collections view inside BotManager.
#[component]
pub fn GroupsView() -> Element {
    let cfg           = GroupsConfig::load();
    let initial_rooms = if cfg.cached_rooms.is_empty() { demo_rooms() } else { cfg.cached_rooms };

    let collections    = use_signal(|| cfg.collections);
    let rooms          = use_signal(|| initial_rooms);
    let sel_collection = use_signal(|| None::<u32>);
    let ctx            = GroupsContext { collections, rooms, sel_collection };
    provide_context(ctx.clone());

    let mut filter         = use_signal(RoomFilter::default);
    let mut new_col_name   = use_signal(String::new);
    let mut new_col_desc   = use_signal(String::new);
    let mut show_new_col   = use_signal(|| false);
    let mut selected_rooms: Signal<Vec<(String, String)>> = use_signal(Vec::new);

    let filtered_rooms: Vec<CachedRoom> = ctx.rooms.read().iter()
        .filter(|r| filter.read().matches(r))
        .cloned()
        .collect();

    let mut platforms: Vec<String> = ctx.rooms.read().iter().map(|r| r.platform.clone()).collect();
    platforms.sort(); platforms.dedup();

    rsx! {
        div { style: "display: flex; gap: 20px; height: 100%; overflow: hidden;",

            // ── Left: Collections ─────────────────────────────────────────────
            div {
                style: "width: 240px; flex-shrink: 0; display: flex; flex-direction: column; gap: 10px; \
                        border-right: 1px solid var(--fs-border); padding-right: 16px; overflow-y: auto;",

                div {
                    style: "display: flex; align-items: center; justify-content: space-between;",
                    span {
                        style: "font-size: 12px; font-weight: 600; text-transform: uppercase; \
                                letter-spacing: 0.06em; color: var(--fs-color-text-muted);",
                        "Collections"
                    }
                    button {
                        style: "background: var(--fs-color-primary); color: #fff; border: none; \
                                border-radius: var(--fs-radius-sm); padding: 3px 8px; font-size: 11px; cursor: pointer;",
                        onclick: move |_| { let v = *show_new_col.read(); show_new_col.set(!v); },
                        "+"
                    }
                }

                if *show_new_col.read() {
                    div {
                        style: "display: flex; flex-direction: column; gap: 6px; padding: 8px; \
                                background: var(--fs-color-bg-overlay); border-radius: var(--fs-radius-md);",
                        input {
                            style: "background: var(--fs-color-bg-base); border: 1px solid var(--fs-color-border-default); \
                                    border-radius: var(--fs-radius-sm); padding: 5px 8px; font-size: 12px; \
                                    color: var(--fs-color-text-primary);",
                            placeholder: "Collection name",
                            oninput: move |e| new_col_name.set(e.value()),
                        }
                        input {
                            style: "background: var(--fs-color-bg-base); border: 1px solid var(--fs-color-border-default); \
                                    border-radius: var(--fs-radius-sm); padding: 5px 8px; font-size: 12px; \
                                    color: var(--fs-color-text-primary);",
                            placeholder: "Description (optional)",
                            oninput: move |e| new_col_desc.set(e.value()),
                        }
                        button {
                            style: "background: var(--fs-color-primary); color: #fff; border: none; \
                                    border-radius: var(--fs-radius-sm); padding: 5px 10px; font-size: 12px; cursor: pointer;",
                            onclick: {
                                let ctx = ctx.clone();
                                move |_| {
                                    ctx.add_collection(
                                        new_col_name.read().clone(),
                                        new_col_desc.read().clone(),
                                    );
                                    new_col_name.set(String::new());
                                    new_col_desc.set(String::new());
                                    show_new_col.set(false);
                                }
                            },
                            "Create"
                        }
                    }
                }

                // All Rooms entry
                {
                    let active      = ctx.sel_collection.read().is_none();
                    let mut sel_col = ctx.sel_collection; // Signal: Copy
                    rsx! {
                        div {
                            style: if active {
                                "padding: 7px 10px; border-radius: var(--fs-radius-md); cursor: pointer; \
                                 background: var(--fs-color-primary); color: #fff; font-size: 13px;"
                            } else {
                                "padding: 7px 10px; border-radius: var(--fs-radius-md); cursor: pointer; \
                                 color: var(--fs-color-text-primary); font-size: 13px;"
                            },
                            onclick: move |_| sel_col.set(None),
                            "🏠 All Rooms ({ctx.rooms.read().len()})"
                        }
                    }
                }

                for col in ctx.collections.read().clone().iter() {
                    {
                        let col    = col.clone();
                        let active = *ctx.sel_collection.read() == Some(col.id);
                        let col_id = col.id;
                        let mut sel_col = ctx.sel_collection; // Signal: Copy
                        rsx! {
                            div {
                                key: "{col.id}",
                                style: if active {
                                    "padding: 7px 10px; border-radius: var(--fs-radius-md); cursor: pointer; \
                                     background: var(--fs-color-primary); color: #fff; font-size: 13px;"
                                } else {
                                    "padding: 7px 10px; border-radius: var(--fs-radius-md); cursor: pointer; \
                                     color: var(--fs-color-text-primary); font-size: 13px;"
                                },
                                onclick: move |_| sel_col.set(Some(col_id)),
                                "📁 {col.name} ({col.members.len()})"
                            }
                        }
                    }
                }
            }

            // ── Right: Room list + filter ──────────────────────────────────────
            div { style: "flex: 1; display: flex; flex-direction: column; gap: 12px; overflow: hidden;",

                // Filter bar
                div { style: "display: flex; gap: 8px; flex-wrap: wrap;",
                    select {
                        style: "padding: 5px 8px; font-size: 12px; border-radius: var(--fs-radius-sm); \
                                border: 1px solid var(--fs-color-border-default); \
                                background: var(--fs-color-bg-overlay); color: var(--fs-color-text-primary);",
                        onchange: move |e| filter.write().platform = if e.value() == "all" { String::new() } else { e.value() },
                        option { value: "all", "All platforms" }
                        for p in &platforms { option { value: "{p}", "{p}" } }
                    }
                    input {
                        style: "padding: 5px 8px; font-size: 12px; border-radius: var(--fs-radius-sm); \
                                border: 1px solid var(--fs-color-border-default); flex: 1; \
                                background: var(--fs-color-bg-overlay); color: var(--fs-color-text-primary);",
                        placeholder: "Filter by name…",
                        oninput: move |e| filter.write().name = e.value(),
                    }
                    input {
                        style: "padding: 5px 8px; font-size: 12px; border-radius: var(--fs-radius-sm); \
                                border: 1px solid var(--fs-color-border-default); width: 80px; \
                                background: var(--fs-color-bg-overlay); color: var(--fs-color-text-primary);",
                        placeholder: "Min members",
                        r#type: "number",
                        oninput: move |e| filter.write().min_members = e.value(),
                    }
                    input {
                        style: "padding: 5px 8px; font-size: 12px; border-radius: var(--fs-radius-sm); \
                                border: 1px solid var(--fs-color-border-default); width: 80px; \
                                background: var(--fs-color-bg-overlay); color: var(--fs-color-text-primary);",
                        placeholder: "Max members",
                        r#type: "number",
                        oninput: move |e| filter.write().max_members = e.value(),
                    }
                }

                // Bulk action bar
                if !selected_rooms.read().is_empty() {
                    div {
                        style: "display: flex; align-items: center; gap: 10px; padding: 8px 12px; \
                                background: var(--fs-color-bg-overlay); border-radius: var(--fs-radius-md); font-size: 13px;",
                        span { style: "color: var(--fs-color-text-muted);",
                            "{selected_rooms.read().len()} selected"
                        }
                        if let Some(col_id) = *ctx.sel_collection.read() {
                            button {
                                style: "background: #ef4444; color: #fff; border: none; border-radius: var(--fs-radius-sm); \
                                        padding: 4px 12px; font-size: 12px; cursor: pointer;",
                                onclick: {
                                    let ctx = ctx.clone();
                                    move |_| {
                                        ctx.remove_rooms_from_collection(col_id, selected_rooms.read().clone());
                                        selected_rooms.set(vec![]);
                                    }
                                },
                                "Remove from collection"
                            }
                        }
                        for col in ctx.collections.read().clone().iter() {
                            {
                                let col    = col.clone();
                                let col_id = col.id;
                                let ctx2 = ctx.clone();
                                rsx! {
                                    button {
                                        key: "{col.id}",
                                        style: "background: var(--fs-color-primary); color: #fff; border: none; \
                                                border-radius: var(--fs-radius-sm); padding: 4px 12px; font-size: 12px; cursor: pointer;",
                                        onclick: move |_| {
                                            ctx2.add_rooms_to_collection(col_id, selected_rooms.read().clone());
                                            selected_rooms.set(vec![]);
                                        },
                                        "Add to: {col.name}"
                                    }
                                }
                            }
                        }
                        button {
                            style: "background: transparent; color: var(--fs-color-text-muted); border: none; \
                                    font-size: 12px; cursor: pointer;",
                            onclick: move |_| selected_rooms.set(vec![]),
                            "✕ Clear"
                        }
                    }
                }

                // Room list
                div { style: "flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 4px;",
                    {
                        let display_rooms: Vec<CachedRoom> = match *ctx.sel_collection.read() {
                            None => filtered_rooms.clone(),
                            Some(col_id) => {
                                let col_members: Vec<(String, String)> = ctx.collections.read().iter()
                                    .find(|c| c.id == col_id)
                                    .map(|c| c.members.clone())
                                    .unwrap_or_default();
                                ctx.rooms.read().iter()
                                    .filter(|r| col_members.contains(&(r.platform.clone(), r.room_id.clone())))
                                    .filter(|r| filter.read().matches(r))
                                    .cloned()
                                    .collect()
                            }
                        };

                        rsx! {
                            for room in display_rooms {
                                {
                                    let key     = (room.platform.clone(), room.room_id.clone());
                                    let checked = selected_rooms.read().contains(&key);
                                    rsx! {
                                        div {
                                            key: "{room.platform}:{room.room_id}",
                                            style: "display: flex; align-items: center; gap: 10px; \
                                                    padding: 8px 12px; border-radius: var(--fs-radius-md); \
                                                    background: var(--fs-color-bg-overlay); font-size: 13px; \
                                                    cursor: pointer;",
                                            onclick: move |_| {
                                                if checked {
                                                    selected_rooms.write().retain(|k| k != &key);
                                                } else {
                                                    selected_rooms.write().push(key.clone());
                                                }
                                            },
                                            span {
                                                style: "color: var(--fs-color-primary); font-size: 15px;",
                                                if checked { "☑" } else { "☐" }
                                            }
                                            span {
                                                style: "color: var(--fs-color-text-muted); font-size: 11px; \
                                                        background: var(--fs-color-bg-base); padding: 1px 5px; \
                                                        border-radius: 3px;",
                                                "{room.platform}"
                                            }
                                            span {
                                                style: "flex: 1; color: var(--fs-color-text-primary); font-weight: 500;",
                                                "{room.room_name}"
                                            }
                                            if let Some(cnt) = room.member_count {
                                                span {
                                                    style: "color: var(--fs-color-text-muted); font-size: 11px;",
                                                    "👥 {cnt}"
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
    }
}
