use crate::config::{Config, Tool};
use crate::project::Project;
use std::io::Write;
use std::process::{Command, Stdio};

fn check_fzf() -> bool {
    Command::new("fzf")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

pub fn project_selection(projects: &[Project]) -> Option<(usize, bool)> {
    // (index, deleted) - deleted=true means user pressed x to delete
    if !check_fzf() {
        eprintln!("fzf is not installed. Run: brew install fzf");
        return None;
    }

    let items: Vec<String> = projects
        .iter()
        .enumerate()
        .map(|(_, p)| format!("{}\t{}", p.display_name(), p.mtime_str()))
        .collect();

    if items.is_empty() {
        return None;
    }

    let mut child = Command::new("fzf")
        .args([
            "--height=40%",
            "--layout=reverse",
            "--border",
            "--prompt=dev > ",
            r#"--header=Enter: select / x: delete project"#,
            "--expect=enter,x",
            "--with-nth=1",
            "--delimiter=\t",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn fzf");

    let stdin = child.stdin.as_mut().expect("Failed to get stdin");
    for item in &items {
        stdin
            .write_all(format!("{}\n", item).as_bytes())
            .expect("Failed to write to fzf");
    }
    drop(stdin);

    let output = child
        .wait_with_output()
        .expect("Failed to wait for fzf");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    if lines.is_empty() {
        return None;
    }

    let key = lines[0];
    let selection = lines.get(1).unwrap_or(&"");

    if selection.is_empty() {
        return None;
    }

    // Find the index by matching the path
    let selected_path = selection.split('\t').next().unwrap_or("");

    let index = projects
        .iter()
        .position(|p| p.display_name() == selected_path)?;

    let deleted = key == "x";
    Some((index, deleted))
}

pub fn tool_selection(tools: &[Tool]) -> Option<usize> {
    if !check_fzf() {
        eprintln!("fzf is not installed. Run: brew install fzf");
        return None;
    }

    let mut items: Vec<String> = tools
        .iter()
        .enumerate()
        .map(|(_, t)| {
            if t.recent > 0 {
                format!("{} (use count: {})", t.name, t.recent)
            } else {
                t.name.clone()
            }
        })
        .collect();

    items.push("[Add new tool...]".to_string());
    items.push("[Cancel]".to_string());

    let mut child = Command::new("fzf")
        .args([
            "--height=50%",
            "--layout=reverse",
            "--border",
            "--prompt=Select tool > ",
            "--with-nth=1",
            "--delimiter=\t",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn fzf");

    let stdin = child.stdin.as_mut().expect("Failed to get stdin");
    for item in &items {
        stdin
            .write_all(format!("{}\n", item).as_bytes())
            .expect("Failed to write to fzf");
    }
    drop(stdin);

    let output = child
        .wait_with_output()
        .expect("Failed to wait for fzf");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let selected = stdout.trim();

    if selected.is_empty() || selected == "[Cancel]" {
        return None;
    }

    if selected == "[Add new tool...]" {
        return Some(usize::MAX); // Magic value for "add new tool"
    }

    // Find by name
    let clean_name = selected.split('\t').next().unwrap_or(selected).split('(').next().unwrap_or(selected).trim().to_string();
    tools.iter().position(|t| t.name == clean_name)
}

pub fn confirm_deletion(project_name: &str) -> bool {
    if !check_fzf() {
        eprintln!("fzf is not installed");
        return false;
    }

    let prompt = format!("Delete {} ? > ", project_name);

    let mut child = Command::new("fzf")
        .args([
            "--height=3",
            "--layout=reverse",
            "--border",
            &prompt,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn fzf");

    let stdin = child.stdin.as_mut().expect("Failed to get stdin");
    stdin
        .write_all(b"Yes\nNo\n")
        .expect("Failed to write to fzf");
    drop(stdin);

    let output = child
        .wait_with_output()
        .expect("Failed to wait for fzf");

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.trim() == "Yes"
}

pub fn add_tool_interactive(config: &mut Config) {
    print!("Tool name: ");
    std::io::stdout().flush().unwrap();
    let mut name = String::new();
    std::io::stdin().read_line(&mut name).unwrap();
    name = name.trim().to_string();

    if name.is_empty() {
        return;
    }

    print!("Command template ($PROJECT_PATH and $PROJECT_NAME will be replaced): ");
    std::io::stdout().flush().unwrap();
    let mut command = String::new();
    std::io::stdin().read_line(&mut command).unwrap();
    command = command.trim().to_string();

    if command.is_empty() {
        return;
    }

    config.tools.push(Tool {
        name,
        command,
        recent: 0,
    });

    println!("Tool added successfully");
}

pub fn onboarding_selection() -> Option<String> {
    if !check_fzf() {
        eprintln!("fzf is not installed. Run: brew install fzf");
        return None;
    }

    let mut child = Command::new("fzf")
        .args([
            "--height=30%",
            "--layout=reverse",
            "--border",
            "--prompt=Select > ",
            "--header=Bind project set: enter path to confirm",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn fzf");

    let stdin = child.stdin.as_mut().expect("Failed to get stdin");
    stdin
        .write_all(b"Bind project set...\nEnter project picker\nExit\n")
        .expect("Failed to write to fzf");
    drop(stdin);

    let output = child
        .wait_with_output()
        .expect("Failed to wait for fzf");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let selected = stdout.trim();

    match selected {
        "Bind project set..." => Some("BIND".to_string()),
        "Enter project picker" => Some("ENTER".to_string()),
        _ => Some("EXIT".to_string()),
    }
}
