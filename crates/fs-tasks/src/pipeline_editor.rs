use crate::model::{DataTrigger, FieldMapping, TaskPipeline};
use dioxus::prelude::*;

#[component]
pub fn PipelineEditor(
    pipeline: TaskPipeline,
    on_save: EventHandler<TaskPipeline>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut name = use_signal(|| pipeline.name.clone());
    let mut trigger = use_signal(|| pipeline.trigger.clone());
    let mappings = pipeline.mappings.clone();
    let source = pipeline.source.clone();
    let target = pipeline.target.clone();

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 16px;",

            div {
                label {
                    style: "display: block; font-size: 12px; font-weight: 600; \
                            text-transform: uppercase; letter-spacing: 0.06em; \
                            color: var(--fs-color-text-muted); margin-bottom: 6px;",
                    "Task Name"
                }
                input {
                    r#type: "text",
                    value: "{name}",
                    oninput: move |e| name.set(e.value()),
                    style: "width: 100%; background: var(--fs-color-bg-overlay); \
                            border: 1px solid var(--fs-color-border-default); \
                            border-radius: var(--fs-radius-md); \
                            padding: 8px 12px; font-size: 13px; font-family: inherit; \
                            color: var(--fs-color-text-primary); box-sizing: border-box;",
                }
            }

            div { style: "display: flex; align-items: center; gap: 16px;",
                ServiceBox { label: "Source", service: source.service.clone(), offer: source.offer.clone() }
                span { style: "font-size: 20px; color: var(--fs-color-primary);", "──▶" }
                ServiceBox { label: "Target", service: target.service.clone(), offer: target.accept.clone() }
            }

            if !mappings.is_empty() {
                div {
                    div {
                        style: "font-size: 12px; font-weight: 600; text-transform: uppercase; \
                                letter-spacing: 0.06em; color: var(--fs-color-text-muted); margin-bottom: 8px;",
                        "Field Mapping"
                    }
                    div { style: "display: flex; flex-direction: column; gap: 4px;",
                        for m in &mappings {
                            MappingRow { key: "{m.target_field}", mapping: m.clone() }
                        }
                    }
                }
            }

            div {
                div {
                    style: "font-size: 12px; font-weight: 600; text-transform: uppercase; \
                            letter-spacing: 0.06em; color: var(--fs-color-text-muted); margin-bottom: 8px;",
                    "Trigger"
                }
                TriggerSelector {
                    trigger: trigger.read().clone(),
                    on_change: move |t| trigger.set(t),
                }
            }

            div { style: "display: flex; gap: 8px;",
                button {
                    onclick: {
                        let n = name.read().clone();
                        let t = trigger.read().clone();
                        let mut updated = pipeline.clone();
                        move |_| {
                            updated.name    = n.clone();
                            updated.trigger = t.clone();
                            on_save.call(updated.clone());
                        }
                    },
                    style: "background: var(--fs-color-primary); color: #fff; \
                            border: none; border-radius: var(--fs-radius-md); \
                            padding: 8px 20px; font-size: 13px; font-weight: 600; cursor: pointer;",
                    "💾 Save"
                }
                button {
                    onclick: move |_| on_cancel.call(()),
                    style: "background: transparent; \
                            border: 1px solid var(--fs-color-border-default); \
                            border-radius: var(--fs-radius-md); \
                            padding: 8px 16px; font-size: 13px; \
                            color: var(--fs-color-text-muted); cursor: pointer;",
                    "Cancel"
                }
            }
        }
    }
}

#[component]
fn ServiceBox(label: &'static str, service: String, offer: String) -> Element {
    rsx! {
        div {
            style: "flex: 1; background: var(--fs-color-bg-overlay); \
                    border: 1px solid var(--fs-color-border-default); \
                    border-radius: var(--fs-radius-md); padding: 12px;",
            div {
                style: "font-size: 10px; font-weight: 600; text-transform: uppercase; \
                        letter-spacing: 0.06em; color: var(--fs-color-text-muted); margin-bottom: 4px;",
                "{label}"
            }
            div { style: "font-size: 13px; font-weight: 600; color: var(--fs-color-text-primary);", "{service}" }
            div { style: "font-size: 12px; color: var(--fs-color-primary); margin-top: 2px;", "{offer}" }
        }
    }
}

#[component]
fn MappingRow(mapping: FieldMapping) -> Element {
    let source = mapping.source_field.as_deref().unwrap_or("(fixed)");
    rsx! {
        div {
            style: "display: grid; grid-template-columns: 1fr auto 1fr; gap: 8px; \
                    align-items: center; background: var(--fs-color-bg-overlay); \
                    border-radius: var(--fs-radius-sm); padding: 6px 10px; font-size: 12px;",
            span { style: "color: var(--fs-color-text-primary);", "{source}" }
            span { style: "color: var(--fs-color-primary); font-size: 14px;", "→" }
            span { style: "color: var(--fs-color-text-primary);", "{mapping.target_field}" }
        }
    }
}

#[component]
fn TriggerSelector(trigger: DataTrigger, on_change: EventHandler<DataTrigger>) -> Element {
    let is_manual = matches!(trigger, DataTrigger::Manual);
    let is_on_event = matches!(trigger, DataTrigger::OnEvent(_));
    let is_scheduled = matches!(trigger, DataTrigger::Scheduled(_));

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 6px;",
            TriggerOption { label: "Manual only",     selected: is_manual,    on_select: move |_| on_change.call(DataTrigger::Manual) }
            TriggerOption { label: "On event",        selected: is_on_event,  on_select: move |_| on_change.call(DataTrigger::OnEvent("commit-pushed".into())) }
            TriggerOption { label: "Scheduled (cron)", selected: is_scheduled, on_select: move |_| on_change.call(DataTrigger::Scheduled("0 8 * * *".into())) }
        }
    }
}

#[component]
fn TriggerOption(label: &'static str, selected: bool, on_select: EventHandler<()>) -> Element {
    let radio = if selected { "●" } else { "○" };
    let color = if selected {
        "var(--fs-color-primary)"
    } else {
        "var(--fs-color-text-muted)"
    };
    rsx! {
        div {
            onclick: move |_| on_select.call(()),
            style: "display: flex; align-items: center; gap: 8px; cursor: pointer; \
                    font-size: 13px; color: {color}; padding: 4px 0;",
            span { "{radio}" }
            span { "{label}" }
        }
    }
}
