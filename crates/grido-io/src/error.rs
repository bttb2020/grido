//! IO 错误类型

use thiserror::Error;

/// IO 错误类型
#[derive(Error, Debug)]
pub enum IoError {
    /// 文件读取错误
    #[error("failed to read file '{path}': {source}")]
    FileRead {
        path: String,
        #[source]
        source: std::io::Error,
    },
    /// 文件写入错误
    #[error("failed to write file '{path}': {source}")]
    FileWrite {
        path: String,
        #[source]
        source: std::io::Error,
    },
    /// CSV 解析错误
    #[error("CSV parse error: {0}")]
    CsvParse(String),
    /// JSON 解析错误
    #[error("JSON parse error: {0}")]
    JsonParse(String),
    /// 不支持的格式
    #[error("unsupported file format: {0}")]
    UnsupportedFormat(String),
}
