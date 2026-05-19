//! CSV/TSV 读写

use std::io::{Read, Write};
use std::path::Path;

use grido_core::{CellValue, Column, Document};

use crate::error::IoError;
use crate::detect::{detect_format, FileFormat};

/// 读取 CSV/TSV 文件
pub fn read_csv(path: &Path) -> Result<Document, IoError> {
    let format = detect_format(path)?;
    let delimiter = format.delimiter();
    let has_header = format.has_header();

    let content = std::fs::read(path).map_err(|e| IoError::FileRead {
        path: path.display().to_string(),
        source: e,
    })?;

    // 处理 BOM
    let content = if content.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &content[3..]
    } else {
        &content
    };

    read_csv_bytes(content, delimiter, has_header, Some(path))
}

/// 从字节读取 CSV
pub fn read_csv_bytes(
    content: &[u8],
    delimiter: u8,
    has_header: bool,
    path: Option<&Path>,
) -> Result<Document, IoError> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(has_header)
        .flexible(true)
        .from_reader(content);

    // 获取表头
    let headers: Vec<String> = if has_header {
        reader
            .headers()
            .map_err(|e| IoError::CsvParse(e.to_string()))?
            .iter()
            .map(|h| h.to_string())
            .collect()
    } else {
        Vec::new()
    };

    // 读取所有记录
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut max_cols = headers.len();

    for result in reader.records() {
        let record = result.map_err(|e| IoError::CsvParse(e.to_string()))?;
        let row: Vec<String> = record.iter().map(|f| f.to_string()).collect();
        max_cols = max_cols.max(row.len());
        rows.push(row);
    }

    // 构建列存储
    if max_cols == 0 {
        return Ok(Document::new());
    }

    let col_count = max_cols;
    let mut columns: Vec<Column> = (0..col_count)
        .map(|i| {
            let name = if i < headers.len() {
                headers[i].clone()
            } else {
                format!("Column {}", i + 1)
            };
            Column::new(name)
        })
        .collect();

    // 填充数据
    for row in &rows {
        for (ci, col) in columns.iter_mut().enumerate() {
            let value = row
                .get(ci)
                .map(|s| CellValue::parse_auto(s))
                .unwrap_or(CellValue::Empty);
            col.cells.push(value);
        }
    }

    // 推断类型
    for col in &mut columns {
        col.infer_type();
    }

    let mut doc = Document::from_columns(columns);
    if let Some(p) = path {
        doc.set_file_path(p.display().to_string());
    }
    doc.mark_saved();

    Ok(doc)
}

/// 将文档写回 CSV 文件
pub fn write_csv(doc: &Document, path: &Path, delimiter: u8) -> Result<(), IoError> {
    let file = std::fs::File::create(path).map_err(|e| IoError::FileWrite {
        path: path.display().to_string(),
        source: e,
    })?;
    let mut writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(file);

    // 写表头
    let headers: Vec<String> = (0..doc.col_count())
        .map(|i| doc.column_name(i).unwrap_or("").to_string())
        .collect();
    writer
        .write_record(&headers)
        .map_err(|e| IoError::CsvParse(e.to_string()))?;

    // 写数据行
    for row in 0..doc.row_count() {
        let record: Vec<String> = (0..doc.col_count())
            .map(|col| doc.cell(row, col).display_string())
            .collect();
        writer
            .write_record(&record)
            .map_err(|e| IoError::CsvParse(e.to_string()))?;
    }

    writer.flush().map_err(|e| IoError::FileWrite {
        path: path.display().to_string(),
        source: e,
    })?;

    Ok(())
}

/// 将文档写为 CSV 字符串
pub fn write_csv_string(doc: &Document, delimiter: u8) -> Result<String, IoError> {
    let mut buf = Vec::new();
    let mut writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(&mut buf);

    let headers: Vec<String> = (0..doc.col_count())
        .map(|i| doc.column_name(i).unwrap_or("").to_string())
        .collect();
    writer
        .write_record(&headers)
        .map_err(|e| IoError::CsvParse(e.to_string()))?;

    for row in 0..doc.row_count() {
        let record: Vec<String> = (0..doc.col_count())
            .map(|col| doc.cell(row, col).display_string())
            .collect();
        writer
            .write_record(&record)
            .map_err(|e| IoError::CsvParse(e.to_string()))?;
    }

    drop(writer);
    String::from_utf8(buf).map_err(|_| IoError::CsvParse("invalid UTF-8 output".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_csv_bytes() {
        let csv_data = b"name,age,active\nAlice,30,true\nBob,25,false\n";
        let doc = read_csv_bytes(csv_data, b',', true, None).unwrap();
        assert_eq!(doc.row_count(), 2);
        assert_eq!(doc.col_count(), 3);
        assert_eq!(doc.cell(0, 0), &CellValue::Text("Alice".into()));
        assert_eq!(doc.cell(0, 1), &CellValue::Number(30.0));
        assert_eq!(doc.cell(1, 2), &CellValue::Bool(false));
    }

    #[test]
    fn test_roundtrip() {
        let csv_data = b"name,age\nAlice,30\nBob,25\n";
        let doc = read_csv_bytes(csv_data, b',', true, None).unwrap();
        let output = write_csv_string(&doc, b',').unwrap();
        assert!(output.contains("Alice"));
        assert!(output.contains("30"));
    }
}
