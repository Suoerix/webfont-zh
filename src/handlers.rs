use crate::{error::AppError, service::FontInfo, utils::parse_codepoints, AppState};
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;


#[derive(Deserialize)]
pub struct FontQuery {
    pub id: String,
    #[serde(rename = "char")]
    pub chars: String,
}

#[derive(Deserialize)]
pub struct GenerateQuery {
    pub id: Option<String>,
    #[serde(rename = "char")]
    pub chars: String,
}

/// GET /api/v1/list - 列出所有可用字体
pub async fn list_fonts(State(service): State<AppState>) -> Result<Json<Vec<FontInfo>>, AppError> {
    let fonts = service.list_fonts().await;
    Ok(Json(fonts))
}

/// GET /api/v1/font - 获取字体文件
pub async fn get_font(
    Query(params): Query<FontQuery>,
    State(service): State<AppState>,
) -> Result<Response, AppError> {
    let codepoints = parse_codepoints(&params.chars)
        .map_err(|_| AppError::ConfigError("无效的字符码点格式".to_string()))?;
    
    if codepoints.is_empty() {
        return Err(AppError::ConfigError("字符码点不能为空".to_string()));
    }
    
    let woff2_data = service.get_cached_font(&params.id, &codepoints).await?;
    
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/font-woff2".parse().unwrap());
    headers.insert(
        header::CACHE_CONTROL,
        "public, max-age=31536000, immutable".parse().unwrap(),
    );
    
    Ok((headers, woff2_data).into_response())
}

/// POST /api/v1/generate - 重新生成字体文件
pub async fn generate_font(
    Query(params): Query<GenerateQuery>,
    State(service): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let codepoints = parse_codepoints(&params.chars)
        .map_err(|_| AppError::ConfigError("无效的字符码点格式".to_string()))?;
    
    if codepoints.is_empty() {
        return Err(AppError::ConfigError("字符码点不能为空".to_string()));
    }
    
    service
        .regenerate_font(params.id.as_deref(), &codepoints)
        .await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "字体文件已重新生成",
        "font_id": params.id,
        "characters": codepoints.len()
    })))
}