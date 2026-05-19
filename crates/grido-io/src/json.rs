//! JSON/JSONL 读取

use std::path::Path;

use grido_core::{CellValue, Column, Document};

use crate::error::IoError;

/// 读取 JSON 文件（数组 of 对象）
pub fn read_json(path: &Path) -> Result<Document, IoError> {
    let content = std::fs::read_to_string(path).map_err(|e| IoError::FileRead {
        path: path.display().to_string(),
        source: e,
    })?;

    let trimmed = content.trim();

    if trimmed.starts_with('[') {
        // JSON array of objects
        parse_json_array(trimmed, Some(path))
    } else if trimmed.starts_with('{') {
        // 可能是 JSONL 或单个对象
        parse_jsonl(trimmed, Some(path))
    } else {
        parse_jsonl(trimmed, Some(path))
    }
}

fn parse_json_array(content: &str, path: Option<&Path>) -> Result<Document, IoError> {
    let arr: Vec<serde_json::Value> =
        serde_json::from_str(content).map_err(|e| IoError::JsonParse(e.to_string()))?;

    json_values_to_doc(&arr, path)
}

fn parse_jsonl(content: &str, path: Option<&Path>) -> Result<Document, IoError> {
    let values: Result<Vec<serde_json::Value>, _> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l))
        .collect();

    let values = values.map_err(|e| IoError::JsonParse(e.to_string()))?;
    json_values_to_doc(&values, path)
}

fn json_values_to_doc(
    values: &[serde_json::Value],
    path: Option<&Path>,
) -> Result<Document, IoError> {
    if values.is_empty() {
        return Ok(Document::new());
    }

    // 收集所有 key（保持顺序）
    let mut keys: Vec<String> = Vec::new();
    for val in values {
        if let serde_json::Value::Object(obj) = val {
            for key in obj.keys() {
                if !keys.contains(key) {
                    keys.push(key.clone());
                }
            }
        }
    }

    if keys.is_empty() {
        return Ok(Document::new());
    }

    // 构建列
    let mut columns: Vec<Column> = keys.iter().map(|k| Column::new(k.clone())).collect();

    for val in values {
        if let serde_json::Value::Object(obj) = val {
            for (ci, key) in keys.iter().enumerate() {
                let cell = match obj.get(key) {
                    None | Some(serde_json::Value::Null) => CellValue::Empty,
                    Some(serde_json::Value::Bool(b)) => CellValue::Bool(*b),
                    Some(serde_json::Value::Number(n)) => {
                        CellValue::Number(n.as_f64().unwrap_or(0.0))
                    }
                    Some(serde_json::Value::String(s)) => {
                        if s.is_empty() {
                            CellValue::Empty
                        } else {
                            CellValue::Text(s.clone())
                        }
                    }
                    Some(other) => CellValue::Text(other.to_string()),
                };
                columns[ci].cells.push(cell);
            }
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_array() {
        let json = r#"[{"name":"Alice","age":30},{"name":"Bob","age":25}]"#;
        let doc = parse_json_array(json, None).unwrap();
        assert_eq!(doc.row_count(), 2);
        assert_eq!(doc.col_count(), 2);
    }

    #[test]
    fn test_parse_jsonl() {
        let jsonl = "{\"name\":\"Alice\",\"age\":30}\n{\"name\":\"Bob\",\"age\":25}\n";
        let doc = parse_jsonl(jsonl, None).unwrap();
        assert_eq!(doc.row_count(), 2);
    }
}
