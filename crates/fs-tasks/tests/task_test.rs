use fs_tasks::model::{DataTrigger, FieldTransform, TaskPipeline};

#[test]
fn task_pipeline_new_default_id_format() {
    let pipeline = TaskPipeline::new_default(42);
    assert_eq!(pipeline.id, "task-42");
    assert_eq!(pipeline.name, "New Task");
}

#[test]
fn task_pipeline_status_label_enabled() {
    let mut p = TaskPipeline::new_default(1);
    p.enabled = true;
    assert_eq!(p.status_label(), "● Active");
}

#[test]
fn task_pipeline_status_label_disabled() {
    let mut p = TaskPipeline::new_default(1);
    p.enabled = false;
    assert_eq!(p.status_label(), "○ Inactive");
}

#[test]
fn data_trigger_label_manual() {
    assert_eq!(DataTrigger::Manual.label(), "Manual only");
}

#[test]
fn data_trigger_label_on_event() {
    let t = DataTrigger::OnEvent("user.created".into());
    assert!(t.label().contains("user.created"));
}

#[test]
fn data_trigger_label_scheduled() {
    let t = DataTrigger::Scheduled("0 * * * *".into());
    assert!(t.label().contains("0 * * * *"));
}

#[test]
fn field_transform_label_direct() {
    assert_eq!(FieldTransform::Direct.label(), "Direct copy");
}

#[test]
fn field_transform_label_template() {
    let t = FieldTransform::Template("{{name}}".into());
    assert!(t.label().contains("{{name}}"));
}

#[test]
fn field_transform_label_fixed() {
    let t = FieldTransform::Fixed("hello".into());
    assert!(t.label().contains("hello"));
}
