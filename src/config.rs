use crate::models::{AIStudioCookieInfo, GeminiApiKeyInfo, Key, KeyStatus, KeyType};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;
use uuid::Uuid;

/// 配置加载错误
#[derive(Debug, Error)]
pub enum ConfigError {
    /// 配置文件不存在
    #[error("配置文件不存在: {0}")]
    FileNotFound(String),

    /// 读取配置文件错误
    #[error("读取配置文件错误: {0}")]
    ReadError(#[from] std::io::Error),

    /// 解析配置文件错误
    #[error("解析配置文件错误: {0}")]
    ParseError(#[from] toml::de::Error),
}

/// 合并角色选项
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MergeRole {
    /// 合并用户角色
    User,
    /// 合并模型角色
    Model,
    /// 不合并角色
    None,
}

impl Default for MergeRole {
    fn default() -> Self {
        Self::None
    }
}

/// 提示词处理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptConfig {
    /// 合并角色选项
    #[serde(default)]
    pub merge_role: MergeRole,

    /// 模型前缀
    #[serde(default = "default_model_prefix")]
    pub model_prefix: String,

    /// 用户前缀
    #[serde(default = "default_user_prefix")]
    pub user_prefix: String,

    /// 模型后缀
    #[serde(default = "default_model_suffix")]
    pub model_suffix: String,

    /// 用户后缀
    #[serde(default = "default_user_suffix")]
    pub user_suffix: String,
}

impl Default for PromptConfig {
    fn default() -> Self {
        Self {

            merge_role: MergeRole::default(),
            model_prefix: default_model_prefix(),
            user_prefix: default_user_prefix(),
            model_suffix: default_model_suffix(),
            user_suffix: default_user_suffix(),
        }
    }
}

/// 默认模型前缀
fn default_model_prefix() -> String {
    "Assistant: ".to_string()
}

/// 默认用户前缀
fn default_user_prefix() -> String {
    "Human: ".to_string()
}

/// 默认模型后缀
fn default_model_suffix() -> String {
    "\n".to_string()
}

/// 默认用户后缀
fn default_user_suffix() -> String {
    "\n".to_string()
}

/// TLS 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// 证书文件路径
    pub cert_path: String,

    /// 私钥文件路径
    pub key_path: String,
}

/// Gemini API Key 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiKeyConfig {
    /// API Key 值
    pub key_value: String,

    /// 是否启用
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// AI Studio Cookie 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIStudioCookieConfig {
    /// Cookie 值
    pub cookie_value: String,

    /// 是否启用
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// 凭证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum KeyConfig {
    /// Gemini API Key 配置
    #[serde(rename = "gemini")]
    Gemini(GeminiKeyConfig),

    /// AI Studio Cookie 配置
    #[serde(rename = "aistudio")]
    AIStudio(AIStudioCookieConfig),
}

impl KeyConfig {
    /// 转换为 Key
    pub fn to_key(&self) -> Key {
        match self {
            KeyConfig::Gemini(config) => Key::Gemini(GeminiApiKeyInfo {
                id: Uuid::new_v4(),
                key_value: config.key_value.clone(),
                last_used: None,
                status: if config.enabled {
                    KeyStatus::Available
                } else {
                    KeyStatus::Invalid
                },
            }),
            KeyConfig::AIStudio(config) => Key::AIStudio(AIStudioCookieInfo {
                id: Uuid::new_v4(),
                cookie_value: config.cookie_value.clone(),
                last_used: None,
                status: if config.enabled {
                    KeyStatus::Available
                } else {
                    KeyStatus::Invalid
                },
            }),
        }
    }

    /// 获取凭证类型
    pub fn get_type(&self) -> KeyType {
        match self {
            KeyConfig::Gemini(_) => KeyType::Gemini,
            KeyConfig::AIStudio(_) => KeyType::AIStudio,
        }
    }
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Gemini API 基础 URL
    #[serde(default = "default_gemini_url")]
    pub gemini_base_url: String,

    /// AI Studio 基础 URL
    #[serde(default = "default_aistudio_url")]
    pub aistudio_base_url: String,

    /// 管理 Key (用于访问代理服务)
    pub management_key: String,

    /// 监听地址
    #[serde(default = "default_listen_address")]
    pub listen_address: String,

    /// TLS 配置 (可选)
    pub tls: Option<TlsConfig>,

    /// 提示词处理配置
    #[serde(default)]
    pub prompt_config: PromptConfig,

    /// 凭证配置列表
    #[serde(default)]
    pub keys: Vec<KeyConfig>,

    /// 重试策略
    #[serde(default = "default_retry_limit")]
    pub retry_limit: u32,

    /// 冷却时间 (秒)
    #[serde(default = "default_cooldown_seconds")]
    pub cooldown_seconds: u64,
}

/// 加载配置文件
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config, ConfigError> {
    let path_ref = path.as_ref();
    if !path_ref.exists() {
        return Err(ConfigError::FileNotFound(
            path_ref.to_string_lossy().to_string(),
        ));
    }

    let content = fs::read_to_string(path_ref)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

/// 默认监听地址
fn default_listen_address() -> String {
    "127.0.0.1:3200".to_string()
}

/// 默认为 true
fn default_true() -> bool {
    true
}

/// 默认 Gemini URL
fn default_gemini_url() -> String {
    "https://generativelanguage.googleapis.com".to_string()
}

/// 默认 AI Studio URL
fn default_aistudio_url() -> String {
    "https://aistudio.google.com/app".to_string()
}

/// 默认重试次数
fn default_retry_limit() -> u32 {
    3
}

/// 默认冷却时间 (秒)
fn default_cooldown_seconds() -> u64 {
    60
}
