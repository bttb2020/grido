//! 单元格值类型定义

use std::fmt;

/// 单元格值枚举 — 支持空、文本、数字、布尔
#[derive(Clone, PartialEq, Debug)]
pub enum CellValue {
    /// 空单元格
    Empty,
    /// 文本值
    Text(String),
    /// 数字值 (f64)
    Number(f64),
    /// 布尔值
    Bool(bool),
}

impl CellValue {
    /// 判断是否为空
    pub fn is_empty(&self) -> bool {
        matches!(self, CellValue::Empty)
    }

    /// 尝试获取数字值
    pub fn as_number(&self) -> Option<f64> {
        match self {
            CellValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// 尝试获取文本值
    pub fn as_text(&self) -> Option<&str> {
        match self {
            CellValue::Text(s) => Some(s),
            _ => None,
        }
    }

    /// 将值转为显示字符串
    pub fn display_string(&self) -> String {
        match self {
            CellValue::Empty => String::new(),
            CellValue::Text(s) => s.clone(),
            CellValue::Number(n) => {
                if *n == (*n as i64) as f64 && n.is_finite() {
                    format!("{}", *n as i64)
                } else {
                    format!("{n}")
                }
            }
            CellValue::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
        }
    }

    /// 从字符串推断类型并解析
    pub fn parse_auto(s: &str) -> CellValue {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return CellValue::Empty;
        }
        // 布尔
        match trimmed.to_lowercase().as_str() {
            "true" | "yes" => return CellValue::Bool(true),
            "false" | "no" => return CellValue::Bool(false),
            _ => {}
        }
        // 数字
        if let Ok(n) = trimmed.parse::<f64>() {
            if n.is_finite() {
                return CellValue::Number(n);
            }
        }
        CellValue::Text(s.to_string())
    }
}

impl Default for CellValue {
    fn default() -> Self {
        CellValue::Empty
    }
}

impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_string())
    }
}

impl From<&str> for CellValue {
    fn from(s: &str) -> Self {
        CellValue::Text(s.to_string())
    }
}

impl From<String> for CellValue {
    fn from(s: String) -> Self {
        CellValue::Text(s)
    }
}

impl From<f64> for CellValue {
    fn from(n: f64) -> Self {
        CellValue::Number(n)
    }
}

impl From<bool> for CellValue {
    fn from(b: bool) -> Self {
        CellValue::Bool(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_auto() {
        assert_eq!(CellValue::parse_auto(""), CellValue::Empty);
        assert_eq!(CellValue::parse_auto("42"), CellValue::Number(42.0));
        assert_eq!(CellValue::parse_auto("3.14"), CellValue::Number(3.14));
        assert_eq!(CellValue::parse_auto("true"), CellValue::Bool(true));
        assert_eq!(CellValue::parse_auto("hello"), CellValue::Text("hello".into()));
    }

    #[test]
    fn test_display_string() {
        assert_eq!(CellValue::Number(42.0).display_string(), "42");
        assert_eq!(CellValue::Number(3.14).display_string(), "3.14");
        assert_eq!(CellValue::Bool(true).display_string(), "TRUE");
        assert_eq!(CellValue::Empty.display_string(), "");
    }
}
