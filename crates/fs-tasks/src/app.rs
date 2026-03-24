/// Tasks — automation pipeline manager.
use dioxus::prelude::*;

use crate::model::{DataSource, DataTarget, DataTrigger, TaskPipeline, TasksConfig};
use crate::pipeline_editor::PipelineEditor;
use crate::templates::BUILTIN_TEMPLATES;

#[component]
pub fn TasksApp() -> Element {
    let mut tasks = use_signal(TasksConfig::load);
    let mut selected_idx: Signal<Option<usize>> = use_signal(|| None);
    let mut editing = use_signal(|| false);
    let mut show_templates = use_signal(|| false);
    let mut next_id = use_signal(|| 100u32);

    let task_list = tasks.read().tasks.clone();
    let sel_idx = *selected_idx.read();
    let selected = sel_idx.and_then(|i| task_list.get(i).cloned());
    let is_editing = *editing.read();
    let is_show_templates = *show_templates.read();

    rsx! {
        div {
            style: "display: flex; height: 100%; width: 100%; overflow: hidden; \
                    background: var(--fs-color-bg-base);",

            // ── Left panel ────────────────────────────────────────────────
            div {
                style: "width: 260px; flex-shrink: 0; display: flex; flex-direction: column; \
                        background: var(--fs-color-bg-surface); \
                        border-right: 1px solid var(--fs-color-border-default);",

                div {
                    style: "padding: 12px; border-bottom: 1px solid var(--fs-color-border-default); \
                            display: flex; flex-direction: column; gap: 6px;",
                    div {
                        style: "font-size: 11px; font-weight: 600; text-transform: uppercase; \
                                letter-spacing: 0.08em; color: var(--fs-color-text-muted);",
                        "Tasks"
                    }
                    div { style: "display: flex; gap: 6px;",
                        button {
                            onclick: move |_| {
                                let id = *next_id.read();
                                next_id.set(id + 1);
                                let task = TaskPipeline::new_default(id);
                                let new_idx = tasks.read().tasks.len();
                                tasks.write().tasks.push(task);
                                selected_idx.set(Some(new_idx));
                                editing.set(true);
                                show_templates.set(false);
                                let _ = tasks.read().save();
                            },
                            style: "flex: 1; background: var(--fs-color-primary); color: #fff; \
                                    border: none; border-radius: var(--fs-radius-sm); \
                                    padding: 6px 10px; font-size: 12px; cursor: pointer;",
                            "+ New Task"
                        }
                        button {
                            onclick: move |_| { let v = *show_templates.read(); show_templates.set(!v); },
                            style: "background: var(--fs-color-bg-overlay); \
                                    border: 1px solid var(--fs-color-border-default); \
                                    border-radius: var(--fs-radius-sm); \
                                    padding: 6px 10px; font-size: 12px; \
                                    color: var(--fs-color-text-muted); cursor: pointer;",
                            "📦 Templates"
                        }
                    }
                }

                div { style: "flex: 1; overflow-y: auto;",
                    if task_list.is_empty() {
                        div {
                            style: "padding: 20px; text-align: center; \
                                    color: var(--fs-color-text-muted); font-size: 13px;",
                            "No tasks yet."
                        }
                    }
                    for (idx, task) in task_list.iter().enumerate() {
                        TaskListRow {
                            key: "{task.id}",
                            task: task.clone(),
                            is_active: sel_idx == Some(idx),
                            on_click: move |_| {
                                selected_idx.set(Some(idx));
                                editing.set(false);
                                show_templates.set(false);
                            },
                        }
                    }
                }
            }

            // ── Right panel ───────────────────────────────────────────────
            div {
                style: "flex: 1; overflow: auto; padding: 20px;",

                if is_show_templates {
                    TemplateBrowser {
                        on_use_template: move |tmpl_id: &'static str| {
                            if let Some(t) = BUILTIN_TEMPLATES.iter().find(|t| t.id == tmpl_id) {
                                let id = *next_id.read();
                                next_id.set(id + 1);
                                let task = TaskPipeline {
                                    id: format!("task-{id}"),
                                    name: t.name.to_string(),
                                    source: DataSource {
                                        service: t.source_service.to_string(),
                                        offer: t.source_offer.to_string(),
                                        fields: vec![],
                                    },
                                    target: DataTarget {
                                        service: t.target_service.to_string(),
                                        accept: t.target_accept.to_string(),
                                        fields: vec![],
                                    },
                                    mappings: vec![],
                                    trigger: DataTrigger::Manual,
                                    enabled: true,
                                    last_run: None,
                                };
                                let new_idx = tasks.read().tasks.len();
                                tasks.write().tasks.push(task);
                                selected_idx.set(Some(new_idx));
                                editing.set(true);
                                show_templates.set(false);
                                let _ = tasks.read().save();
                            }
                        }
                    }
                } else if let Some(task) = selected {
                    if is_editing {
                        PipelineEditor {
                            pipeline: task.clone(),
                            on_save: move |updated: TaskPipeline| {
                                if let Some(i) = sel_idx {
                                    tasks.write().tasks[i] = updated;
                                    let _ = tasks.read().save();
                                }
                                editing.set(false);
                            },
                            on_cancel: move |_| editing.set(false),
                        }
                    } else {
                        TaskDetail {
                            task,
                            on_edit: move |_| editing.set(true),
                            on_delete: move |_| {
                                if let Some(i) = sel_idx {
                                    tasks.write().tasks.remove(i);
                                    let _ = tasks.read().save();
                                    selected_idx.set(None);
                                }
                            },
                            on_run: move |_| {},
                        }
                    }
                } else {
                    div {
                        style: "display: flex; flex-direction: column; align-items: center; \
                                justify-content: center; height: 200px; gap: 12px; \
                                color: var(--fs-color-text-muted);",
                        span { style: "font-size: 32px;", "📋" }
                        span { style: "font-size: 14px;", "Select a task or create a new one" }
                    }
                }
            }
        }
    }
}

