use fs_lenses::model::{Lens, LensItem, LensRole};

#[test]
fn lens_new_sets_name_and_query() {
    let lens = Lens::new("My Lens", "rust");
    assert_eq!(lens.name, "My Lens");
    assert_eq!(lens.query, "rust");
    assert!(lens.items.is_empty());
    assert!(lens.last_refreshed.is_none());
}

#[test]
fn lens_role_id_stable() {
    assert_eq!(LensRole::Wiki.id(), "wiki");
    assert_eq!(LensRole::Chat.id(), "chat");
    assert_eq!(LensRole::Git.id(), "git");
    assert_eq!(LensRole::Tasks.id(), "tasks");
    assert_eq!(LensRole::Other("custom".into()).id(), "other:custom");
}

#[test]
fn lens_grouped_empty_when_no_items() {
    let lens = Lens::new("empty", "query");
    assert!(lens.grouped().is_empty());
}

#[test]
fn lens_grouped_single_role() {
    let mut lens = Lens::new("test", "query");
    lens.items.push(LensItem {
        role: LensRole::Wiki,
        summary: "Page 1".into(),
        link: None,
        source: "wiki-service".into(),
    });
    lens.items.push(LensItem {
        role: LensRole::Wiki,
        summary: "Page 2".into(),
        link: None,
        source: "wiki-service".into(),
    });
    let grouped = lens.grouped();
    assert_eq!(grouped.len(), 1);
    assert_eq!(grouped[0].0, LensRole::Wiki);
    assert_eq!(grouped[0].1.len(), 2);
}

#[test]
fn lens_grouped_multiple_roles() {
    let mut lens = Lens::new("test", "query");
    lens.items.push(LensItem {
        role: LensRole::Wiki,
        summary: "wiki result".into(),
        link: None,
        source: "wiki".into(),
    });
    lens.items.push(LensItem {
        role: LensRole::Chat,
        summary: "chat result".into(),
        link: None,
        source: "matrix".into(),
    });
    let grouped = lens.grouped();
    assert_eq!(grouped.len(), 2);
}

#[test]
fn lens_role_other_equals_by_content() {
    assert_eq!(LensRole::Other("x".into()), LensRole::Other("x".into()));
    assert_ne!(LensRole::Other("x".into()), LensRole::Other("y".into()));
}
