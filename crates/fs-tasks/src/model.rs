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
            Self::Manual          => "Manual only".into(),
            Self::OnEvent(ev)     => format!("On event: {ev}"),
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
            Self::Direct         => "Direct copy".into(),
            Self::Template(tmpl) => format!("Template: {tmpl}"),
            Self::Fixed(val)     => format!("Fixed: {val}"),
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
    pub fn status_label(&self) -> &'static str {
        if self.enabled { "● Active" } else { "○ Inactive" }
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
        PathBuf::from(home).join(".config").join("fsn").join("tasks.toml")
    }

    pub fn load() -> Vec<TaskPipeline> {
        let path = Self::path();
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str::<Self>(&content).unwrap_or_default().tasks
    }

    pub fn save(tasks: &[TaskPipeline]) -> Result<(), String> {
        let path = Self::path();
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
        }
        let cfg = Self { tasks: tasks.to_vec() };
        let content = toml::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }
}
