use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("字体未找到: {0}")]
    FontNotFound(String),
    
    #[error("字符未找到: {0}")]
    CharacterNotFound(u32),
    
    #[error("配置错误: {0}")]
    ConfigError(String),
    
    #[error("字体处理错误: {0}")]
    FontProcessingError(String),
    
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("序列化错误: {0}")]
    SerdeError(#[from] serde_json::Error),
    
    #[error("内部错误: {0}")]
    InternalError(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::FontNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::CharacterNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::ConfigError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::FontProcessingError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::IoError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "文件系统错误".to_string()),
            AppError::SerdeError(_) => (StatusCode::BAD_REQUEST, "请求格式错误".to_string()),
            AppError::InternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "内部服务器错误".to_string()),
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}