use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataTrigger {
    Manual,
    OnEvent(String),
    Scheduled(String),
}

impl DataTrigger {
    pub fn label(&self) -> String {
        match self {
            Self::Manual => "Manual only".into(),
            Self::OnEvent(ev) => format!("On event: {ev}"),
            Self::Scheduled(cron) => format!("Scheduled: {cron}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataField {
    pub name: String,
    pub label: String,
    pub example: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldTransform {
    Direct,
    Template(String),
    Fixed(String),
}

impl FieldTransform {
    pub fn label(&self) -> String {
        match self {
            Self::Direct => "Direct copy".into(),
            Self::Template(tmpl) => format!("Template: {tmpl}"),
            Self::Fixed(val) => format!("Fixed: {val}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldMapping {
    pub source_field: Option<String>,
    pub target_field: String,
    pub transform: FieldTransform,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataSource {
    pub service: String,
    pub offer: String,
    pub fields: Vec<DataField>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataTarget {
    pub service: String,
    pub accept: String,
    pub fields: Vec<DataField>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskPipeline {
    pub id: String,
    pub name: String,
    pub source: DataSource,
    pub target: DataTarget,
    pub mappings: Vec<FieldMapping>,
    pub trigger: DataTrigger,
    pub enabled: bool,
    #[serde(default)]
    pub last_run: Option<DateTime<Utc>>,
}

impl TaskPipeline {
    pub fn new_default(id: u32) -> Self {
        Self {
            id: format!("task-{id}"),
            name: "New Task".into(),
            source: DataSource {
                service: "Forgejo".into(),
                offer: "repos.list".into(),
                fields: vec![
                    DataField {
                        name: "name".into(),
                        label: "Repo Name".into(),
                        example: "my-repo".into(),
                    },
                    DataField {
                        name: "description".into(),
                        label: "Description".into(),
                        example: "A cool project".into(),
                    },
                ],
            },
            target: DataTarget {
                service: "Outline".into(),
                accept: "document.create".into(),
                fields: vec![
                    DataField {
                        name: "title".into(),
                        label: "Title".into(),
                        example: "My Doc".into(),
                    },
                    DataField {
                        name: "body".into(),
                        label: "Body".into(),
                        example: "Content…".into(),
                    },
                ],
            },
            mappings: vec![
                FieldMapping {
                    source_field: Some("name".into()),
                    target_field: "title".into(),
                    transform: FieldTransform::Template("Repo: {{ value }}".into()),
                },
                FieldMapping {
                    source_field: Some("description".into()),
                    target_field: "body".into(),
                    transform: FieldTransform::Direct,
                },
                FieldMapping {
                    source_field: None,
                    target_field: "collection".into(),
                    transform: FieldTransform::Fixed("Documentation".into()),
                },
            ],
            trigger: DataTrigger::Manual,
            enabled: true,
            last_run: None,
        }
    }

    pub fn status_label(&self) -> &'static str {
        if self.enabled {
            "● Active"
        } else {
            "○ Inactive"
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TaskTemplate {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
    pub source_service: &'static str,
    pub source_offer: &'static str,
    pub target_service: &'static str,
    pub target_accept: &'static str,
}

#[derive(Default, Serialize, Deserialize)]
pub struct TasksConfig {
    #[serde(default)]
    pub tasks: Vec<TaskPipeline>,
}

impl TasksConfig {
    fn path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home)
            .join(".config")
            .join("fsn")
            .join("tasks.toml")
    }

    pub fn load() -> Self {
        let path = Self::path();
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str::<Self>(&content).unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::path();
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }
}
