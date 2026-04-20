use crate::error::{CacheError, Result};
use dirs::cache_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const MRU_FILENAME: &str = "mru.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MruData {
    #[serde(flatten)]
    pub scores: HashMap<String, u32>,
}

pub struct Cache {
    dir: PathBuf,
    mru_data: MruData,
}

impl Cache {
    pub fn new(dir: Option<PathBuf>) -> Result<Self> {
        let dir = dir
            .or_else(|| std::env::var("HOPPER_CACHE_DIR").ok().map(PathBuf::from))
            .or_else(|| cache_dir().map(|p| p.join("hopper")))
            .unwrap_or_else(|| PathBuf::from(".hopper/cache"));

        let mut cache = Self {
            dir,
            mru_data: MruData::default(),
        };

        cache.load()?;
        Ok(cache)
    }

    fn mru_path(&self) -> PathBuf {
        self.dir.join(MRU_FILENAME)
    }

    fn load(&mut self) -> Result<()> {
        let path = self.mru_path();
        if !path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| CacheError::ReadError(format!("{}: {}", path.display(), e)))?;

        self.mru_data = serde_json::from_str(&content)
            .map_err(|e| CacheError::ReadError(format!("Parse error: {}", e)))?;

        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.dir.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CacheError::AccessError(format!("{}: {}", parent.display(), e)))?;
        }

        fs::create_dir_all(&self.dir)
            .map_err(|e| CacheError::AccessError(format!("{}: {}", self.dir.display(), e)))?;

        let content = serde_json::to_string_pretty(&self.mru_data)
            .map_err(|e| CacheError::WriteError(format!("Serialize error: {}", e)))?;

        fs::write(self.mru_path(), content)
            .map_err(|e| CacheError::WriteError(format!("{}: {}", self.mru_path().display(), e)))?;

        Ok(())
    }

    pub fn record_access(&mut self, project_path: &PathBuf) {
        let key = project_path.to_string_lossy().to_string();
        *self.mru_data.scores.entry(key).or_insert(0) += 1;
    }

    pub fn get_score(&self, project_path: &PathBuf) -> u32 {
        let key = project_path.to_string_lossy().to_string();
        self.mru_data.scores.get(&key).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_record_access() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = Cache::new(Some(temp_dir.path().to_path_buf())).unwrap();

        let path = PathBuf::from("/test/project");
        cache.record_access(&path);
        cache.save().unwrap();

        assert_eq!(cache.get_score(&path), 1);

        cache.record_access(&path);
        assert_eq!(cache.get_score(&path), 2);
    }

    #[test]
    fn test_cache_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let path1 = temp_dir.path().to_path_buf();

        {
            let mut cache = Cache::new(Some(path1.clone())).unwrap();
            cache.record_access(&PathBuf::from("/test/project"));
            cache.save().unwrap();
        }

        {
            let cache = Cache::new(Some(path1)).unwrap();
            assert_eq!(cache.get_score(&PathBuf::from("/test/project")), 1);
        }
    }
}
