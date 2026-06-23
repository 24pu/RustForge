// types/plugin.rs

use serde::Deserialize;

#[derive(Deserialize)]
pub struct InstallPluginRequest {
    pub file_path: Option<String>,
}