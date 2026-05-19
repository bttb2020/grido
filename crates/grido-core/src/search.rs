//! 搜索与替换

use crate::cell_value::CellValue;
use crate::document::Document;

/// 搜索结果 — 匹配的单元格位置
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchMatch {
    /// 行号
    pub row: usize,
    /// 列号
    pub col: usize,
}

/// 搜索选项
#[derive(Clone, Debug)]
pub struct SearchOptions {
    /// 是否区分大小写
    pub case_sensitive: bool,
    /// 是否全词匹配
    pub whole_word: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        SearchOptions {
            case_sensitive: false,
            whole_word: false,
        }
    }
}

/// 在文档中搜索
pub fn search(doc: &Document, query: &str, options: &SearchOptions) -> Vec<SearchMatch> {
    if query.is_empty() {
        return Vec::new();
    }

    let query_lower = if options.case_sensitive {
        query.to_string()
    } else {
        query.to_lowercase()
    };

    let mut results = Vec::new();

    for row in 0..doc.row_count() {
        for col in 0..doc.col_count() {
            let cell = doc.cell(row, col);
            let text = cell.display_string();

            let text_cmp = if options.case_sensitive {
                text.clone()
            } else {
                text.to_lowercase()
            };

            let matches = if options.whole_word {
                text_cmp == query_lower
            } else {
                text_cmp.contains(&query_lower)
            };

            if matches {
                results.push(SearchMatch { row, col });
            }
        }
    }

    results
}

/// 替换所有匹配
pub fn replace_all(
    doc: &mut Document,
    query: &str,
    replacement: &str,
    options: &SearchOptions,
) -> usize {
    let matches = search(doc, query, options);
    let count = matches.len();

    for m in matches.iter().rev() {
        let cell = doc.cell(m.row, m.col);
        let text = cell.display_string();

        let new_text = if options.case_sensitive {
            text.replace(query, replacement)
        } else {
            // 简单的大小写不敏感替换
            let lower = text.to_lowercase();
            let query_lower = query.to_lowercase();
            let mut result = String::new();
            let mut last = 0;
            for (idx, _) in lower.match_indices(&query_lower) {
                result.push_str(&text[last..idx]);
                result.push_str(replacement);
                last = idx + query.len();
            }
            result.push_str(&text[last..]);
            result
        };

        let new_value = CellValue::parse_auto(&new_text);
        let _ = doc.set_cell(m.row, m.col, new_value);
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::Column;

    fn search_doc() -> Document {
        Document::from_columns(vec![
            Column::with_cells("name", vec![
                CellValue::Text("Alice".into()),
                CellValue::Text("Bob".into()),
                CellValue::Text("alice smith".into()),
            ]),
            Column::with_cells("city", vec![
                CellValue::Text("New York".into()),
                CellValue::Text("London".into()),
                CellValue::Text("new york".into()),
            ]),
        ])
    }

    #[test]
    fn test_search_case_insensitive() {
        let doc = search_doc();
        let results = search(&doc, "alice", &SearchOptions::default());
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_case_sensitive() {
        let doc = search_doc();
        let opts = SearchOptions { case_sensitive: true, ..Default::default() };
        let results = search(&doc, "Alice", &opts);
        assert_eq!(results.len(), 1);
    }
}
