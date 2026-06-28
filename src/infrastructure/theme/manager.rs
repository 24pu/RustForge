use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use walkdir::WalkDir;
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{warn, info};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::core::{Theme, ThemeManager, ThemeMetadata, ThemeError};
use crate::core::PluginHookRepository;
use crate::infrastructure::i18n::I18n;
use crate::core::models::PluginHook;
use super::tera_theme::TeraTheme;

// 编译一次正则，用于匹配 {{ key }}
static TEMPLATE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{\{\s*([^}]+)\s*\}\}").unwrap()
});


pub struct TeraThemeManager {
    themes: RwLock<HashMap<String, Box<dyn Theme>>>,
    active: RwLock<String>,
    hook_repo: Option<Arc<dyn PluginHookRepository>>,
    i18n: Arc<I18n>,
}

impl TeraThemeManager {
    pub async fn scan_and_load(
        themes_dir: &str,
        i18n: Arc<I18n>,
        default_theme: &str,
        hook_repo: Option<Arc<dyn PluginHookRepository>>,
    ) -> Result<Self, ThemeError> {
        let mut themes_map = HashMap::new();
        let entries = WalkDir::new(themes_dir)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir());

        for entry in entries {
            let theme_name = entry.file_name().to_string_lossy().to_string();
            let theme_path = entry.path();
            let meta_path = theme_path.join("theme.toml");
            if !meta_path.exists() {
                warn!("Skipping {}: missing theme.toml", theme_name);
                continue;
            }
            let metadata: ThemeMetadata = toml::from_str(
                &std::fs::read_to_string(&meta_path)
                    .map_err(|e| ThemeError::LoadError(e.to_string()))?
            ).map_err(|e| ThemeError::LoadError(e.to_string()))?;

            let templates_dir = theme_path.join("templates");
            if !templates_dir.exists() {
                warn!("Skipping {}: no templates directory", theme_name);
                continue;
            }
            let pattern = format!("{}/**/*.html", templates_dir.display());

            let mut tera_theme = TeraTheme::new(&pattern, metadata.clone())
                .map_err(|e| ThemeError::LoadError(e.to_string()))?;

            let i18n_clone = i18n.clone();
            tera_theme.register_function("t", move |args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
                let lang = args.get("lang").and_then(|v| v.as_str()).unwrap_or("zh");
                let key = args.get("key").and_then(|v| v.as_str()).unwrap_or("");
                Ok(tera::Value::String(i18n_clone.t(lang, key)))
            });

            themes_map.insert(theme_name.clone(), Box::new(tera_theme) as Box<dyn Theme>);
            info!("Loaded theme: {}", theme_name);
        }

        if themes_map.is_empty() {
            return Err(ThemeError::ScanError("No valid themes found".into()));
        }

        let active = if themes_map.contains_key(default_theme) {
            default_theme.to_string()
        } else {
            warn!("Default theme '{}' not found, using first available theme", default_theme);
            themes_map.keys().next().unwrap().clone()
        };

        Ok(Self {
            themes: RwLock::new(themes_map),
            active: RwLock::new(active),
            hook_repo,
            i18n,
        })
    }

    pub async fn add_theme(&self, name: &str, theme: Box<dyn Theme>) {
        let mut themes = self.themes.write().await;
        themes.insert(name.to_string(), theme);
    }

    /// 加载钩子内容并替换其中的 {{ key }} 变量
 

   async fn load_hooks(&self, lang: &str) -> HashMap<String, String> {
        let mut hooks = HashMap::new();
        if let Some(repo) = &self.hook_repo {
            // 1. 获取通用（lang=''）且所属插件已启用的钩子
            let generic = repo.list_enabled_by_lang("").await.unwrap_or_default();
            // 2. 获取当前语言且所属插件已启用的钩子
            let specific = repo.list_enabled_by_lang(lang).await.unwrap_or_default();

            // 合并、分组、排序（逻辑保持不变）
            let mut all = generic;
            all.extend(specific);

            let mut groups: HashMap<String, Vec<PluginHook>> = HashMap::new();
            for hook in all {
                groups.entry(hook.hook_name.clone()).or_default().push(hook);
            }

            for (hook_name, mut hooks_vec) in groups {
                hooks_vec.sort_by_key(|h| h.sort_order);
                let generic_vec: Vec<_> = hooks_vec.iter()
                    .filter(|h| h.lang.is_none() || h.lang == Some("".to_string()))
                    .collect();
                let specific_vec: Vec<_> = hooks_vec.iter()
                    .filter(|h| h.lang == Some(lang.to_string()))
                    .collect();
                let selected = if !specific_vec.is_empty() { specific_vec } else { generic_vec };
                if selected.is_empty() {
                    continue;
                }
                let combined = selected.iter()
                    .map(|h| h.content.clone())
                    .collect::<Vec<String>>()
                    .join("\n");
                let translated = TEMPLATE_REGEX.replace_all(&combined, |caps: &regex::Captures| {
                    let key = caps.get(1).unwrap().as_str().trim();
                    self.i18n.t(lang, key)
                });
                hooks.insert(hook_name, translated.to_string());
            }
             
        }
        hooks
       
    }
}

#[async_trait]
impl ThemeManager for TeraThemeManager {
    fn list_themes(&self) -> Vec<ThemeMetadata> {
        let themes = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.themes.read())
        });
        themes.values()
            .map(|t| t.metadata().clone())
            .collect()
    }

    fn active_theme(&self) -> String {
        let active = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.active.read())
        });
        active.clone()
    }

    fn set_active_theme(&mut self, name: &str) -> Result<(), ThemeError> {
        let themes = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.themes.read())
        });
        if themes.contains_key(name) {
            let mut active = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(self.active.write())
            });
            *active = name.to_string();
            Ok(())
        } else {
            Err(ThemeError::ThemeNotFound(name.to_string()))
        }
    }

    async fn render(&self, template: &str, context: HashMap<String, Value>) -> Result<String, ThemeError> {
        let themes = self.themes.read().await;
        let active = self.active.read().await;
        let theme = themes.get(&*active)
            .ok_or_else(|| ThemeError::ThemeNotFound(active.clone()))?;

        // 获取当前语言
        let lang = context
            .get("lang")
            .and_then(|v| v.as_str())
            .unwrap_or("zh")
            .to_string();

        // 加载钩子内容（并翻译）
        let hooks = self.load_hooks(&lang).await;
        

        // 合并钩子到 context
        let mut new_context = context.clone();
        if let Some(existing) = new_context.get_mut("hooks") {
            if let Value::Object(ref mut map) = existing {
                for (k, v) in hooks {
                    map.insert(k, Value::String(v));
                }
            }
        } else {
            let mut hook_map = serde_json::Map::new();
            for (k, v) in hooks {
                hook_map.insert(k, Value::String(v));
            }
            new_context.insert("hooks".to_string(), Value::Object(hook_map));
        }

        theme.render(template, new_context).await
    }

    async fn reload_theme(&self, theme_name: &str) -> Result<(), ThemeError> {
        let mut themes = self.themes.write().await;
        if let Some(theme) = themes.get_mut(theme_name) {
            theme.reload().await?;
            Ok(())
        } else {
            Err(ThemeError::ThemeNotFound(theme_name.to_string()))
        }
    }
}