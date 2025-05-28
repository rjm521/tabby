use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{query, query_as, query_scalar, FromRow};

use super::DbConn;
use crate::SQLXResultExt;

/// 可用模型数据结构（数据库层）
#[derive(FromRow)]
pub struct AvailableModelDAO {
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub model_type: String,
    pub provider: String,
    pub performance_tier: String,
    pub max_tokens: Option<i32>,
    pub context_window: Option<i32>,
    pub enabled: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 用户模型偏好数据结构（数据库层）
#[derive(FromRow)]
pub struct UserModelPreferenceDAO {
    pub id: i64,
    pub user_id: i64,
    pub completion_model_id: Option<i64>,
    pub chat_model_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 数据库操作实现
impl DbConn {
    // === 可用模型相关操作 ===

    /// 获取所有可用模型
    pub async fn list_available_models(&self, model_type: Option<&str>) -> Result<Vec<AvailableModelDAO>> {
        let sql = if model_type.is_some() {
            "SELECT id, name, display_name, model_type, provider, performance_tier, max_tokens, context_window, enabled, description, created_at, updated_at FROM available_models WHERE model_type = ? ORDER BY created_at DESC"
        } else {
            "SELECT id, name, display_name, model_type, provider, performance_tier, max_tokens, context_window, enabled, description, created_at, updated_at FROM available_models ORDER BY created_at DESC"
        };

        let models = if let Some(model_type) = model_type {
            sqlx::query_as::<_, AvailableModelDAO>(sql)
                .bind(model_type)
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query_as::<_, AvailableModelDAO>(sql)
                .fetch_all(&self.pool)
                .await?
        };

        Ok(models)
    }

