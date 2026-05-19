//! 列定义与类型推断

use crate::cell_value::CellValue;

/// 列类型枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColumnType {
    /// 文本列
    Text,
    /// 数字列
    Number,
    /// 布尔列
    Bool,
    /// 混合类型
    Mixed,
}

/// 列定义 — 名称、类型、数据
#[derive(Clone, Debug)]
pub struct Column {
    /// 列名
    pub name: String,
    /// 推断的列类型
    pub col_type: ColumnType,
    /// 单元格数据
    pub cells: Vec<CellValue>,
}

impl Column {
    /// 创建空列
    pub fn new(name: impl Into<String>) -> Self {
        Column {
            name: name.into(),
            col_type: ColumnType::Text,
            cells: Vec::new(),
        }
    }

    /// 创建带数据的列
    pub fn with_cells(name: impl Into<String>, cells: Vec<CellValue>) -> Self {
        let mut col = Column {
            name: name.into(),
            col_type: ColumnType::Text,
            cells,
        };
        col.infer_type();
        col
    }

    /// 根据数据推断列类型（扫描前 1000 行）
    pub fn infer_type(&mut self) {
        let sample_size = self.cells.len().min(1000);
        if sample_size == 0 {
            self.col_type = ColumnType::Text;
            return;
        }

        let mut num_count = 0usize;
        let mut bool_count = 0usize;
        let mut text_count = 0usize;
        let mut non_empty = 0usize;

        for cell in self.cells.iter().take(sample_size) {
            match cell {
                CellValue::Empty => {}
                CellValue::Number(_) => {
                    num_count += 1;
                    non_empty += 1;
                }
                CellValue::Bool(_) => {
                    bool_count += 1;
                    non_empty += 1;
                }
                CellValue::Text(_) => {
                    text_count += 1;
                    non_empty += 1;
                }
            }
        }

        if non_empty == 0 {
            self.col_type = ColumnType::Text;
        } else if num_count == non_empty {
            self.col_type = ColumnType::Number;
        } else if bool_count == non_empty {
            self.col_type = ColumnType::Bool;
        } else if text_count == non_empty {
            self.col_type = ColumnType::Text;
        } else {
            self.col_type = ColumnType::Mixed;
        }
    }

    /// 行数
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// 是否为空列
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

/// 排序方向
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortOrder {
    /// 升序
    Ascending,
    /// 降序
    Descending,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_number_column() {
        let col = Column::with_cells("age", vec![
            CellValue::Number(25.0),
            CellValue::Number(30.0),
            CellValue::Empty,
            CellValue::Number(42.0),
        ]);
        assert_eq!(col.col_type, ColumnType::Number);
    }

    #[test]
    fn test_infer_mixed_column() {
        let col = Column::with_cells("data", vec![
            CellValue::Number(1.0),
            CellValue::Text("hello".into()),
        ]);
        assert_eq!(col.col_type, ColumnType::Mixed);
    }
}
