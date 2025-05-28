-- 删除触发器
DROP TRIGGER IF EXISTS trigger_available_models_updated_at;
DROP TRIGGER IF EXISTS trigger_user_model_preferences_updated_at_v2;

-- 删除索引
DROP INDEX IF EXISTS idx_user_model_preferences_user_id;
DROP INDEX IF EXISTS idx_available_models_model_type;
DROP INDEX IF EXISTS idx_available_models_enabled;
DROP INDEX IF EXISTS idx_available_models_name;

-- 删除表
DROP TABLE IF EXISTS user_model_preferences;
DROP TABLE IF EXISTS available_models;