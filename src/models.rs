use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

/// 凭证类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyType {
    /// Google Gemini API Key
    Gemini,
    /// Google AI Studio Cookie
    AIStudio,
}

/// Gemini API Key 信息
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GeminiApiKeyInfo {
    /// 唯一标识符
    pub id: Uuid,
    /// API Key 值
    pub key_value: String,
    /// 最后使用时间
    #[serde(skip)]
    pub last_used: Option<Instant>,
    /// 凭证状态
    pub status: KeyStatus,
}

/// AI Studio Cookie 信息
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AIStudioCookieInfo {
    /// 唯一标识符
    pub id: Uuid,
    /// Cookie 值
    pub cookie_value: String,
    /// 最后使用时间
    #[serde(skip)]
    pub last_used: Option<Instant>,
    /// 凭证状态
    pub status: KeyStatus,
}

/// 访问凭证 (API Key 或 Cookie)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Key {
    /// Gemini API Key
    Gemini(GeminiApiKeyInfo),
    /// AI Studio Cookie
    AIStudio(AIStudioCookieInfo),
}

impl Key {
    /// 获取凭证类型
    pub fn get_type(&self) -> KeyType {
        match self {
            Key::Gemini(_) => KeyType::Gemini,
            Key::AIStudio(_) => KeyType::AIStudio,
        }
    }

    /// 获取凭证ID
    pub fn get_id(&self) -> Uuid {
        match self {
            Key::Gemini(info) => info.id,
            Key::AIStudio(info) => info.id,
        }
    }

    /// 获取凭证状态
    pub fn get_status(&self) -> KeyStatus {
        match self {
            Key::Gemini(info) => info.status,
            Key::AIStudio(info) => info.status,
        }
    }

    /// 设置凭证状态
    pub fn set_status(&mut self, status: KeyStatus) {
        match self {
            Key::Gemini(info) => info.status = status,
            Key::AIStudio(info) => info.status = status,
        }
    }
}

/// 凭证状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyStatus {
    /// 可用
    Available,
    /// 使用中
    InUse,
    /// 已失效
    Invalid,
    /// 冷却中
    CoolingDown,
}

/// 凭证回收原因
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    /// 成功完成请求
    Success,
    /// 超出速率限制
    RateLimited,
    /// 认证错误
    AuthError,
    /// 网络错误
    NetworkError,
    /// 未知错误
    Unknown,
}

/// 查询参数映射类型
pub type QueryParamsMap = HashMap<String, String>;

/// 处理器响应类型
pub type HandlerResponse = Result<reqwest::Response, HandlerError>;

/// 处理器错误类型
#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    /// 请求失败
    #[error("请求失败: {0}")]
    RequestFailed(String),

    /// 提示词处理失败
    #[error("提示词处理失败: {0}")]
    PromptProcessingFailed(String),

    /// 后端认证错误
    #[error("后端认证错误: {0}")]
    BackendAuthError(String),

    /// 后端速率限制
    #[error("后端速率限制: {0}")]
    BackendRateLimited(String),

    /// 后端服务错误
    #[error("后端服务错误: {0}")]
    BackendServiceError(String),
}

/// 分发请求
#[derive(Debug)]
pub struct DispatchRequest {
    /// 请求路径
    pub path: String,

    /// 查询参数
    pub query: QueryParamsMap,

    /// 请求体
    pub body: serde_json::Value,

    /// 目标凭证
    pub target_key: Key,

    /// 响应发送通道
    pub response_sender: oneshot::Sender<HandlerResponse>,

    /// 凭证返回通道
    pub key_return_tx: mpsc::Sender<(Key, Option<Reason>)>,
}
