use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub path: PathBuf,
    #[serde(with = "opt_system_time_serde")]
    pub mtime: Option<SystemTime>,
}

mod opt_system_time_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::SystemTime;

    pub fn serialize<S>(time: &Option<SystemTime>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match time {
            Some(t) => s.serialize_u64(t.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Option<SystemTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<u64> = Option::deserialize(d)?;
        Ok(opt.map(|v| SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(v)))
    }
}

impl Project {
    pub fn display_name(&self) -> String {
        self.path.to_string_lossy().to_string()
    }

    pub fn relative_name(&self, base: &PathBuf) -> String {
        if let Ok(stripped) = self.path.strip_prefix(base) {
            stripped.to_string_lossy().to_string()
        } else {
            self.path.to_string_lossy().to_string()
        }
    }

    pub fn mtime_str(&self) -> String {
        match self.mtime {
            Some(t) => {
                let datetime: DateTime<Local> = t.into();
                let now = Local::now();
                let duration = now.signed_duration_since(datetime);

                if duration.num_hours() < 1 {
                    format!("{} min ago", duration.num_minutes().max(0))
                } else if duration.num_hours() < 24 {
                    format!("{}h ago", duration.num_hours())
                } else if duration.num_days() < 7 {
                    format!("{}d ago", duration.num_days())
                } else {
                    datetime.format("%Y-%m-%d").to_string()
                }
            }
            None => "unknown".to_string(),
        }
    }
}

pub fn discover_projects(project_sets: &[PathBuf]) -> Vec<Project> {
    let mut projects = Vec::new();

    for set_path in project_sets {
        if !set_path.exists() || !set_path.is_dir() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(set_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let mtime = path.metadata().and_then(|m| m.modified()).ok();
                    projects.push(Project { path, mtime });
                }
            }
        }
    }

    // Sort by mtime descending (most recent first)
    projects.sort_by(|a, b| b.mtime.cmp(&a.mtime));
    projects
}

pub fn delete_project(project: &Project) -> std::io::Result<()> {
    fs::remove_dir_all(&project.path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_empty() {
        let sets: Vec<PathBuf> = vec![];
        let projects = discover_projects(&sets);
        assert_eq!(projects.len(), 0);
    }
}
