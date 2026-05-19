//! 文档模型 — 核心数据结构

use crate::cell_value::CellValue;
use crate::column::{Column, ColumnType, SortOrder};
use crate::edit::{EditCommand, EditHistory};
use crate::error::CoreError;

/// 文档 — 表格式数据的内存表示
#[derive(Clone, Debug)]
pub struct Document {
    /// 列数据（列存储）
    columns: Vec<Column>,
    /// 编辑历史
    history: EditHistory,
    /// 源文件路径
    file_path: Option<String>,
    /// 行排序索引（排序时使用，None 表示原始顺序）
    sort_index: Option<Vec<usize>>,
    /// 当前排序状态
    sort_state: Option<(usize, SortOrder)>,
}

impl Document {
    /// 创建空文档
    pub fn new() -> Self {
        Document {
            columns: Vec::new(),
            history: EditHistory::new(),
            file_path: None,
            sort_index: None,
            sort_state: None,
        }
    }

    /// 从列数据创建文档
    pub fn from_columns(columns: Vec<Column>) -> Self {
        Document {
            columns,
            history: EditHistory::new(),
            file_path: None,
            sort_index: None,
            sort_state: None,
        }
    }

    /// 设置文件路径
    pub fn set_file_path(&mut self, path: impl Into<String>) {
        self.file_path = Some(path.into());
    }

    /// 获取文件路径
    pub fn file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    /// 行数
    pub fn row_count(&self) -> usize {
        self.columns.first().map_or(0, |c| c.len())
    }

    /// 列数
    pub fn col_count(&self) -> usize {
        self.columns.len()
    }

    /// 获取列名
    pub fn column_name(&self, col: usize) -> Option<&str> {
        self.columns.get(col).map(|c| c.name.as_str())
    }

    /// 获取列类型
    pub fn column_type(&self, col: usize) -> Option<ColumnType> {
        self.columns.get(col).map(|c| c.col_type)
    }

    /// 获取列引用
    pub fn column(&self, col: usize) -> Option<&Column> {
        self.columns.get(col)
    }

    /// 将逻辑行映射到物理行（处理排序）
    fn physical_row(&self, logical_row: usize) -> usize {
        self.sort_index
            .as_ref()
            .map_or(logical_row, |idx| idx[logical_row])
    }

    /// 获取单元格值
    pub fn cell(&self, row: usize, col: usize) -> &CellValue {
        let phys_row = self.physical_row(row);
        self.columns
            .get(col)
            .and_then(|c| c.cells.get(phys_row))
            .unwrap_or(&CellValue::Empty)
    }

    /// 设置单元格值（带撤销）
    pub fn set_cell(&mut self, row: usize, col: usize, value: CellValue) -> Result<(), CoreError> {
        if col >= self.col_count() {
            return Err(CoreError::ColumnOutOfRange { col, max: self.col_count() });
        }
        let phys_row = self.physical_row(row);
        if phys_row >= self.row_count() {
            return Err(CoreError::RowOutOfRange { row, max: self.row_count() });
        }

        let old = self.columns[col].cells[phys_row].clone();
        let cmd = EditCommand::SetCell {
            row: phys_row,
            col,
            old,
            new: value.clone(),
        };
        self.columns[col].cells[phys_row] = value;
        self.history.push(cmd);
        Ok(())
    }

    /// 撤销
    pub fn undo(&mut self) -> bool {
        // 克隆命令以避免借用冲突
        let cmd = self.history.undo().cloned();
        if let Some(cmd) = cmd {
            self.apply_undo(&cmd);
            true
        } else {
            false
        }
    }

    /// 重做
    pub fn redo(&mut self) -> bool {
        let cmd = self.history.redo().cloned();
        if let Some(cmd) = cmd {
            self.apply_redo(&cmd);
            true
        } else {
            false
        }
    }

