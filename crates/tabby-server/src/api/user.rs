use axum::{Json, extract::{Query, State}, response::IntoResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, IntoParams};
use std::sync::Arc;
use tabby_db::DbConn;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};

/// 用户注册请求体
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    /// 用户名（可选）
    pub name: Option<String>,
    /// 邮箱（必填）
    pub email: String,
    /// 密码（必填）
    pub password: String,
}

/// 用户注册响应体
#[derive(Debug, Serialize, ToSchema)]
pub struct RegisterResponse {
    pub success: bool,
    pub user_id: i64,
    pub email: String,
    pub token: String,
    pub group: String,
    pub message: Option<String>,
}

/// 查询token请求参数
#[derive(Debug, Deserialize, IntoParams)]
pub struct QueryTokenRequest {
    /// 邮箱（可选）
    pub email: Option<String>,
    /// 用户名（可选）
    pub name: Option<String>,
}

/// 查询token响应体
#[derive(Debug, Serialize, ToSchema)]
pub struct QueryTokenResponse {
    pub success: bool,
    pub token: Option<String>,
    pub message: Option<String>,
}

/// 密码加密函数（使用argon2算法）
fn password_hash(raw: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    match argon2.hash_password(raw.as_bytes(), &salt) {
        Ok(hash) => Ok(hash.to_string()),
        Err(_) => Err("密码加密失败".to_string()),
    }
}

#[utoipa::path(
    post,
    path = "/v1/index/user/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "注册成功", body = RegisterResponse),
        (status = 400, description = "参数错误", body = RegisterResponse),
        (status = 409, description = "用户已存在", body = RegisterResponse),
        (status = 500, description = "服务器错误", body = RegisterResponse)
    ),
    tag = "User Management"
)]
pub async fn register_user(
    State(db): State<Arc<DbConn>>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    // 1. 参数验证
    if req.email.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(RegisterResponse {
                success: false,
                user_id: 0,
                email: req.email,
                token: "".to_string(),
                group: "".to_string(),
                message: Some("邮箱不能为空".to_string()),
            })
        );
    }

    if req.password.len() < 6 {
        return (
            StatusCode::BAD_REQUEST,
            Json(RegisterResponse {
                success: false,
                user_id: 0,
                email: req.email,
                token: "".to_string(),
                group: "".to_string(),
                message: Some("密码长度至少6位".to_string()),
            })
        );
    }

    // 2. 检查用户是否已存在
    match db.get_user_by_email(&req.email).await {
        Ok(Some(_)) => {
            return (
                StatusCode::CONFLICT,
                Json(RegisterResponse {
                    success: false,
                    user_id: 0,
                    email: req.email,
                    token: "".to_string(),
                    group: "".to_string(),
                    message: Some("用户已存在".to_string()),
                })
            );
        }
        Ok(None) => {
            // 用户不存在，可以注册
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterResponse {
                    success: false,
                    user_id: 0,
                    email: req.email,
                    token: "".to_string(),
                    group: "".to_string(),
                    message: Some(format!("数据库查询错误: {}", e)),
                })
            );
        }
    }

    // 3. 密码加密
    let password_encrypted = match password_hash(&req.password) {
        Ok(hash) => hash,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterResponse {
                    success: false,
                    user_id: 0,
                    email: req.email,
                    token: "".to_string(),
                    group: "".to_string(),
                    message: Some(e),
                })
            );
        }
    };

    // 4. 创建用户（默认为普通用户，非管理员）
    match db.create_user(
        req.email.clone(),
        Some(password_encrypted),
        false, // is_admin = false，普通用户
        req.name,
    ).await {
        Ok(user_id) => {
            // 5. 获取生成的用户信息（包含token）
            match db.get_user(user_id).await {
                Ok(Some(user)) => {
                    (
                        StatusCode::OK,
                        Json(RegisterResponse {
                            success: true,
                            user_id,
                            email: user.email,
                            token: user.auth_token,
                            group: "default".to_string(), // 默认用户组
                            message: Some("注册成功".to_string()),
                        })
                    )
                }
                Ok(None) => {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(RegisterResponse {
                            success: false,
                            user_id: 0,
                            email: req.email,
                            token: "".to_string(),
                            group: "".to_string(),
                            message: Some("用户创建后未能找到".to_string()),
                        })
                    )
                }
                Err(e) => {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(RegisterResponse {
                            success: false,
                            user_id: 0,
                            email: req.email,
                            token: "".to_string(),
                            group: "".to_string(),
                            message: Some(format!("获取用户信息错误: {}", e)),
                        })
                    )
                }
            }
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterResponse {
                    success: false,
                    user_id: 0,
                    email: req.email,
                    token: "".to_string(),
                    group: "".to_string(),
                    message: Some(format!("创建用户失败: {}", e)),
                })
            )
        }
    }
}

#[utoipa::path(
    get,
    path = "/v1/index/user/token",
    params(QueryTokenRequest),
    responses(
        (status = 200, description = "查询成功", body = QueryTokenResponse),
        (status = 400, description = "参数错误", body = QueryTokenResponse),
        (status = 404, description = "未找到用户", body = QueryTokenResponse),
        (status = 500, description = "服务器错误", body = QueryTokenResponse)
    ),
    tag = "User Management"
)]
pub async fn query_user_token(
    State(db): State<Arc<DbConn>>,
    Query(query): Query<QueryTokenRequest>,
) -> impl IntoResponse {
    // 1. 参数验证：邮箱和用户名至少要有一个
    if query.email.is_none() && query.name.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(QueryTokenResponse {
                success: false,
                token: None,
                message: Some("请提供邮箱或用户名".to_string()),
            })
        );
    }

    // 2. 根据邮箱查询用户
    if let Some(email) = query.email {
        match db.get_user_by_email(&email).await {
            Ok(Some(user)) => {
                return (
                    StatusCode::OK,
                    Json(QueryTokenResponse {
                        success: true,
                        token: Some(user.auth_token),
                        message: Some("查询成功".to_string()),
                    })
                );
            }
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(QueryTokenResponse {
                        success: false,
                        token: None,
                        message: Some("未找到该邮箱对应的用户".to_string()),
                    })
                );
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(QueryTokenResponse {
                        success: false,
                        token: None,
                        message: Some(format!("数据库查询错误: {}", e)),
                    })
                );
            }
        }
    }

    // 3. 根据用户名查询（需要遍历所有用户）
    if let Some(name) = query.name {
        // 由于数据库没有按name查询的直接方法，我们需要列出所有用户然后过滤
        match db.list_users_with_filter(None, None, None, false).await {
            Ok(users) => {
                for user in users {
                    if let Some(user_name) = &user.name {
                        if user_name == &name {
                            return (
                                StatusCode::OK,
                                Json(QueryTokenResponse {
                                    success: true,
                                    token: Some(user.auth_token),
                                    message: Some("查询成功".to_string()),
                                })
                            );
                        }
                    }
                }

                (
                    StatusCode::NOT_FOUND,
                    Json(QueryTokenResponse {
                        success: false,
                        token: None,
                        message: Some("未找到该用户名对应的用户".to_string()),
                    })
                )
            }
            Err(e) => {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(QueryTokenResponse {
                        success: false,
                        token: None,
                        message: Some(format!("数据库查询错误: {}", e)),
                    })
                )
            }
        }
    } else {
        // 理论上不会到这里，因为前面已经检查过了
        (
            StatusCode::BAD_REQUEST,
            Json(QueryTokenResponse {
                success: false,
                token: None,
                message: Some("请提供邮箱或用户名".to_string()),
            })
        )
    }
}