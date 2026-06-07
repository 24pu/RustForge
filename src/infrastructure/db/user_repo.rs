use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::Utc;
use anyhow::Result;
use async_trait::async_trait;
use crate::core::models::{User, Permission, RoleInfo};
use crate::core::UserRepository;

pub struct PostgresUserRepo {
    pub pool: PgPool,
}

impl PostgresUserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepo {
    async fn create_user(&self, email: &str, password_hash: &str, name: Option<&str>) -> Result<User> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let row = sqlx::query(
            "INSERT INTO users (id, email, password_hash, name, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, email, name, password_hash, created_at, updated_at"
        )
        .bind(id)
        .bind(email)
        .bind(password_hash)
        .bind(name)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;
        Ok(User {
            id: row.get("id"),
            email: row.get("email"),
            name: row.get("name"),
            password_hash: row.get("password_hash"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let row = sqlx::query(
            "SELECT id, email, name, password_hash, created_at, updated_at FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| User {
            id: r.get("id"),
            email: r.get("email"),
            name: r.get("name"),
            password_hash: r.get("password_hash"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let row = sqlx::query(
            "SELECT id, email, name, password_hash, created_at, updated_at FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| User {
            id: r.get("id"),
            email: r.get("email"),
            name: r.get("name"),
            password_hash: r.get("password_hash"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn update_user(&self, id: Uuid, name: Option<String>) -> Result<User> {
        let now = Utc::now();
        let row = sqlx::query(
            "UPDATE users SET name = $1, updated_at = $2 WHERE id = $3
             RETURNING id, email, name, created_at, updated_at"
        )
        .bind(name)
        .bind(now)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(User {
            id: row.get("id"),
            email: row.get("email"),
            name: row.get("name"),
            password_hash: String::new(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }



    async fn delete_user(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query!("DELETE FROM users WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    // 角色管理
    async fn list_roles(&self) -> Result<Vec<RoleInfo>> {
        let rows = sqlx::query!("SELECT id, name, description FROM roles ORDER BY id")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter()
            .map(|r| RoleInfo { id: r.id, name: r.name, description: r.description })
            .collect())
    }

    async fn create_role(&self, name: &str, description: Option<&str>) -> Result<RoleInfo> {
        let row = sqlx::query!(
            "INSERT INTO roles (name, description) VALUES ($1, $2) RETURNING id, name, description",
            name, description
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(RoleInfo { id: row.id, name: row.name, description: row.description })
    }

    async fn update_role(&self, role_id: i32, name: &str, description: Option<&str>) -> Result<RoleInfo> {
        let row = sqlx::query!(
            "UPDATE roles SET name = $1, description = $2 WHERE id = $3 RETURNING id, name, description",
            name, description, role_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(RoleInfo { id: row.id, name: row.name, description: row.description })
    }

    async fn delete_role(&self, role_id: i32) -> Result<bool> {
        let res = sqlx::query!("DELETE FROM roles WHERE id = $1", role_id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    async fn assign_role_by_name(&self, user_id: Uuid, role_name: &str) -> Result<()> {
        let role_row = sqlx::query!("SELECT id FROM roles WHERE name = $1", role_name)
            .fetch_optional(&self.pool)
            .await?;
        let role_id = role_row.ok_or_else(|| anyhow::anyhow!("Role not found"))?.id;
        sqlx::query!(
            "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            user_id, role_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn revoke_role_by_name(&self, user_id: Uuid, role_name: &str) -> Result<()> {
        let role_row = sqlx::query!("SELECT id FROM roles WHERE name = $1", role_name)
            .fetch_optional(&self.pool)
            .await?;
        let role_id = role_row.ok_or_else(|| anyhow::anyhow!("Role not found"))?.id;
        sqlx::query!(
            "DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2",
            user_id, role_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<String>> {
        let rows = sqlx::query!(
            "SELECT r.name FROM user_roles ur JOIN roles r ON ur.role_id = r.id WHERE ur.user_id = $1",
            user_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.name).collect())
    }

    async fn list_users_with_roles(&self, limit: i64, offset: i64) -> Result<Vec<(User, Vec<String>)>> {
        let users = sqlx::query_as!(
            User,
            "SELECT id, email, name, password_hash, created_at, updated_at FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            limit, offset
        )
        .fetch_all(&self.pool)
        .await?;
        let mut result = Vec::new();
        for user in users {
            let roles = self.get_user_roles(user.id).await?;
            result.push((user, roles));
        }
        Ok(result)
    }

    // 权限管理
    async fn list_permissions(&self) -> Result<Vec<Permission>> {
        let rows = sqlx::query!(
            "SELECT id, name, description, module FROM permissions ORDER BY module, name"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter()
            .map(|r| Permission {
                id: r.id,
                name: r.name,
                description: r.description,
                module: r.module,
            })
            .collect())
    }

    async fn get_role_permissions(&self, role_id: i32) -> Result<Vec<Permission>> {
        let rows = sqlx::query!(
            "SELECT p.id, p.name, p.description, p.module
             FROM role_permissions rp
             JOIN permissions p ON rp.permission_id = p.id
             WHERE rp.role_id = $1",
            role_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter()
            .map(|r| Permission {
                id: r.id,
                name: r.name,
                description: r.description,
                module: r.module,
            })
            .collect())
    }

    async fn assign_permission(&self, role_id: i32, permission_id: i32) -> Result<()> {
        sqlx::query!(
            "INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            role_id, permission_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn revoke_permission(&self, role_id: i32, permission_id: i32) -> Result<()> {
        sqlx::query!(
            "DELETE FROM role_permissions WHERE role_id = $1 AND permission_id = $2",
            role_id, permission_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_role_permissions(&self, role_id: i32, permission_ids: &[i32]) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!("DELETE FROM role_permissions WHERE role_id = $1", role_id)
            .execute(&mut *tx)
            .await?;
        for &pid in permission_ids {
            sqlx::query!(
                "INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2)",
                role_id, pid
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn user_has_permission(&self, user_id: Uuid, permission: &str) -> Result<bool> {
        let row = sqlx::query!(
            "SELECT EXISTS (
                SELECT 1 FROM user_roles ur
                JOIN role_permissions rp ON ur.role_id = rp.role_id
                JOIN permissions p ON rp.permission_id = p.id
                WHERE ur.user_id = $1 AND p.name = $2
            ) as exists",
            user_id, permission
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.exists.unwrap_or(false))
    }

    // 移除旧的实现（返回 bool 的版本），替换为：
    async fn update_password(&self, user_id: Uuid, password_hash: &str) -> Result<()> {
        sqlx::query("UPDATE users SET password_hash = $1, updated_at = $2 WHERE id = $3")
            .bind(password_hash)
            .bind(Utc::now())
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
