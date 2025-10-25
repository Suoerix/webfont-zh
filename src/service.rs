use crate::{
    config::{AppConfig, FontConfig},
    error::AppError,
    font::FontProcessor,
    utils::{generate_cache_filename, cleanup_expired_cache},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,

    sync::Arc,
};
use tokio::sync::RwLock;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontInfo {
    pub id: String,
    pub version: String,
    pub font_family: String,
    pub license: String,
    pub fallback: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<crate::config::LocalizedText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<crate::config::LocalizedText>,
}

pub struct FontService {
    config: AppConfig,
    fonts: Arc<RwLock<HashMap<String, FontConfig>>>,
    processors: Arc<RwLock<HashMap<String, Arc<FontProcessor>>>>,
}

impl FontService {
    pub async fn new(config: AppConfig) -> Result<Self> {
        let service = Self {
            config,
            fonts: Arc::new(RwLock::new(HashMap::new())),
            processors: Arc::new(RwLock::new(HashMap::new())),
        };
        
        service.load_fonts().await?;
        service.start_cleanup_task();
        
        Ok(service)
    }
    
    /// 加载所有字体配置
    async fn load_fonts(&self) -> Result<()> {
        let fonts_dir = self.config.data_dir.join("fonts");
        let mut fonts = self.fonts.write().await;
        let mut processors = self.processors.write().await;
        
        for entry in WalkDir::new(&fonts_dir)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
        {
            let font_dir = entry.path();
            
            match FontConfig::load_from_dir(&font_dir.to_path_buf()) {
                Ok(font_config) => {
                    log::info!("加载字体配置: {}", font_config.id);
                    
                    // 为每个字体文件创建处理器
                    for font_file in &font_config.files {
                        let font_path = font_dir.join(&font_file.path);
                        if font_path.exists() {
                            match FontProcessor::new(&font_path) {
                                Ok(processor) => {
                                    let key = format!("{}:{}", font_config.id, font_file.font_family);
                                    processors.insert(key, Arc::new(processor));
                                    log::info!("加载字体处理器: {} - {}", font_config.id, font_file.font_family);
                                }
                                Err(e) => {
                                    log::error!("加载字体处理器失败 {}: {}", font_path.display(), e);
                                }
                            }
                        } else {
                            log::error!("字体文件不存在: {}", font_path.display());
                        }
                    }
                    
                    fonts.insert(font_config.id.clone(), font_config);
                }
                Err(e) => {
                    log::error!("加载字体配置失败 {}: {}", font_dir.display(), e);
                }
            }
        }
        
        log::info!("共加载 {} 个字体配置", fonts.len());
        Ok(())
    }
    
    /// 获取所有字体信息
    pub async fn list_fonts(&self) -> Vec<FontInfo> {
        let fonts = self.fonts.read().await;
        fonts
            .values()
            .map(|config| FontInfo {
                id: config.id.clone(),
                version: config.version.clone(),
                font_family: config.font_family.clone(),
                license: config.license.clone(),
                fallback: config.fallback.clone(),
                name: config.name.clone(),
                title: config.title.clone(),
            })
            .collect()
    }
    
    /// 生成字体WOFF2文件
    pub async fn generate_font(&self, font_id: Option<&str>, codepoints: &[u32]) -> Result<Vec<u8>, AppError> {
        if codepoints.is_empty() {
            return Err(AppError::CharacterNotFound(0));
        }
        
        // 如果指定了字体ID，直接使用该字体
        if let Some(id) = font_id {
            return self.generate_font_by_id(id, codepoints).await;
        }
        
        // 否则尝试所有字体，使用第一个包含字符的字体
        let fonts = self.fonts.read().await;
        for font_config in fonts.values() {
            if let Ok(woff2_data) = self.generate_font_by_id(&font_config.id, codepoints).await {
                return Ok(woff2_data);
            }
        }
        
        Err(AppError::CharacterNotFound(codepoints[0]))
    }
    