    fn apply_undo(&mut self, cmd: &EditCommand) {
        match cmd {
            EditCommand::SetCell { row, col, old, .. } => {
                self.columns[*col].cells[*row] = old.clone();
            }
            EditCommand::SetRange {
                start_row,
                start_col,
                old_values,
                ..
            } => {
                for (ri, row_vals) in old_values.iter().enumerate() {
                    for (ci, val) in row_vals.iter().enumerate() {
                        self.columns[start_col + ci].cells[start_row + ri] = val.clone();
                    }
                }
            }
            EditCommand::InsertRow { row, .. } => {
                for col in &mut self.columns {
                    col.cells.remove(*row);
                }
            }
            EditCommand::DeleteRow { row, values } => {
                for (ci, val) in values.iter().enumerate() {
                    self.columns[ci].cells.insert(*row, val.clone());
                }
            }
            EditCommand::InsertColumn { col, .. } => {
                self.columns.remove(*col);
            }
            EditCommand::DeleteColumn {
                col, name, values, ..
            } => {
                let column = Column::with_cells(name.clone(), values.clone());
                self.columns.insert(*col, column);
            }
        }
    }

    fn apply_redo(&mut self, cmd: &EditCommand) {
        match cmd {
            EditCommand::SetCell { row, col, new, .. } => {
                self.columns[*col].cells[*row] = new.clone();
            }
            EditCommand::SetRange {
                start_row,
                start_col,
                new_values,
                ..
            } => {
                for (ri, row_vals) in new_values.iter().enumerate() {
                    for (ci, val) in row_vals.iter().enumerate() {
                        self.columns[start_col + ci].cells[start_row + ri] = val.clone();
                    }
                }
            }
            EditCommand::InsertRow { row, values } => {
                for (ci, val) in values.iter().enumerate() {
                    self.columns[ci].cells.insert(*row, val.clone());
                }
            }
            EditCommand::DeleteRow { row, .. } => {
                for col in &mut self.columns {
                    col.cells.remove(*row);
                }
            }
            EditCommand::InsertColumn {
                col, name, values, ..
            } => {
                let column = Column::with_cells(name.clone(), values.clone());
                self.columns.insert(*col, column);
            }
            EditCommand::DeleteColumn { col, .. } => {
                self.columns.remove(*col);
            }
        }
    }

    /// 插入行
    pub fn insert_row(&mut self, row: usize, values: Vec<CellValue>) -> Result<(), CoreError> {
        if row > self.row_count() {
            return Err(CoreError::RowOutOfRange { row, max: self.row_count() });
        }
        let values = if values.len() < self.col_count() {
            let mut v = values;
            v.resize(self.col_count(), CellValue::Empty);
            v
        } else {
            values
        };

        for (ci, val) in values.iter().enumerate() {
            if ci < self.columns.len() {
                self.columns[ci].cells.insert(row, val.clone());
            }
        }
        self.history.push(EditCommand::InsertRow {
            row,
            values,
        });
        self.sort_index = None;
        self.sort_state = None;
        Ok(())
    }

    /// 删除行
    pub fn delete_row(&mut self, row: usize) -> Result<(), CoreError> {
        if row >= self.row_count() {
            return Err(CoreError::RowOutOfRange { row, max: self.row_count() });
        }
        let values: Vec<CellValue> = self.columns.iter().map(|c| c.cells[row].clone()).collect();
        for col in &mut self.columns {
            col.cells.remove(row);
        }
        self.history.push(EditCommand::DeleteRow { row, values });
        self.sort_index = None;
        self.sort_state = None;
        Ok(())
    }

    /// 按列排序
    pub fn sort_by_column(&mut self, col: usize, order: SortOrder) {
        if col >= self.col_count() {
            return;
        }
        let row_count = self.row_count();
        let mut indices: Vec<usize> = (0..row_count).collect();

        let cells = &self.columns[col].cells;
        indices.sort_by(|&a, &b| {
            let va = &cells[a];
            let vb = &cells[b];
            let cmp = compare_cell_values(va, vb);
            match order {
                SortOrder::Ascending => cmp,
                SortOrder::Descending => cmp.reverse(),
            }
        });

        self.sort_index = Some(indices);
        self.sort_state = Some((col, order));
    }

    /// 清除排序
    pub fn clear_sort(&mut self) {
        self.sort_index = None;
        self.sort_state = None;
    }

    /// 获取排序状态
    pub fn sort_state(&self) -> Option<(usize, SortOrder)> {
        self.sort_state
    }

