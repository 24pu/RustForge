use crate::core::{Theme, ThemeMetadata, ThemeError};
use async_trait::async_trait;
use tera::{Tera, Context, Function};
use std::collections::HashMap;
use serde_json::Value;

pub struct TeraTheme {
    tera: Tera,
    metadata: ThemeMetadata,
}

impl TeraTheme {
    pub fn new(templates_dir: &str, metadata: ThemeMetadata) -> Result<Self, ThemeError> {
        let tera = Tera::new(templates_dir).map_err(|e| ThemeError::RenderError(e.to_string()))?;
        Ok(Self { tera, metadata })
    }

    /// 注册自定义 Tera 函数（例如翻译函数 t()）
    pub fn register_function<F>(&mut self, name: &str, f: F)
    where
        F: Function + 'static,
    {
        self.tera.register_function(name, f);
    }
}

#[async_trait]
impl Theme for TeraTheme {
    fn metadata(&self) -> &ThemeMetadata {
        &self.metadata
    }

    async fn render(&self, template_name: &str, context: HashMap<String, Value>) -> Result<String, ThemeError> {
        let mut ctx = Context::new();
        for (key, value) in context {
            ctx.insert(key, &value);
        }
        self.tera.render(template_name, &ctx)
            .map_err(|e| ThemeError::RenderError(e.to_string()))
    }

    async fn reload(&mut self) -> Result<(), ThemeError> {
        self.tera.full_reload().map_err(|e| ThemeError::RenderError(e.to_string()))
    }
}