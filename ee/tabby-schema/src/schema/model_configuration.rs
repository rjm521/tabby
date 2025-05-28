use chrono::{DateTime, Utc};
use juniper::{GraphQLEnum, GraphQLInputObject, GraphQLObject, ID as JuniperID};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use validator::Validate;
use anyhow;

use crate::schema::{Context, Result};

/// 模型类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, GraphQLEnum)]
pub enum ModelType {
    /// 代码补全模型
    #[graphql(name = "completion")]
    Completion,
    /// 聊天模型
    #[graphql(name = "chat")]
    Chat,
}

/// 性能级别枚举
#[derive(Debug, Clone, Serialize, Deserialize, GraphQLEnum)]
pub enum PerformanceTier {
    /// 快速模型（低延迟）
    #[graphql(name = "fast")]
    Fast,
    /// 平衡模型（中等延迟和质量）
    #[graphql(name = "balanced")]
    Balanced,
    /// 高质量模型（高延迟但质量好）
    #[graphql(name = "quality")]
    Quality,
}

/// 可用模型数据结构
#[derive(Debug, Clone, GraphQLObject)]
#[graphql(context = Context)]
pub struct AvailableModel {
    pub id: JuniperID,
    pub name: String,
    pub display_name: String,
    pub model_type: ModelType,
    pub provider: String,
    pub performance_tier: PerformanceTier,
    pub max_tokens: Option<i32>,
    pub context_window: Option<i32>,
    pub enabled: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 用户模型偏好设置
#[derive(Debug, Clone, GraphQLObject)]
#[graphql(context = Context)]
pub struct UserModelPreference {
    pub id: JuniperID,
    pub user_id: JuniperID,
    pub completion_model_id: Option<JuniperID>,
    pub chat_model_id: Option<JuniperID>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 更新用户模型偏好的输入参数
#[derive(Debug, GraphQLInputObject, Validate)]
pub struct UpdateUserModelPreferenceInput {
    /// 代码补全模型ID
    pub completion_model_id: Option<JuniperID>,
    /// 聊天模型ID
    pub chat_model_id: Option<JuniperID>,
}

/// 创建可用模型的输入参数（管理员使用）
#[derive(Debug, GraphQLInputObject, Validate)]
pub struct CreateAvailableModelInput {
    #[validate(length(min = 1, max = 100, code = "name", message = "模型名称长度必须在1-100字符之间"))]
    pub name: String,

    #[validate(length(min = 1, max = 100, code = "display_name", message = "显示名称长度必须在1-100字符之间"))]
    pub display_name: String,

    pub model_type: ModelType,

    #[validate(length(min = 1, max = 50, code = "provider", message = "提供商名称长度必须在1-50字符之间"))]
    pub provider: String,

    pub performance_tier: PerformanceTier,
    pub max_tokens: Option<i32>,
    pub context_window: Option<i32>,
    pub enabled: Option<bool>,

    #[validate(length(max = 500, code = "description", message = "描述长度不能超过500字符"))]
    pub description: Option<String>,
}

/// 更新可用模型的输入参数（管理员使用）
#[derive(Debug, GraphQLInputObject, Validate)]
pub struct UpdateAvailableModelInput {
    #[validate(length(min = 1, max = 100, code = "display_name", message = "显示名称长度必须在1-100字符之间"))]
    pub display_name: Option<String>,

    pub performance_tier: Option<PerformanceTier>,
    pub max_tokens: Option<i32>,
    pub context_window: Option<i32>,
    pub enabled: Option<bool>,

    #[validate(length(max = 500, code = "description", message = "描述长度不能超过500字符"))]
    pub description: Option<String>,
}

/// 模型配置服务 trait
#[async_trait]
pub trait ModelConfigurationService: Send + Sync {
    /// 获取用户的模型偏好设置
    async fn get_user_model_preference(&self, user_id: &JuniperID) -> Result<Option<UserModelPreference>>;

    /// 更新用户的模型偏好设置
    async fn update_user_model_preference(
        &self,
        user_id: &JuniperID,
        input: UpdateUserModelPreferenceInput,
    ) -> Result<UserModelPreference>;

    /// 获取所有可用模型列表
    async fn list_available_models(&self, model_type: Option<ModelType>) -> Result<Vec<AvailableModel>>;

    /// 根据ID获取可用模型
    async fn get_available_model(&self, id: &JuniperID) -> Result<Option<AvailableModel>>;

    /// 创建可用模型（管理员功能）
    async fn create_available_model(&self, input: CreateAvailableModelInput) -> Result<AvailableModel>;

    /// 更新可用模型（管理员功能）
    async fn update_available_model(
        &self,
        id: &JuniperID,
        input: UpdateAvailableModelInput,
    ) -> Result<AvailableModel>;

    /// 删除可用模型（管理员功能）
    async fn delete_available_model(&self, id: &JuniperID) -> Result<JuniperID>;

    /// 获取用户当前使用的模型（用于代码补全）
    async fn get_user_completion_model(&self, user_id: &JuniperID) -> Result<Option<AvailableModel>>;

    /// 获取用户当前使用的模型（用于聊天）
    async fn get_user_chat_model(&self, user_id: &JuniperID) -> Result<Option<AvailableModel>>;
}

// GraphQL 查询解析器函数
pub async fn user_model_preferences(ctx: &Context) -> Result<Option<UserModelPreference>> {
    use crate::schema::check_user;

    let user = check_user(ctx).await?;
    ctx.locator.model_configuration().get_user_model_preference(&user.id).await
}

pub async fn available_models(ctx: &Context, model_type: Option<ModelType>) -> Result<Vec<AvailableModel>> {
    // 可用模型列表不需要特殊权限检查，任何已登录用户都可以查看
    ctx.locator.model_configuration().list_available_models(model_type).await
}

// GraphQL 变更解析器函数
pub async fn update_user_model_preferences(ctx: &Context, input: UpdateUserModelPreferenceInput) -> Result<UserModelPreference> {
    use crate::schema::check_user;

    let user = check_user(ctx).await?;
    ctx.locator.model_configuration().update_user_model_preference(&user.id, input).await
}

pub async fn reset_user_model_preferences(ctx: &Context) -> Result<UserModelPreference> {
    use crate::schema::check_user;

    let user = check_user(ctx).await?;
    // 重置用户模型偏好 - 将 completion_model_id 和 chat_model_id 都设为 None
    let reset_input = UpdateUserModelPreferenceInput {
        completion_model_id: None,
        chat_model_id: None,
    };
    ctx.locator.model_configuration().update_user_model_preference(&user.id, reset_input).await
}

// 实现 DbEnum trait，用于数据库枚举映射
impl crate::dao::DbEnum for ModelType {
    fn as_enum_str(&self) -> &'static str {
        match self {
            ModelType::Completion => "completion",
            ModelType::Chat => "chat",
        }
    }

    fn from_enum_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "completion" => Ok(ModelType::Completion),
            "chat" => Ok(ModelType::Chat),
            _ => anyhow::bail!("Invalid ModelType: {}", s),
        }
    }
}

impl crate::dao::DbEnum for PerformanceTier {
    fn as_enum_str(&self) -> &'static str {
        match self {
            PerformanceTier::Fast => "fast",
            PerformanceTier::Balanced => "balanced",
            PerformanceTier::Quality => "quality",
        }
    }

    fn from_enum_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "fast" => Ok(PerformanceTier::Fast),
            "balanced" => Ok(PerformanceTier::Balanced),
            "quality" => Ok(PerformanceTier::Quality),
            _ => anyhow::bail!("Invalid PerformanceTier: {}", s),
        }
    }
}