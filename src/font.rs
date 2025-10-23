use anyhow::{anyhow, Result};
use harfbuzz_rs_now::{Face, Owned};
use harfbuzz_rs_now::subset::Subset;
use std::path::Path;

/// 字体处理器，负责字体分包和woff2生成
pub struct FontProcessor {
    font_data: Vec<u8>,
    font_face: ttf_parser::Face<'static>,
    harfbuzz_face: Owned<Face<'static>>,
}

impl FontProcessor {
    pub fn new(font_path: &Path) -> Result<Self> {
        let font_data = std::fs::read(font_path)?;
        
        // 使用 Box::leak 来获得 'static 生命周期
        let static_data: &'static [u8] = Box::leak(font_data.clone().into_boxed_slice());
        
        let font_face = ttf_parser::Face::parse(static_data, 0)
            .map_err(|e| anyhow!("解析字体失败: {:?}", e))?;
        
        // 创建HarfBuzz Face用于字体子集化，使用static_data避免生命周期问题
        let harfbuzz_face = Face::from_bytes(static_data, 0);
            
        Ok(Self {
            font_data,
            font_face,
            harfbuzz_face,
        })
    }
    
    /// 检查字体是否包含指定字符
    pub fn contains_char(&self, codepoint: u32) -> bool {
        if let Some(ch) = char::from_u32(codepoint) {
            self.font_face.glyph_index(ch).is_some()
        } else {
            false
        }
    }
    
    /// 获取字体中包含的字符集合
    pub fn get_available_chars(&self, codepoints: &[u32]) -> Vec<u32> {
        codepoints
            .iter()
            .filter(|&&cp| self.contains_char(cp))
            .copied()
            .collect()
    }
    
    /// 生成包含指定字符的子集字体
    pub fn subset_font(&self, codepoints: &[u32]) -> Result<Vec<u8>> {
        // 过滤出字体实际包含的字符
        let available_chars: Vec<char> = codepoints
            .iter()
            .filter_map(|&cp| {
                if self.contains_char(cp) {
                    char::from_u32(cp)
                } else {
                    None
                }
            })
            .collect();
            
        if available_chars.is_empty() {
            return Err(anyhow!("字体不包含任何请求的字符"));
        }
        
        // 使用harfbuzz进行字体子集化
        self.create_subset(&available_chars)
    }
    
    fn create_subset(&self, chars: &[char]) -> Result<Vec<u8>> {
        // 使用HarfBuzz进行字体子集化
        let subset_runner = Subset::new();
        subset_runner.clear_drop_table();
        subset_runner.adjust_layout();
        
        // 将字符转换为Unicode码点
        let codepoints: Vec<u32> = chars.iter().map(|&c| c as u32).collect();
        subset_runner.add_chars(&codepoints);
        
        // 执行子集化
        let subset_face = subset_runner.run_subset(&self.harfbuzz_face);
        let subset_data = subset_face.face_data();
        
        Ok(subset_data.get_data().to_vec())
    }
    
    /// 将TTF数据转换为WOFF2格式
    pub fn ttf_to_woff2(ttf_data: &[u8]) -> Result<Vec<u8>> {
        // 使用woff库进行TTF到WOFF2转换
        match woff::version2::compress(ttf_data, String::new(), 1, true) {
            Some(woff2_data) => Ok(woff2_data),
            None => {
                log::warn!("WOFF2转换失败，返回TTF数据");
                Ok(ttf_data.to_vec())
            }
        }
    }
    
    /// 生成包含指定字符的WOFF2字体
    pub fn generate_woff2(&self, codepoints: &[u32]) -> Result<Vec<u8>> {
        let ttf_data = self.subset_font(codepoints)?;
        Self::ttf_to_woff2(&ttf_data)
    }
}