    /// 根据字体ID生成WOFF2文件
    async fn generate_font_by_id(&self, font_id: &str, codepoints: &[u32]) -> Result<Vec<u8>, AppError> {
        let fonts = self.fonts.read().await;
        let font_config = fonts
            .get(font_id)
            .ok_or_else(|| AppError::FontNotFound(font_id.to_string()))?;
        
        // 尝试每个字体文件，直到找到包含字符的文件
        let processors = self.processors.read().await;
        for font_file in &font_config.files {
            let key = format!("{}:{}", font_id, font_file.font_family);
            if let Some(processor) = processors.get(&key) {
                let available_chars = processor.get_available_chars(codepoints);
                if !available_chars.is_empty() {
                    match processor.generate_woff2(&available_chars) {
                        Ok(woff2_data) => return Ok(woff2_data),
                        Err(e) => log::warn!("生成WOFF2失败 {}: {}", key, e),
                    }
                }
            }
        }
        
        // 如果当前字体不包含字符，尝试fallback字体
        for fallback_id in &font_config.fallback {
            let fallback_result = Box::pin(self.generate_font_by_id(fallback_id, codepoints)).await;
            if let Ok(woff2_data) = fallback_result {
                return Ok(woff2_data);
            }
        }
        
        Err(AppError::CharacterNotFound(codepoints[0]))
    }
    
    /// 获取或生成缓存的字体文件
    pub async fn get_cached_font(&self, font_id: &str, codepoints: &[u32]) -> Result<Vec<u8>, AppError> {
        let cache_filename = generate_cache_filename(codepoints);
        let cache_path = self.config.static_dir.join(font_id).join(&cache_filename);
        
        // 检查缓存是否存在
        if cache_path.exists() {
            match tokio::fs::read(&cache_path).await {
                Ok(data) => {
                    log::debug!("使用缓存文件: {:?}", cache_path);
                    return Ok(data);
                }
                Err(e) => log::warn!("读取缓存文件失败 {:?}: {}", cache_path, e),
            }
        }
        
        // 生成新的字体文件
        let woff2_data = self.generate_font(Some(font_id), codepoints).await?;
        
        // 保存到缓存
        if let Some(parent) = cache_path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                log::warn!("创建缓存目录失败 {:?}: {}", parent, e);
            }
        }
        
        if let Err(e) = tokio::fs::write(&cache_path, &woff2_data).await {
            log::warn!("保存缓存文件失败 {:?}: {}", cache_path, e);
        } else {
            log::info!("保存缓存文件: {:?}", cache_path);
        }
        
        Ok(woff2_data)
    }
    
    /// 强制重新生成字体文件并缓存
    pub async fn regenerate_font(&self, font_id: Option<&str>, codepoints: &[u32]) -> Result<(), AppError> {
        if let Some(id) = font_id {
            // 为单个字符生成缓存
            for &codepoint in codepoints {
                let woff2_data = self.generate_font(Some(id), &[codepoint]).await?;
                let cache_filename = generate_cache_filename(&[codepoint]);
                let cache_path = self.config.static_dir.join(id).join(&cache_filename);
                
                if let Some(parent) = cache_path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                
                tokio::fs::write(&cache_path, &woff2_data).await?;
                log::info!("重新生成缓存文件: {:?}", cache_path);
            }
        } else {
            // 为所有字体生成缓存
            let fonts = self.fonts.read().await;
            for font_id in fonts.keys() {
                let result = Box::pin(self.regenerate_font(Some(font_id), codepoints)).await;
                if let Err(e) = result {
                    log::warn!("重新生成字体缓存失败 {}: {}", font_id, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// 启动定期清理任务
    fn start_cleanup_task(&self) {
        let static_dir = self.config.static_dir.clone();
        let cleanup_days = self.config.cache_cleanup_days;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(24 * 3600)); // 每天执行一次
            
            loop {
                interval.tick().await;
                
                log::info!("开始清理过期缓存文件");
                
                // 清理每个字体目录下的cache文件夹
                if let Ok(entries) = tokio::fs::read_dir(&static_dir).await {
                    let mut entries = entries;
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        let path = entry.path();
                        if path.is_dir() {
                            let cache_dir = path.join("cache");
                            if cache_dir.exists() {
                                match cleanup_expired_cache(&cache_dir, cleanup_days) {
                                    Ok(count) => {
                                        if count > 0 {
                                            log::info!("清理了 {} 个过期缓存文件: {:?}", count, cache_dir);
                                        }
                                    }
                                    Err(e) => log::error!("清理缓存失败 {:?}: {}", cache_dir, e),
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}