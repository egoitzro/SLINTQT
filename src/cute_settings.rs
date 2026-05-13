use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct CuteSettings {
    org_name: String,
    app_name: String,
    data: HashMap<String, Value>,
    file_path: PathBuf,
}

impl CuteSettings {
    pub fn new(org_name: &str, app_name: &str) -> Self {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(org_name);
        path.push(app_name);
        
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }
        
        path.push("settings.json");
        
        let data = if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                serde_json::from_str(&content).unwrap_or_default()
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };

        Self {
            org_name: org_name.to_string(),
            app_name: app_name.to_string(),
            data,
            file_path: path,
        }
    }

    pub fn set_value(&mut self, key: &str, value: impl Serialize) {
        if let Ok(v) = serde_json::to_value(value) {
            self.data.insert(key.to_string(), v);
        }
    }

    pub fn value<T: for<'de> Deserialize<'de>>(&self, key: &str, default: T) -> T {
        if let Some(v) = self.data.get(key) {
            if let Ok(parsed) = serde_json::from_value(v.clone()) {
                return parsed;
            }
        }
        default
    }

    pub fn sync(&self) {
        if let Ok(content) = serde_json::to_string_pretty(&self.data) {
            let _ = fs::write(&self.file_path, content);
        }
    }
}
