use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct CuteTranslator {
    locale: String,
    translations: HashMap<String, String>,
}

impl CuteTranslator {
    pub fn new(locales_dir: &str) -> Self {
        // Detect OS locale, e.g. "es-ES" -> "es"
        let sys_locale = sys_locale::get_locale().unwrap_or_else(|| "en".to_string());
        let lang = sys_locale.split('-').next().unwrap_or("en").to_string();

        let mut t = Self {
            locale: lang.clone(),
            translations: HashMap::new(),
        };

        t.load_language(locales_dir, &lang);
        t
    }

    pub fn load_language(&mut self, dir: &str, lang: &str) {
        let path = Path::new(dir).join(format!("{}.json", lang));
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(json) = serde_json::from_str::<HashMap<String, String>>(&content) {
                    self.translations = json;
                }
            }
        } else if lang != "en" {
            // Fallback to English
            let path_en = Path::new(dir).join("en.json");
            if path_en.exists() {
                if let Ok(content) = fs::read_to_string(&path_en) {
                    if let Ok(json) = serde_json::from_str::<HashMap<String, String>>(&content) {
                        self.translations = json;
                    }
                }
            }
        }
    }

    pub fn tr(&self, key: &str) -> String {
        self.translations.get(key).cloned().unwrap_or_else(|| key.to_string())
    }

    pub fn current_locale(&self) -> &str {
        &self.locale
    }
}
