use std::fs;
use std::path::Path;
use std::sync::RwLock;
use std::collections::{HashMap, HashSet};
use serde_json::Value;
use sqlx::PgPool;
use serde::Serialize;  // 添加

#[derive(Serialize, Clone, Debug)]
pub struct LangOption {
    pub code: String,
    pub name: String,
}




pub struct I18n {
    translations: RwLock<HashMap<String, HashMap<String, String>>>,
    supported_langs: RwLock<Vec<String>>,
    default_lang: RwLock<String>,
    locales_dir: String,
    plugins_dir: String,
}

impl I18n {
    /// 初始化时暂不加载插件语言包（需要 await 数据库查询）
    pub fn new(locales_dir: &str, plugins_dir: &str) -> Self {
        let dir = locales_dir.to_string();
        let plugins = plugins_dir.to_string();
        let supported_langs = Self::scan_langs(&dir);
        let default_lang = supported_langs.first().cloned().unwrap_or_else(|| "zh".to_string());
        let translations = Self::load_all_translations(&dir, &plugins, &supported_langs, &HashSet::new());
        
        Self {
            translations: RwLock::new(translations),
            supported_langs: RwLock::new(supported_langs),
            default_lang: RwLock::new(default_lang),
            locales_dir: dir,
            plugins_dir: plugins,
        }
    }

    /// 从数据库获取已启用插件列表后，重新加载语言包
    pub async fn init_with_plugins(&self, pool: &PgPool) {
        let enabled_plugins = Self::get_enabled_plugin_names(pool).await;
        let supported_langs = self.supported_langs();
        let default_lang = self.default_lang();
        let translations = Self::load_all_translations(
            &self.locales_dir, &self.plugins_dir, &supported_langs, &enabled_plugins
        );
        *self.translations.write().unwrap() = translations;
    }

    fn scan_langs(dir: &str) -> Vec<String> {
        let mut langs = Vec::new();
        if let Ok(entries) = fs::read_dir(Path::new(dir)) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |e| e == "json") {
                    if let Some(lang) = path.file_stem().and_then(|s| s.to_str()) {
                        langs.push(lang.to_string());
                    }
                }
            }
        }
        langs.sort();
        langs
    }

    async fn get_enabled_plugin_names(pool: &PgPool) -> HashSet<String> {
        let rows = sqlx::query!("SELECT name FROM plugins WHERE enabled = true")
            .fetch_all(pool)
            .await
            .unwrap_or_default();
        rows.into_iter().map(|r| r.name).collect()
    }

    /// 加载主程序和已启用插件的翻译
    fn load_all_translations(
        main_dir: &str,
        plugins_dir: &str,
        langs: &[String],
        enabled_plugins: &HashSet<String>,
    ) -> HashMap<String, HashMap<String, String>> {
        let mut all_translations: HashMap<String, HashMap<String, String>> = HashMap::new();
        
        for lang in langs {
            let mut merged = HashMap::new();
            
            // 1. 加载主程序语言包
            let main_file = Path::new(main_dir).join(format!("{}.json", lang));
            if let Ok(content) = fs::read_to_string(&main_file) {
                if let Ok(map) = serde_json::from_str::<Value>(&content) {
                    if let Some(obj) = map.as_object() {
                        for (k, v) in obj {
                            merged.insert(k.clone(), v.as_str().unwrap_or("").to_string());
                        }
                    }
                }
            }
            
            // 2. 只加载已启用插件的语言包
            if !enabled_plugins.is_empty() {
                if let Ok(entries) = fs::read_dir(Path::new(plugins_dir)) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let plugin_name = path.file_name().unwrap().to_string_lossy().to_string();
                            // 只处理已启用的插件
                            if !enabled_plugins.contains(&plugin_name) {
                                continue;
                            }
                            let plugin_locales_file = path.join("locales").join(format!("{}.json", lang));
                            if plugin_locales_file.exists() {
                                if let Ok(content) = fs::read_to_string(&plugin_locales_file) {
                                    if let Ok(map) = serde_json::from_str::<Value>(&content) {
                                        if let Some(obj) = map.as_object() {
                                            for (k, v) in obj {
                                                // 主程序优先，插件键不覆盖
                                                merged.entry(k.clone()).or_insert_with(|| v.as_str().unwrap_or("").to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            all_translations.insert(lang.clone(), merged);
        }
        
        all_translations
    }

    pub fn t(&self, lang: &str, key: &str) -> String {
        let translations = self.translations.read().unwrap();
        let default_lang = self.default_lang.read().unwrap();
        
        translations
            .get(lang)
            .and_then(|m| m.get(key))
            .cloned()
            .unwrap_or_else(|| {
                translations
                    .get(&*default_lang)
                    .and_then(|m| m.get(key))
                    .cloned()
                    .unwrap_or_else(|| key.to_string())
            })
    }

    pub fn supported_langs(&self) -> Vec<String> {
        self.supported_langs.read().unwrap().clone()
    }

    pub fn default_lang(&self) -> String {
        self.default_lang.read().unwrap().clone()
    }

    pub fn lang_name(&self, lang: &str) -> String {
        self.t(lang, "lang_name")
    }

    pub fn lang_options(&self) -> Vec<LangOption> {
        self.supported_langs()
            .iter()
            .map(|code| LangOption {
                code: code.clone(),
                name: self.lang_name(code),
            })
            .collect()
    }

    pub fn reload(&self, supported_langs: Vec<String>, default_lang: String) {
        // 注意：reload 时无法获取已启用插件列表，需要外部传入
        // 这里使用空的 HashSet，表示只加载主程序语言包
        let translations = Self::load_all_translations(
            &self.locales_dir, &self.plugins_dir, &supported_langs, &HashSet::new()
        );
        *self.translations.write().unwrap() = translations;
        *self.supported_langs.write().unwrap() = supported_langs;
        *self.default_lang.write().unwrap() = default_lang;
    }

    /// 带插件过滤的 reload
    pub async fn reload_with_plugins(&self, supported_langs: Vec<String>, default_lang: String, pool: &PgPool) {
        let enabled_plugins = Self::get_enabled_plugin_names(pool).await;
        let translations = Self::load_all_translations(
            &self.locales_dir, &self.plugins_dir, &supported_langs, &enabled_plugins
        );
        *self.translations.write().unwrap() = translations;
        *self.supported_langs.write().unwrap() = supported_langs;
        *self.default_lang.write().unwrap() = default_lang;
    }
}