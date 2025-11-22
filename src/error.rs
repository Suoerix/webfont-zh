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
        let (status, error_message, error_code) = match &self {
            AppError::FontNotFound(font_id) => (
                StatusCode::NOT_FOUND,
                format!("字体未找到: {}", font_id),
                "FONT_NOT_FOUND",
            ),
            AppError::CharacterNotFound(codepoint) => {
                let char_info = char::from_u32(*codepoint)
                    .map(|c| format!("U+{:04X} ({})", codepoint, c))
                    .unwrap_or_else(|| format!("U+{:04X}", codepoint));
                (
                    StatusCode::NOT_FOUND,
                    format!("所有字体都不包含请求的字符: {}", char_info),
                    "CHARACTER_NOT_FOUND",
                )
            }
            AppError::ConfigError(_) => (StatusCode::BAD_REQUEST, self.to_string(), "CONFIG_ERROR"),
            AppError::FontProcessingError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                self.to_string(),
                "FONT_PROCESSING_ERROR",
            ),
            AppError::IoError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "文件系统错误".to_string(),
                "IO_ERROR",
            ),
            AppError::SerdeError(_) => (
                StatusCode::BAD_REQUEST,
                "请求格式错误".to_string(),
                "SERDE_ERROR",
            ),
            AppError::InternalError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "内部服务器错误".to_string(),
                "INTERNAL_ERROR",
            ),
        };

        let body = Json(json!({
            "error": error_message,
            "code": error_code
        }));

        (status, body).into_response()
    }
}
