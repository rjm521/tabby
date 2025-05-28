use std::sync::Arc;

use async_trait::async_trait;
use juniper::ID as JuniperID;
use tabby_db::{DbConn, AvailableModelDAO, UserModelPreferenceDAO};
use tabby_schema::{
    AsID, AsRowid, DbEnum,
    model_configuration::{
        AvailableModel, CreateAvailableModelInput, ModelConfigurationService, ModelType,
        PerformanceTier, UpdateAvailableModelInput, UpdateUserModelPreferenceInput,
        UserModelPreference,
    },
    Result as SchemaResult,
};

pub struct ModelConfigurationServiceImpl {
    db: Arc<DbConn>,
}

impl ModelConfigurationServiceImpl {
    pub fn new(db: Arc<DbConn>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ModelConfigurationService for ModelConfigurationServiceImpl {
    async fn get_user_model_preference(&self, user_id: &JuniperID) -> SchemaResult<Option<UserModelPreference>> {
        let user_rowid = user_id.as_rowid()?;
        let preference = self.db.get_user_model_preference(user_rowid).await?;
        Ok(preference.map(|p| convert_user_preference(p)))
    }

    async fn update_user_model_preference(
        &self,
        user_id: &JuniperID,
        input: UpdateUserModelPreferenceInput,
    ) -> SchemaResult<UserModelPreference> {
        let user_rowid = user_id.as_rowid()?;

        let completion_model_id = if let Some(id) = input.completion_model_id {
            Some(id.as_rowid()?)
        } else {
            None
        };

        let chat_model_id = if let Some(id) = input.chat_model_id {
            Some(id.as_rowid()?)
        } else {
            None
        };

        let _preference_id = self
            .db
            .upsert_user_model_preference(user_rowid, completion_model_id, chat_model_id)
            .await?;

        let preference = self
            .db
            .get_user_model_preference(user_rowid)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve updated preference"))?;

        Ok(convert_user_preference(preference))
    }

    async fn list_available_models(&self, model_type: Option<ModelType>) -> SchemaResult<Vec<AvailableModel>> {
        let model_type_str = model_type.map(|t| t.as_enum_str());
        let models = self.db.list_available_models(model_type_str).await?;
        Ok(models.into_iter().map(|m| convert_available_model(m)).collect())
    }

    async fn get_available_model(&self, id: &JuniperID) -> SchemaResult<Option<AvailableModel>> {
        let rowid = id.as_rowid()?;
        let model = self.db.get_available_model(rowid).await?;
        Ok(model.map(|m| convert_available_model(m)))
    }

    async fn create_available_model(&self, input: CreateAvailableModelInput) -> SchemaResult<AvailableModel> {
        let model_id = self
            .db
            .create_available_model(
                &input.name,
                &input.display_name,
                input.model_type.as_enum_str(),
                &input.provider,
                input.performance_tier.as_enum_str(),
                input.max_tokens,
                input.context_window,
                input.enabled.unwrap_or(true),
                input.description.as_deref(),
            )
            .await?;

        let model = self
            .db
            .get_available_model(model_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve created model"))?;

        Ok(convert_available_model(model))
    }

    async fn update_available_model(
        &self,
        id: &JuniperID,
        input: UpdateAvailableModelInput,
    ) -> SchemaResult<AvailableModel> {
        let rowid = id.as_rowid()?;

        self.db
            .update_available_model(
                rowid,
                input.display_name.as_deref(),
                input.performance_tier.map(|t| t.as_enum_str()),
                input.max_tokens,
                input.context_window,
                input.enabled,
                input.description.as_deref(),
            )
            .await?;

        let model = self
            .db
            .get_available_model(rowid)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Model not found"))?;

        Ok(convert_available_model(model))
    }

    async fn delete_available_model(&self, id: &JuniperID) -> SchemaResult<JuniperID> {
        let rowid = id.as_rowid()?;
        self.db.delete_available_model(rowid).await?;
        Ok(id.clone())
    }

    async fn get_user_completion_model(&self, user_id: &JuniperID) -> SchemaResult<Option<AvailableModel>> {
        let user_rowid = user_id.as_rowid()?;
        let model = self.db.get_user_completion_model(user_rowid).await?;
        Ok(model.map(|m| convert_available_model(m)))
    }

    async fn get_user_chat_model(&self, user_id: &JuniperID) -> SchemaResult<Option<AvailableModel>> {
        let user_rowid = user_id.as_rowid()?;
        let model = self.db.get_user_chat_model(user_rowid).await?;
        Ok(model.map(|m| convert_available_model(m)))
    }
}

// 数据转换函数 - 避免孤儿规则问题
fn convert_available_model(dao: AvailableModelDAO) -> AvailableModel {
    AvailableModel {
        id: dao.id.as_id(),
        name: dao.name,
        display_name: dao.display_name,
        model_type: ModelType::from_enum_str(&dao.model_type).unwrap_or(ModelType::Completion),
        provider: dao.provider,
        performance_tier: PerformanceTier::from_enum_str(&dao.performance_tier)
            .unwrap_or(PerformanceTier::Balanced),
        max_tokens: dao.max_tokens,
        context_window: dao.context_window,
        enabled: dao.enabled,
        description: dao.description,
        created_at: dao.created_at,
        updated_at: dao.updated_at,
    }
}

fn convert_user_preference(dao: UserModelPreferenceDAO) -> UserModelPreference {
    UserModelPreference {
        id: dao.id.as_id(),
        user_id: dao.user_id.as_id(),
        completion_model_id: dao.completion_model_id.map(|id| id.as_id()),
        chat_model_id: dao.chat_model_id.map(|id| id.as_id()),
        created_at: dao.created_at,
        updated_at: dao.updated_at,
    }
}

/// 创建模型配置服务实例
pub fn create(db: Arc<DbConn>) -> Arc<dyn ModelConfigurationService> {
    Arc::new(ModelConfigurationServiceImpl::new(db))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tabby_db::DbConn;
    use std::sync::Arc;

    async fn new_service() -> Arc<dyn ModelConfigurationService> {
        let db = DbConn::new_in_memory().await.unwrap();
        let migrator = sqlx::migrate!("../../../../ee/tabby-db/migrations");
        migrator.run(db.as_ref()).await.unwrap();
        create(db)
    }

    #[tokio::test]
    async fn test_list_available_models() {
        let service = new_service().await;

        // Test listing all models
        let all_models = service.list_available_models(None).await.unwrap();
        assert!(!all_models.is_empty(), "Should return some available models");
        assert_eq!(all_models.len(), 6, "Expected 6 pre-seeded models");

        // Test filtering by completion models
        let completion_models = service.list_available_models(Some(ModelType::Completion)).await.unwrap();
        assert_eq!(completion_models.len(), 3, "Expected 3 completion models");
        for model in completion_models {
            assert_eq!(model.model_type, ModelType::Completion);
        }

        // Test filtering by chat models
        let chat_models = service.list_available_models(Some(ModelType::Chat)).await.unwrap();
        assert_eq!(chat_models.len(), 3, "Expected 3 chat models");
        for model in chat_models {
            assert_eq!(model.model_type, ModelType::Chat);
        }
    }

    #[tokio::test]
    async fn test_get_and_update_user_preferences() {
        let service = new_service().await;
        let user_id = 1i64.as_id();

        // Create a test user first
        let db = DbConn::new_in_memory().await.unwrap();
        let migrator = sqlx::migrate!("../../../../ee/tabby-db/migrations");
        migrator.run(db.as_ref()).await.unwrap();

        sqlx::query("INSERT INTO users (id, email, auth_token) VALUES (?1, 'testuser@example.com', 'testtoken')")
            .bind(1i64)
            .execute(db.as_ref()).await.unwrap();

        let service = create(db);

        // 1. Initially, no preferences
        let prefs = service.get_user_model_preference(&user_id).await.unwrap();
        assert!(prefs.is_none());

        // 2. Set completion model
        let input1 = UpdateUserModelPreferenceInput {
            completion_model_id: Some(1i64.as_id()),
            chat_model_id: None,
        };
        let updated_prefs1 = service.update_user_model_preference(&user_id, input1).await.unwrap();
        assert_eq!(updated_prefs1.user_id, user_id);
        assert_eq!(updated_prefs1.completion_model_id.as_ref().and_then(|id| id.as_rowid().ok()), Some(1i64));
        assert!(updated_prefs1.chat_model_id.is_none());

        // 3. Get and verify
        let current_prefs1 = service.get_user_model_preference(&user_id).await.unwrap().unwrap();
        assert_eq!(current_prefs1.completion_model_id.as_ref().and_then(|id| id.as_rowid().ok()), Some(1i64));

        // 4. Update chat model and change completion model
        let input2 = UpdateUserModelPreferenceInput {
            completion_model_id: Some(2i64.as_id()),
            chat_model_id: Some(3i64.as_id()),
        };
        let updated_prefs2 = service.update_user_model_preference(&user_id, input2).await.unwrap();
        assert_eq!(updated_prefs2.completion_model_id.as_ref().and_then(|id| id.as_rowid().ok()), Some(2i64));
        assert_eq!(updated_prefs2.chat_model_id.as_ref().and_then(|id| id.as_rowid().ok()), Some(3i64));
    }
}