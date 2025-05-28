-- 删除已存在的表（如果有的话）
DROP TABLE IF EXISTS available_models;

-- 重新创建 available_models 表，使用正确的字段名称
CREATE TABLE available_models (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,                      -- 模型名称（唯一）
    display_name TEXT NOT NULL,                     -- 显示名称
    model_type TEXT NOT NULL CHECK (model_type IN ('completion', 'chat')), -- 模型类型
    provider TEXT NOT NULL,                         -- 提供商
    performance_tier TEXT NOT NULL CHECK (performance_tier IN ('fast', 'balanced', 'quality')), -- 性能等级
    max_tokens INTEGER,                             -- 最大token数
    context_window INTEGER,                         -- 上下文窗口大小
    enabled BOOLEAN NOT NULL DEFAULT TRUE,          -- 是否启用
    description TEXT,                               -- 描述
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建 updated_at 触发器，用于自动更新 available_models 表的 updated_at 字段
CREATE TRIGGER trigger_available_models_updated_at
AFTER UPDATE ON available_models
FOR EACH ROW
BEGIN
    UPDATE available_models
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = OLD.id;
END;

-- 修复 user_model_preferences 表：删除并重新创建
DROP TABLE IF EXISTS user_model_preferences;

CREATE TABLE user_model_preferences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,                       -- 与 users.id 类型一致
    completion_model_id INTEGER,                    -- 代码补全模型ID（外键）
    chat_model_id INTEGER,                          -- 聊天模型ID（外键）
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (completion_model_id) REFERENCES available_models(id) ON DELETE SET NULL,
    FOREIGN KEY (chat_model_id) REFERENCES available_models(id) ON DELETE SET NULL
);

-- 创建 updated_at 触发器，用于自动更新 user_model_preferences 表的 updated_at 字段
CREATE TRIGGER trigger_user_model_preferences_updated_at_v2
AFTER UPDATE ON user_model_preferences
FOR EACH ROW
BEGIN
    UPDATE user_model_preferences
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = OLD.id;
END;

-- 初始可用模型数据
INSERT INTO available_models (name, display_name, model_type, provider, performance_tier, max_tokens, context_window, enabled, description) VALUES
('starcoder-1b', 'StarCoder 1B', 'completion', 'HuggingFace', 'fast', 2048, 8192, TRUE, '轻量级代码补全模型，适用于快速响应场景'),
('starcoder-7b', 'StarCoder 7B', 'completion', 'HuggingFace', 'quality', 2048, 8192, TRUE, '高质量代码补全模型，准确性更高'),
('deepseek-coder-1.3b', 'DeepSeek Coder 1.3B', 'completion', 'DeepSeek', 'balanced', 2048, 16384, TRUE, '平衡性能和质量的代码补全模型'),
('qwen2-1.5b-instruct', 'Qwen2 1.5B Instruct', 'chat', 'Alibaba', 'fast', 2048, 32768, TRUE, '轻量级聊天对话模型'),
('qwen2-7b-instruct', 'Qwen2 7B Instruct', 'chat', 'Alibaba', 'quality', 4096, 32768, TRUE, '高质量聊天对话模型'),
('codellama-7b-instruct', 'CodeLlama 7B Instruct', 'chat', 'Meta', 'quality', 4096, 16384, TRUE, '专业代码聊天模型');

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_user_model_preferences_user_id ON user_model_preferences(user_id);
CREATE INDEX IF NOT EXISTS idx_available_models_model_type ON available_models(model_type);
CREATE INDEX IF NOT EXISTS idx_available_models_enabled ON available_models(enabled);
CREATE INDEX IF NOT EXISTS idx_available_models_name ON available_models(name);