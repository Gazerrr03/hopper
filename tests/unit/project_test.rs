use hopper::core::cache::Cache;
use hopper::core::project::{discover_projects, find_project, sort_by_mru, Project};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_discover_empty_project_sets() {
    let sets: Vec<PathBuf> = vec![];
    let projects = discover_projects(&sets);
    assert!(projects.is_empty());
}

#[test]
fn test_find_project_exact_match() {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path().to_path_buf();

    let projects = vec![
        Project {
            path: base.join("project1"),
            mtime: None,
            base_path: base.clone(),
        },
        Project {
            path: base.join("project2"),
            mtime: None,
            base_path: base.clone(),
        },
    ];

    assert!(find_project("project1", &projects).is_some());
    assert!(find_project("PROJECT1", &projects).is_some());
    assert!(find_project("nonexistent", &projects).is_none());
}

#[test]
fn test_find_project_prefix_match() {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path().to_path_buf();

    let projects = vec![
        Project {
            path: base.join("my-project"),
            mtime: None,
            base_path: base.clone(),
        },
    ];

    assert!(find_project("my", &projects).is_some());
    assert!(find_project("my-", &projects).is_some());
}

#[test]
fn test_find_project_substring_match() {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path().to_path_buf();

    let projects = vec![
        Project {
            path: base.join("backend-api"),
            mtime: None,
            base_path: base.clone(),
        },
    ];

    assert!(find_project("end", &projects).is_some());
    assert!(find_project("api", &projects).is_some());
}

#[test]
fn test_sort_by_mru_empty() {
    let temp_dir = TempDir::new().unwrap();
    let cache = Cache::new(Some(temp_dir.path().to_path_buf())).unwrap();
    let mut projects: Vec<Project> = vec![];
    sort_by_mru(&mut projects, &cache);
    assert!(projects.is_empty());
}

#[test]
fn test_project_display_name() {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path().to_path_buf();

    let project = Project {
        path: base.join("myproject"),
        mtime: None,
        base_path: base.clone(),
    };

    assert_eq!(project.display_name(), "myproject");
}
