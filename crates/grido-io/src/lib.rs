//! Grido 文件 I/O — CSV、JSON、Parquet 等格式的读写

pub mod csv;
pub mod detect;
pub mod error;
pub mod json;

use std::path::Path;

use grido_core::Document;

pub use detect::{detect_format, FileFormat};
pub use error::IoError;

/// 打开文件（自动检测格式）
pub fn open_file(path: &Path) -> Result<Document, IoError> {
    let format = detect_format(path)?;

    match format {
        FileFormat::Csv { .. } | FileFormat::Tsv { .. } => csv::read_csv(path),
        FileFormat::Json | FileFormat::JsonLines => json::read_json(path),
        FileFormat::Parquet => Err(IoError::UnsupportedFormat("Parquet".into())),
        FileFormat::Xlsx { .. } => Err(IoError::UnsupportedFormat("XLSX".into())),
    }
}

/// 保存文件
pub fn save_file(doc: &Document, path: &Path) -> Result<(), IoError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("csv")
        .to_lowercase();

    match ext.as_str() {
        "tsv" | "tab" => csv::write_csv(doc, path, b'\t'),
        _ => csv::write_csv(doc, path, b','),
    }
}
