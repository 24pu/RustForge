// types/role.rs

use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdateRolesRequest {
    pub roles: Vec<String>,
}

#[derive(Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateRolePermissionsRequest {
    pub permission_ids: Vec<i32>,
}