    /// 根据ID获取可用模型
    pub async fn get_available_model(&self, id: i64) -> Result<Option<AvailableModelDAO>> {
        let sql = "SELECT id, name, display_name, model_type, provider, performance_tier, max_tokens, context_window, enabled, description, created_at, updated_at FROM available_models WHERE id = ?";

        let model = sqlx::query_as::<_, AvailableModelDAO>(sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(model)
    }

    /// 创建可用模型
    pub async fn create_available_model(
        &self,
        name: &str,
        display_name: &str,
        model_type: &str,
        provider: &str,
        performance_tier: &str,
        max_tokens: Option<i32>,
        context_window: Option<i32>,
        enabled: bool,
        description: Option<&str>,
    ) -> Result<i64> {
        let sql = "INSERT INTO available_models (name, display_name, model_type, provider, performance_tier, max_tokens, context_window, enabled, description) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";

        let res = sqlx::query(sql)
            .bind(name)
            .bind(display_name)
            .bind(model_type)
            .bind(provider)
            .bind(performance_tier)
            .bind(max_tokens)
            .bind(context_window)
            .bind(enabled)
            .bind(description)
            .execute(&self.pool)
            .await
            .unique_error("Model with this name already exists")?;

        Ok(res.last_insert_rowid())
    }

    /// 更新可用模型
    pub async fn update_available_model(
        &self,
        id: i64,
        display_name: Option<&str>,
        performance_tier: Option<&str>,
        max_tokens: Option<i32>,
        context_window: Option<i32>,
        enabled: Option<bool>,
        description: Option<&str>,
    ) -> Result<()> {
        let sql = "UPDATE available_models SET display_name = COALESCE(?, display_name), performance_tier = COALESCE(?, performance_tier), max_tokens = COALESCE(?, max_tokens), context_window = COALESCE(?, context_window), enabled = COALESCE(?, enabled), description = COALESCE(?, description), updated_at = DATETIME('now') WHERE id = ?";

        sqlx::query(sql)
            .bind(display_name)
            .bind(performance_tier)
            .bind(max_tokens)
            .bind(context_window)
            .bind(enabled)
            .bind(description)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 删除可用模型
    pub async fn delete_available_model(&self, id: i64) -> Result<()> {
        let sql = "DELETE FROM available_models WHERE id = ?";
        sqlx::query(sql)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // === 用户模型偏好相关操作 ===

    /// 获取用户模型偏好
    pub async fn get_user_model_preference(&self, user_id: i64) -> Result<Option<UserModelPreferenceDAO>> {
        let sql = "SELECT id, user_id, completion_model_id, chat_model_id, created_at, updated_at FROM user_model_preferences WHERE user_id = ?";

        let preference = sqlx::query_as::<_, UserModelPreferenceDAO>(sql)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(preference)
    }

    /// 更新或创建用户模型偏好
    pub async fn upsert_user_model_preference(
        &self,
        user_id: i64,
        completion_model_id: Option<i64>,
        chat_model_id: Option<i64>,
    ) -> Result<i64> {
        // 先尝试更新
        let update_sql = "UPDATE user_model_preferences SET completion_model_id = ?, chat_model_id = ?, updated_at = DATETIME('now') WHERE user_id = ?";
        let updated = sqlx::query(update_sql)
            .bind(completion_model_id)
            .bind(chat_model_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        if updated.rows_affected() > 0 {
            // 获取更新的记录ID
            let select_sql = "SELECT id FROM user_model_preferences WHERE user_id = ?";
            let id: i64 = sqlx::query_scalar(select_sql)
                .bind(user_id)
                .fetch_one(&self.pool)
                .await?;
            Ok(id)
        } else {
            // 创建新记录
            let insert_sql = "INSERT INTO user_model_preferences (user_id, completion_model_id, chat_model_id) VALUES (?, ?, ?)";
            let res = sqlx::query(insert_sql)
                .bind(user_id)
                .bind(completion_model_id)
                .bind(chat_model_id)
                .execute(&self.pool)
                .await?;

            Ok(res.last_insert_rowid())
        }
    }

    /// 获取用户当前使用的代码补全模型
    pub async fn get_user_completion_model(&self, user_id: i64) -> Result<Option<AvailableModelDAO>> {
        let sql = "SELECT am.id, am.name, am.display_name, am.model_type, am.provider, am.performance_tier, am.max_tokens, am.context_window, am.enabled, am.description, am.created_at, am.updated_at FROM available_models am INNER JOIN user_model_preferences ump ON am.id = ump.completion_model_id WHERE ump.user_id = ? AND am.enabled = true";

        let model = sqlx::query_as::<_, AvailableModelDAO>(sql)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(model)
    }

    /// 获取用户当前使用的聊天模型
    pub async fn get_user_chat_model(&self, user_id: i64) -> Result<Option<AvailableModelDAO>> {
        let sql = "SELECT am.id, am.name, am.display_name, am.model_type, am.provider, am.performance_tier, am.max_tokens, am.context_window, am.enabled, am.description, am.created_at, am.updated_at FROM available_models am INNER JOIN user_model_preferences ump ON am.id = ump.chat_model_id WHERE ump.user_id = ? AND am.enabled = true";

        let model = sqlx::query_as::<_, AvailableModelDAO>(sql)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutils::create_user;

    #[tokio::test]
    async fn test_available_models() {
        let conn = DbConn::new_in_memory().await.unwrap();

        // 创建一个可用模型
        let model_id = conn
            .create_available_model(
                "gpt-4",
                "GPT-4",
                "chat",
                "openai",
                "quality",
                Some(4096),
                Some(8192),
                true,
                Some("GPT-4 model for chat"),
            )
            .await
            .unwrap();

        // 获取模型
        let model = conn.get_available_model(model_id).await.unwrap().unwrap();
        assert_eq!(model.name, "gpt-4");
        assert_eq!(model.display_name, "GPT-4");
        assert_eq!(model.model_type, "chat");

        // 列出所有模型
        let models = conn.list_available_models(None).await.unwrap();
        assert_eq!(models.len(), 1);

        // 按类型过滤
        let chat_models = conn.list_available_models(Some("chat")).await.unwrap();
        assert_eq!(chat_models.len(), 1);

        let completion_models = conn.list_available_models(Some("completion")).await.unwrap();
        assert_eq!(completion_models.len(), 0);
    }

    #[tokio::test]
    async fn test_user_model_preferences() {
        let conn = DbConn::new_in_memory().await.unwrap();

        // 创建用户
        let user_id = create_user(&conn).await;

        // 创建模型
        let completion_model_id = conn
            .create_available_model(
                "codegen",
                "CodeGen",
                "completion",
                "salesforce",
                "fast",
                Some(2048),
                Some(4096),
                true,
                Some("Code completion model"),
            )
            .await
            .unwrap();

        let chat_model_id = conn
            .create_available_model(
                "gpt-4",
                "GPT-4",
                "chat",
                "openai",
                "quality",
                Some(4096),
                Some(8192),
                true,
                Some("Chat model"),
            )
            .await
            .unwrap();

        // 设置用户偏好
        let _pref_id = conn
            .upsert_user_model_preference(user_id, Some(completion_model_id), Some(chat_model_id))
            .await
            .unwrap();

        // 获取用户偏好
        let preference = conn
            .get_user_model_preference(user_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(preference.user_id, user_id);
        assert_eq!(preference.completion_model_id, Some(completion_model_id));
        assert_eq!(preference.chat_model_id, Some(chat_model_id));

        // 获取用户当前使用的模型
        let completion_model = conn.get_user_completion_model(user_id).await.unwrap().unwrap();
        assert_eq!(completion_model.name, "codegen");

        let chat_model = conn.get_user_chat_model(user_id).await.unwrap().unwrap();
        assert_eq!(chat_model.name, "gpt-4");
    }
}