// ── TaskListRow ───────────────────────────────────────────────────────────────

#[component]
fn TaskListRow(task: TaskPipeline, is_active: bool, on_click: EventHandler<MouseEvent>) -> Element {
    let bg = if is_active {
        "var(--fs-color-bg-overlay)"
    } else {
        "transparent"
    };
    let border = if is_active {
        "2px solid var(--fs-color-primary)"
    } else {
        "2px solid transparent"
    };
    let dot = if task.enabled { "#22c55e" } else { "#64748b" };

    rsx! {
        div {
            onclick: on_click,
            style: "padding: 10px 12px; cursor: pointer; \
                    border-left: {border}; background: {bg}; \
                    display: flex; align-items: center; gap: 10px;",
            div { style: "flex: 1; min-width: 0;",
                div {
                    style: "font-size: 13px; font-weight: 500; color: var(--fs-color-text-primary); \
                            overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{task.name}"
                }
                div {
                    style: "font-size: 11px; color: var(--fs-color-text-muted); margin-top: 2px; \
                            overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{task.source.service} → {task.target.service}"
                }
            }
            span { style: "font-size: 10px; color: {dot}; flex-shrink: 0;", "●" }
        }
    }
}

// ── TaskDetail ────────────────────────────────────────────────────────────────

#[component]
fn TaskDetail(
    task: TaskPipeline,
    on_edit: EventHandler<()>,
    on_delete: EventHandler<()>,
    on_run: EventHandler<()>,
) -> Element {
    let status_color = if task.enabled { "#22c55e" } else { "#64748b" };
    let trigger_label = task.trigger.label();

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 20px;",

            div {
                h2 { style: "margin: 0 0 4px; font-size: 18px; color: var(--fs-color-text-primary);",
                    "{task.name}"
                }
                span { style: "font-size: 12px; color: {status_color};", "{task.status_label()}" }
            }

            div {
                style: "display: flex; align-items: center; gap: 16px; \
                        background: var(--fs-color-bg-overlay); \
                        border-radius: var(--fs-radius-md); padding: 16px;",
                div {
                    div { style: "font-size: 10px; text-transform: uppercase; color: var(--fs-color-text-muted);", "Source" }
                    div { style: "font-size: 14px; font-weight: 600; color: var(--fs-color-text-primary);", "{task.source.service}" }
                    div { style: "font-size: 12px; color: var(--fs-color-primary);", "{task.source.offer}" }
                }
                div { style: "font-size: 20px; color: var(--fs-color-primary); flex: 1; text-align: center;", "──▶" }
                div {
                    div { style: "font-size: 10px; text-transform: uppercase; color: var(--fs-color-text-muted);", "Target" }
                    div { style: "font-size: 14px; font-weight: 600; color: var(--fs-color-text-primary);", "{task.target.service}" }
                    div { style: "font-size: 12px; color: var(--fs-color-primary);", "{task.target.accept}" }
                }
            }

            if !task.mappings.is_empty() {
                div {
                    div { style: "font-size: 12px; font-weight: 600; text-transform: uppercase; \
                                  letter-spacing: 0.06em; color: var(--fs-color-text-muted); margin-bottom: 8px;",
                        "Field Mapping"
                    }
                    div { style: "display: flex; flex-direction: column; gap: 4px;",
                        for (i, m) in task.mappings.iter().enumerate() {
                            div {
                                key: "{i}",
                                style: "display: flex; align-items: center; gap: 8px; \
                                        background: var(--fs-color-bg-overlay); \
                                        border-radius: var(--fs-radius-sm); \
                                        padding: 6px 10px; font-size: 12px;",
                                span { style: "color: var(--fs-color-text-muted); min-width: 120px;",
                                    "{m.source_field.as_deref().unwrap_or(\"(fixed)\")}"
                                }
                                span { style: "color: var(--fs-color-primary);", "→" }
                                span { style: "color: var(--fs-color-text-primary);", "{m.target_field}" }
                                span { style: "color: var(--fs-color-text-muted); font-size: 11px; margin-left: auto;",
                                    "{m.transform.label()}"
                                }
                            }
                        }
                    }
                }
            }

            div {
                div { style: "font-size: 12px; font-weight: 600; text-transform: uppercase; \
                              letter-spacing: 0.06em; color: var(--fs-color-text-muted); margin-bottom: 4px;",
                    "Trigger"
                }
                span { style: "font-size: 13px; color: var(--fs-color-text-primary);", "{trigger_label}" }
            }

            div { style: "display: flex; gap: 8px;",
                button {
                    onclick: move |_| on_run.call(()),
                    style: "background: var(--fs-color-primary); color: #fff; \
                            border: none; border-radius: var(--fs-radius-md); \
                            padding: 8px 16px; font-size: 13px; font-weight: 600; cursor: pointer;",
                    "▶ Run Now"
                }
                button {
                    onclick: move |_| on_edit.call(()),
                    style: "background: var(--fs-color-bg-overlay); \
                            border: 1px solid var(--fs-color-border-default); \
                            border-radius: var(--fs-radius-md); \
                            padding: 8px 16px; font-size: 13px; \
                            color: var(--fs-color-text-primary); cursor: pointer;",
                    "✏ Edit"
                }
                button {
                    onclick: move |_| on_delete.call(()),
                    style: "background: transparent; \
                            border: 1px solid #ef4444; \
                            border-radius: var(--fs-radius-md); \
                            padding: 8px 16px; font-size: 13px; \
                            color: #ef4444; cursor: pointer;",
                    "🗑 Delete"
                }
            }
        }
    }
}

