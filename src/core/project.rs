use crate::core::cache::Cache;
use crate::error::{ProjectError, Result};
use chrono::{DateTime, Local};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub path: PathBuf,
    #[serde(with = "opt_system_time_serde")]
    pub mtime: Option<SystemTime>,
    pub base_path: PathBuf,
}

mod opt_system_time_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::SystemTime;

    pub fn serialize<S>(time: &Option<SystemTime>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match time {
            Some(t) => s.serialize_u64(
                t.duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ),
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
        if let Ok(stripped) = self.path.strip_prefix(&self.base_path) {
            let name = stripped.to_string_lossy();
            if name.is_empty() {
                self.path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| self.path.to_string_lossy().to_string())
            } else {
                name.to_string()
            }
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
    project_sets
        .par_iter()
        .flat_map_iter(|set_path| discover_single_set(set_path))
        .collect()
}

fn discover_single_set(set_path: &PathBuf) -> Vec<Project> {
    let mut projects = Vec::new();

    if !set_path.exists() || !set_path.is_dir() {
        return projects;
    }

    if let Ok(entries) = fs::read_dir(set_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let mtime = path.metadata().and_then(|m| m.modified()).ok();
                projects.push(Project {
                    path,
                    mtime,
                    base_path: set_path.clone(),
                });
            }
        }
    }

    projects
}

pub fn sort_by_mru(projects: &mut Vec<Project>, cache: &Cache) {
    let now = SystemTime::now();

    projects.sort_by(|a, b| {
        let score_a = cache.get_score(&a.path);
        let score_b = cache.get_score(&b.path);

        let recency_a = a
            .mtime
            .map(|t| now.duration_since(t).unwrap_or_default().as_secs())
            .unwrap_or(u64::MAX);
        let recency_b = b
            .mtime
            .map(|t| now.duration_since(t).unwrap_or_default().as_secs())
            .unwrap_or(u64::MAX);

        // 综合评分: mru_score * 100000 + (MAX_TIME - recency)
        let rank_a = score_a as u64 * 100000 + (u64::MAX / 2).saturating_sub(recency_a);
        let rank_b = score_b as u64 * 100000 + (u64::MAX / 2).saturating_sub(recency_b);

        rank_b.cmp(&rank_a)
    });
}

pub fn find_project<'a>(name: &str, projects: &'a [Project]) -> Option<&'a Project> {
    let name_lower = name.to_lowercase();

    // Exact match first
    if let Some(p) = projects.iter().find(|p| p.display_name().to_lowercase() == name_lower) {
        return Some(p);
    }

    // Prefix match
    if let Some(p) = projects
        .iter()
        .find(|p| p.display_name().to_lowercase().starts_with(&name_lower))
    {
        return Some(p);
    }

    // Substring match
    projects
        .iter()
        .find(|p| p.display_name().to_lowercase().contains(&name_lower))
}

pub fn delete_project(project: &Project) -> Result<()> {
    fs::remove_dir_all(&project.path).map_err(|e| ProjectError::DeleteError(e.to_string()).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_empty() {
        let sets: Vec<PathBuf> = vec![];
        let projects = discover_projects(&sets);
        assert!(projects.is_empty());
    }

    #[test]
    fn test_find_project_exact() {
        let projects = vec![Project {
            path: PathBuf::from("/tmp/test"),
            mtime: None,
            base_path: PathBuf::from("/tmp"),
        }];

        assert!(find_project("test", &projects).is_some());
        assert!(find_project("TEST", &projects).is_some());
        assert!(find_project("nonexistent", &projects).is_none());
    }
}
