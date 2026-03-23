/// Log viewer — shows recent journal logs for a systemd service.
///
/// Calls `journalctl --user -u <unit> -n 200 --no-pager` (no Podman socket).
use dioxus::prelude::*;
use fs_i18n;

/// Log entry with severity level.
#[derive(Clone, Debug, PartialEq)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
}

/// Log severity level — detected from common log prefixes.
#[derive(Clone, Debug, PartialEq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

impl LogLevel {
    pub fn color(&self) -> &str {
        match self {
            Self::Info  => "var(--fs-text-primary)",
            Self::Warn  => "var(--fs-warning)",
            Self::Error => "var(--fs-error)",
            Self::Debug => "var(--fs-text-muted)",
        }
    }

    fn detect(line: &str) -> Self {
        let l = line.to_ascii_lowercase();
        if l.contains("error") || l.contains("err]") || l.contains("fatal") || l.contains("panic") {
            Self::Error
        } else if l.contains("warn") || l.contains("wrn]") {
            Self::Warn
        } else if l.contains("debug") || l.contains("dbg]") || l.contains("trace") {
            Self::Debug
        } else {
            Self::Info
        }
    }
}

/// Fetch last N lines from journalctl --user for the given unit.
async fn fetch_journal(unit: &str, lines: u32) -> Vec<String> {
    let Ok(out) = tokio::process::Command::new("journalctl")
        .args(["--user", "-u", unit, "-n", &lines.to_string(), "--no-pager", "--output=short"])
        .output()
        .await
    else {
        return vec![];
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.to_string())
        .collect()
}

/// Live log viewer component — polls the last 200 lines every 3 seconds.
#[component]
pub fn LogViewer(service: String) -> Element {
    let mut entries = use_signal(Vec::<LogEntry>::new);
    let mut follow  = use_signal(|| true);

    use_future({
        let service = service.clone();
        move || {
            let service = service.clone();
            async move {
                if service.is_empty() {
                    return;
                }
                loop {
                    let lines = fetch_journal(&service, 200).await;
                    let new_entries: Vec<LogEntry> = lines
                        .into_iter()
                        .filter(|l| !l.is_empty())
                        .map(|line| LogEntry {
                            level: LogLevel::detect(&line),
                            message: line,
                        })
                        .collect();
                    if !new_entries.is_empty() {
                        entries.set(new_entries);
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                }
            }
        }
    });

    rsx! {
        div {
            class: "fs-log-viewer",
            style: "display: flex; flex-direction: column; height: 100%;",

            // Toolbar
            div {
                style: "display: flex; align-items: center; gap: 12px; padding: 8px 0; border-bottom: 1px solid var(--fs-border);",

                h3 { style: "margin: 0; flex: 1;", {fs_i18n::t_with("container.logs.title", &[("service", &service)])} }

                label { style: "display: flex; align-items: center; gap: 4px; font-size: 13px;",
                    input {
                        r#type: "checkbox",
                        checked: *follow.read(),
                        oninput: move |e| follow.set(e.checked()),
                    }
                    {fs_i18n::t("container.logs.follow")}
                }

                button {
                    style: "padding: 4px 8px; background: var(--fs-bg-surface); border: 1px solid var(--fs-border); border-radius: 4px; cursor: pointer; font-size: 12px; color: var(--fs-text-primary);",
                    onclick: move |_| entries.set(vec![]),
                    {fs_i18n::t("actions.clear")}
                }
            }

            // Log output
            div {
                style: "flex: 1; overflow-y: auto; padding: 8px 0; font-family: var(--fs-font-mono); font-size: 12px;",

                if entries.read().is_empty() {
                    div {
                        style: "color: var(--fs-text-muted); padding: 16px;",
                        if service.is_empty() {
                            {fs_i18n::t("container.logs.empty_no_service")}
                        } else {
                            {fs_i18n::t_with("container.logs.empty", &[("service", &service)])}
                        }
                    }
                }

                for entry in entries.read().iter() {
                    div {
                        style: "padding: 2px 0; color: {entry.level.color()}; word-break: break-all;",
                        "{entry.message}"
                    }
                }
            }
        }
    }
}