// ── TemplateBrowser ───────────────────────────────────────────────────────────

#[component]
fn TemplateBrowser(on_use_template: EventHandler<&'static str>) -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 12px;",
            h3 { style: "margin: 0; font-size: 16px; color: var(--fs-color-text-primary);",
                "Task Templates"
            }
            p { style: "margin: 0; font-size: 13px; color: var(--fs-color-text-muted);",
                "Pre-configured automation pipelines."
            }
            div { style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 12px;",
                for tmpl in BUILTIN_TEMPLATES {
                    TemplateCard { key: "{tmpl.id}", template: tmpl, on_use: on_use_template }
                }
            }
        }
    }
}

#[component]
fn TemplateCard(
    template: &'static crate::model::TaskTemplate,
    on_use: EventHandler<&'static str>,
) -> Element {
    let id = template.id;
    rsx! {
        div {
            style: "background: var(--fs-color-bg-overlay); \
                    border: 1px solid var(--fs-color-border-default); \
                    border-radius: var(--fs-radius-md); padding: 16px; \
                    display: flex; flex-direction: column; gap: 8px;",
            div { style: "display: flex; align-items: center; gap: 10px;",
                span { style: "font-size: 24px;", "{template.icon}" }
                span { style: "font-size: 14px; font-weight: 600; color: var(--fs-color-text-primary);",
                    "{template.name}"
                }
            }
            p { style: "margin: 0; font-size: 12px; color: var(--fs-color-text-muted);",
                "{template.description}"
            }
            div { style: "font-size: 11px; color: var(--fs-color-text-muted);",
                "{template.source_service} → {template.target_service}"
            }
            button {
                onclick: move |_| on_use.call(id),
                style: "background: var(--fs-color-primary); color: #fff; \
                        border: none; border-radius: var(--fs-radius-sm); \
                        padding: 6px 14px; font-size: 12px; font-weight: 600; \
                        cursor: pointer; align-self: flex-start;",
                "Use Template"
            }
        }
    }
}
