use hopper::core::tool::replace_variables;
use std::path::PathBuf;

#[test]
fn test_replace_variables_path_and_name() {
    let path = PathBuf::from("/Users/qizhi/Projects/my-project");
    let cmd = "claude --path $PROJECT_PATH --name $PROJECT_NAME";

    let result = replace_variables(cmd, &path);

    assert!(result.contains("/Users/qizhi/Projects/my-project"));
    assert!(result.contains("my-project"));
}

#[test]
fn test_replace_variables_no_variables() {
    let path = PathBuf::from("/test/project");
    let cmd = "echo hello world";

    let result = replace_variables(cmd, &path);

    assert_eq!(result, "echo hello world");
}

#[test]
fn test_replace_variables_only_path() {
    let path = PathBuf::from("/tmp/test");
    let cmd = "cd $PROJECT_PATH && ls";

    let result = replace_variables(cmd, &path);

    assert!(result.contains("/tmp/test"));
    assert!(!result.contains("$PROJECT_PATH"));
}

#[test]
fn test_replace_variables_only_name() {
    let path = PathBuf::from("/workspace/myapp");
    let cmd = "echo $PROJECT_NAME";

    let result = replace_variables(cmd, &path);

    assert!(result.contains("myapp"));
    assert!(!result.contains("$PROJECT_NAME"));
}

#[test]
fn test_replace_variables_multiple_occurrences() {
    let path = PathBuf::from("/test/project");
    let cmd = "$PROJECT_NAME: $PROJECT_PATH, also known as $PROJECT_NAME";

    let result = replace_variables(cmd, &path);

    assert_eq!(result, "project: /test/project, also known as project");
}
