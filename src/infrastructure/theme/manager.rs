use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use walkdir::WalkDir;
use serde_json::Value;
use tokio::sync::RwLock;

use crate::core::{Theme, ThemeManager, ThemeMetadata, ThemeError};
use crate::infrastructure::i18n::I18n;
use super::tera_theme::TeraTheme;

pub struct TeraThemeManager {
    themes: RwLock<HashMap<String, Box<dyn Theme>>>,
    active: RwLock<String>,
}

impl TeraThemeManager {
    pub async fn scan_and_load(themes_dir: &str, i18n: Arc<I18n>) -> Result<Self, ThemeError> {
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
                tracing::warn!("Skipping {}: missing theme.toml", theme_name);
                continue;
            }
            let metadata: ThemeMetadata = toml::from_str(
                &std::fs::read_to_string(&meta_path).map_err(|e| ThemeError::LoadError(e.to_string()))?
            ).map_err(|e| ThemeError::LoadError(e.to_string()))?;

            let templates_dir = theme_path.join("templates");
            if !templates_dir.exists() {
                tracing::warn!("Skipping {}: no templates directory", theme_name);
                continue;
            }
            let pattern = format!("{}/**/*.html", templates_dir.display());

            // 创建 Tera 实例并注册翻译函数
            let mut tera_theme = TeraTheme::new(&pattern, metadata.clone())
                .map_err(|e| ThemeError::LoadError(e.to_string()))?;

            // 注册翻译函数 t()
            let i18n_clone = i18n.clone();
            tera_theme.register_function("t", move |args: &std::collections::HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
                let lang = args.get("lang").and_then(|v| v.as_str()).unwrap_or("zh");
                let key = args.get("key").and_then(|v| v.as_str()).unwrap_or("");
                Ok(tera::Value::String(i18n_clone.t(lang, key)))
            });

            themes_map.insert(theme_name.clone(), Box::new(tera_theme) as Box<dyn Theme>);
            tracing::info!("Loaded theme: {}", theme_name);
        }

        if themes_map.is_empty() {
            return Err(ThemeError::ScanError("No valid themes found".into()));
        }

        let active = themes_map.keys().next().unwrap().clone();
        Ok(Self {
            themes: RwLock::new(themes_map),
            active: RwLock::new(active),
        })
    }

    pub async fn add_theme(&self, name: &str, theme: Box<dyn Theme>) {
        let mut themes = self.themes.write().await;
        themes.insert(name.to_string(), theme);
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
        theme.render(template, context).await
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