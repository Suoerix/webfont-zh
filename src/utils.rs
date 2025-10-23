use std::path::Path;

/// 解析逗号分隔的unicode码点字符串
pub fn parse_codepoints(chars_str: &str) -> Result<Vec<u32>, std::num::ParseIntError> {
    chars_str
        .split(',')
        .map(|s| s.trim().parse::<u32>())
        .collect()
}

/// 生成缓存文件名
pub fn generate_cache_filename(codepoints: &[u32]) -> String {
    let mut sorted_codepoints = codepoints.to_vec();
    sorted_codepoints.sort_unstable();
    
    if sorted_codepoints.len() == 1 {
        format!("{}.woff2", sorted_codepoints[0])
    } else {
        let codepoints_str = sorted_codepoints
            .iter()
            .map(|cp| cp.to_string())
            .collect::<Vec<_>>()
            .join(",");
        format!("cache/{}.woff2", codepoints_str)
    }
}

/// 生成文件的MD5哈希
pub fn generate_file_hash(data: &[u8]) -> String {
    format!("{:x}", md5::compute(data))
}

/// 检查文件是否过期
pub fn is_file_expired(file_path: &Path, days: u64) -> bool {
    if let Ok(metadata) = std::fs::metadata(file_path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.elapsed() {
                return duration.as_secs() > days * 24 * 3600;
            }
        }
    }
    true // 如果无法获取文件信息，认为已过期
}

/// 清理过期的缓存文件
pub fn cleanup_expired_cache(cache_dir: &Path, days: u64) -> std::io::Result<usize> {
    let mut cleaned_count = 0;
    
    if cache_dir.exists() {
        for entry in std::fs::read_dir(cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && is_file_expired(&path, days) {
                if std::fs::remove_file(&path).is_ok() {
                    cleaned_count += 1;
                    log::info!("清理过期缓存文件: {:?}", path);
                }
            }
        }
    }
    
    Ok(cleaned_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_codepoints() {
        assert_eq!(parse_codepoints("40339").unwrap(), vec![40339]);
        assert_eq!(parse_codepoints("40339,40340,40341").unwrap(), vec![40339, 40340, 40341]);
        assert_eq!(parse_codepoints("40339, 40340, 40341").unwrap(), vec![40339, 40340, 40341]);
    }

    #[test]
    fn test_generate_cache_filename() {
        assert_eq!(generate_cache_filename(&[40339]), "40339.woff2");
        assert_eq!(generate_cache_filename(&[40341, 40339, 40340]), "cache/40339,40340,40341.woff2");
    }
}