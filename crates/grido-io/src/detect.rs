//! 文件格式检测

use std::path::Path;

use crate::error::IoError;

/// 文件格式枚举
#[derive(Clone, Debug, PartialEq)]
pub enum FileFormat {
    /// CSV 文件
    Csv {
        /// 分隔符
        delimiter: u8,
        /// 是否有表头
        has_header: bool,
    },
    /// TSV 文件
    Tsv {
        /// 是否有表头
        has_header: bool,
    },
    /// JSON 文件
    Json,
    /// JSON Lines 文件
    JsonLines,
    /// Parquet 文件 (V1+)
    Parquet,
    /// XLSX 文件 (V1+)
    Xlsx {
        /// 工作表名
        sheet_name: Option<String>,
    },
}

impl FileFormat {
    /// 获取分隔符
    pub fn delimiter(&self) -> u8 {
        match self {
            FileFormat::Csv { delimiter, .. } => *delimiter,
            FileFormat::Tsv { .. } => b'\t',
            _ => b',',
        }
    }

    /// 是否有表头
    pub fn has_header(&self) -> bool {
        match self {
            FileFormat::Csv { has_header, .. } | FileFormat::Tsv { has_header } => *has_header,
            _ => true,
        }
    }
}

/// 从文件路径检测格式
pub fn detect_format(path: &Path) -> Result<FileFormat, IoError> {
    // 1. 根据扩展名判断
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "tsv" | "tab" => {
            return Ok(FileFormat::Tsv { has_header: true });
        }
        "json" => {
            return Ok(FileFormat::Json);
        }
        "jsonl" | "ndjson" => {
            return Ok(FileFormat::JsonLines);
        }
        "parquet" | "pq" => {
            return Ok(FileFormat::Parquet);
        }
        "xlsx" | "xls" => {
            return Ok(FileFormat::Xlsx { sheet_name: None });
        }
        _ => {}
    }

    // 2. 对于 .csv 或未知扩展名，检测分隔符
    let content = std::fs::read(path).map_err(|e| IoError::FileRead {
        path: path.display().to_string(),
        source: e,
    })?;

    let delimiter = detect_delimiter(&content);
    let has_header = detect_header(&content, delimiter);

    Ok(FileFormat::Csv {
        delimiter,
        has_header,
    })
}

/// 从文件内容检测分隔符
pub fn detect_delimiter(content: &[u8]) -> u8 {
    // 取前 8KB 分析
    let sample_size = content.len().min(8192);
    let sample = &content[..sample_size];

    // 跳过 BOM
    let sample = if sample.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &sample[3..]
    } else {
        sample
    };

    let delimiters = [b',', b'\t', b';', b'|'];
    let mut scores = [0u32; 4];

    // 统计每行中各分隔符的一致性
    let lines: Vec<&[u8]> = sample.split(|&b| b == b'\n').take(20).collect();
    if lines.len() < 2 {
        return b','; // 默认逗号
    }

    for (di, &delim) in delimiters.iter().enumerate() {
        let counts: Vec<usize> = lines
            .iter()
            .filter(|l| !l.is_empty())
            .map(|l| l.iter().filter(|&&b| b == delim).count())
            .collect();

        if counts.is_empty() || counts[0] == 0 {
            continue;
        }

        // 一致性得分：所有行分隔符数量相同得高分
        let first = counts[0];
        let consistent = counts.iter().filter(|&&c| c == first).count();
        scores[di] = (consistent * first) as u32;
    }

    let best = scores
        .iter()
        .enumerate()
        .max_by_key(|(_, &s)| s)
        .map(|(i, _)| i)
        .unwrap_or(0);

    if scores[best] == 0 {
        b','
    } else {
        delimiters[best]
    }
}

/// 检测首行是否为表头
fn detect_header(content: &[u8], delimiter: u8) -> bool {
    let sample = if content.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &content[3..]
    } else {
        content
    };

    // 获取前两行
    let mut lines = sample.split(|&b| b == b'\n');
    let first_line = match lines.next() {
        Some(l) => l,
        None => return true,
    };
    let second_line = match lines.next() {
        Some(l) => l,
        None => return true,
    };

    // 如果第一行字段都是非数字的文本，而第二行有数字，很可能是表头
    let first_fields: Vec<&[u8]> = first_line.split(|&b| b == delimiter).collect();
    let second_fields: Vec<&[u8]> = second_line.split(|&b| b == delimiter).collect();

    if first_fields.is_empty() {
        return true;
    }

    let first_all_text = first_fields.iter().all(|f| {
        let s = String::from_utf8_lossy(f);
        let trimmed = s.trim().trim_matches('"');
        trimmed.parse::<f64>().is_err()
    });

    let second_has_numbers = second_fields.iter().any(|f| {
        let s = String::from_utf8_lossy(f);
        let trimmed = s.trim().trim_matches('"');
        trimmed.parse::<f64>().is_ok()
    });

    first_all_text && (second_has_numbers || first_fields.len() == second_fields.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_comma() {
        let content = b"a,b,c\n1,2,3\n4,5,6\n";
        assert_eq!(detect_delimiter(content), b',');
    }

    #[test]
    fn test_detect_tab() {
        let content = b"a\tb\tc\n1\t2\t3\n4\t5\t6\n";
        assert_eq!(detect_delimiter(content), b'\t');
    }

    #[test]
    fn test_detect_semicolon() {
        let content = b"a;b;c\n1;2;3\n4;5;6\n";
        assert_eq!(detect_delimiter(content), b';');
    }
}
