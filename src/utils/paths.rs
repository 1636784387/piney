use std::path::PathBuf;

/// 获取数据目录
pub fn get_data_dir() -> PathBuf {
    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_string());
    PathBuf::from(data_dir)
}

/// 获取相对于数据目录的子路径
pub fn get_data_path(subpath: &str) -> PathBuf {
    get_data_dir().join(subpath)
}