    /// 是否有未保存的修改
    pub fn is_modified(&self) -> bool {
        self.history.is_modified()
    }

    /// 标记已保存
    pub fn mark_saved(&mut self) {
        self.history.mark_saved();
    }

    /// 获取编辑历史引用
    pub fn history(&self) -> &EditHistory {
        &self.history
    }

    /// 获取所有列名
    pub fn column_names(&self) -> Vec<&str> {
        self.columns.iter().map(|c| c.name.as_str()).collect()
    }

    /// 获取行数据（逻辑行 → 所有列的值）
    pub fn row_values(&self, row: usize) -> Vec<&CellValue> {
        let phys_row = self.physical_row(row);
        self.columns
            .iter()
            .map(|c| c.cells.get(phys_row).unwrap_or(&CellValue::Empty))
            .collect()
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

/// 比较两个单元格值（用于排序）
fn compare_cell_values(a: &CellValue, b: &CellValue) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (a, b) {
        (CellValue::Empty, CellValue::Empty) => Ordering::Equal,
        (CellValue::Empty, _) => Ordering::Greater, // 空值排最后
        (_, CellValue::Empty) => Ordering::Less,
        (CellValue::Number(na), CellValue::Number(nb)) => {
            na.partial_cmp(nb).unwrap_or(Ordering::Equal)
        }
        (CellValue::Text(sa), CellValue::Text(sb)) => sa.cmp(sb),
        (CellValue::Bool(ba), CellValue::Bool(bb)) => ba.cmp(bb),
        // 不同类型：Number < Bool < Text
        (CellValue::Number(_), _) => Ordering::Less,
        (_, CellValue::Number(_)) => Ordering::Greater,
        (CellValue::Bool(_), CellValue::Text(_)) => Ordering::Less,
        (CellValue::Text(_), CellValue::Bool(_)) => Ordering::Greater,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_doc() -> Document {
        let columns = vec![
            Column::with_cells("Name", vec![
                CellValue::Text("Alice".into()),
                CellValue::Text("Bob".into()),
                CellValue::Text("Charlie".into()),
            ]),
            Column::with_cells("Age", vec![
                CellValue::Number(30.0),
                CellValue::Number(25.0),
                CellValue::Number(35.0),
            ]),
        ];
        Document::from_columns(columns)
    }

    #[test]
    fn test_basic_access() {
        let doc = sample_doc();
        assert_eq!(doc.row_count(), 3);
        assert_eq!(doc.col_count(), 2);
        assert_eq!(doc.cell(0, 0), &CellValue::Text("Alice".into()));
        assert_eq!(doc.cell(1, 1), &CellValue::Number(25.0));
    }

    #[test]
    fn test_set_cell_and_undo() {
        let mut doc = sample_doc();
        doc.set_cell(0, 1, CellValue::Number(99.0)).unwrap();
        assert_eq!(doc.cell(0, 1), &CellValue::Number(99.0));

        doc.undo();
        assert_eq!(doc.cell(0, 1), &CellValue::Number(30.0));

        doc.redo();
        assert_eq!(doc.cell(0, 1), &CellValue::Number(99.0));
    }

    #[test]
    fn test_sort() {
        let mut doc = sample_doc();
        doc.sort_by_column(1, SortOrder::Ascending);
        // Age sorted: 25 (Bob), 30 (Alice), 35 (Charlie)
        assert_eq!(doc.cell(0, 0), &CellValue::Text("Bob".into()));
        assert_eq!(doc.cell(1, 0), &CellValue::Text("Alice".into()));
        assert_eq!(doc.cell(2, 0), &CellValue::Text("Charlie".into()));
    }

    #[test]
    fn test_insert_delete_row() {
        let mut doc = sample_doc();
        let original_rows = doc.row_count();

        doc.insert_row(1, vec![CellValue::Text("Dave".into()), CellValue::Number(28.0)]).unwrap();
        assert_eq!(doc.row_count(), original_rows + 1);
        assert_eq!(doc.cell(1, 0), &CellValue::Text("Dave".into()));

        doc.delete_row(1).unwrap();
        assert_eq!(doc.row_count(), original_rows);
        assert_eq!(doc.cell(1, 0), &CellValue::Text("Bob".into()));
    }
}
