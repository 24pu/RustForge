// types/media.rs

use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateFolderRequest {
    pub name: String,
    pub parent_id: Option<i32>,
}

#[derive(Deserialize)]
pub struct RenameFolderRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct ListMediaParams {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub folder_id: Option<i32>,
}