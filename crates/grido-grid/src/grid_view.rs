//! 网格视图 Widget — Makepad 实现

use makepad_widgets::*;

use grido_core::{CellValue, ColumnType, Document};

use crate::selection::{CellPos, Selection};
use crate::viewport::Viewport;

/// 网格视图状态
pub struct GridViewState {
    /// 文档数据
    pub document: Option<Document>,
    /// 视口
    pub viewport: Viewport,
    /// 当前选区
    pub selection: Selection,
    /// 是否在编辑模式
    pub editing: bool,
    /// 编辑缓冲
    pub edit_buffer: String,
    /// 文件修改标记
    pub modified: bool,
}

impl GridViewState {
    pub fn new() -> Self {
        GridViewState {
            document: None,
            viewport: Viewport::new(),
            selection: Selection::default(),
            editing: false,
            edit_buffer: String::new(),
            modified: false,
        }
    }

    /// 加载文档
    pub fn load_document(&mut self, doc: Document) {
        let rows = doc.row_count();
        let cols = doc.col_count();
        self.viewport.total_rows = rows;
        self.viewport.set_default_col_widths(cols);
        self.selection = Selection::single(0, 0);
        self.editing = false;
        self.edit_buffer.clear();
        self.modified = false;
        self.document = Some(doc);
    }

    /// 移动光标
    pub fn move_cursor(&mut self, drow: isize, dcol: isize, extend_selection: bool) {
        let doc = match &self.document {
            Some(d) => d,
            None => return,
        };

        let max_row = doc.row_count().saturating_sub(1);
        let max_col = doc.col_count().saturating_sub(1);

        let new_row = (self.selection.cursor.row as isize + drow)
            .clamp(0, max_row as isize) as usize;
        let new_col = (self.selection.cursor.col as isize + dcol)
            .clamp(0, max_col as isize) as usize;

        let new_pos = CellPos::new(new_row, new_col);

        if extend_selection {
            self.selection.cursor = new_pos;
        } else {
            self.selection = Selection::single(new_row, new_col);
        }

        self.viewport.ensure_visible(new_row, new_col);
    }

    /// 进入编辑模式
    pub fn start_editing(&mut self) {
        if let Some(doc) = &self.document {
            let cell = doc.cell(self.selection.cursor.row, self.selection.cursor.col);
            self.edit_buffer = cell.display_string();
            self.editing = true;
        }
    }

    /// 确认编辑
    pub fn commit_edit(&mut self) {
        if !self.editing {
            return;
        }
        let row = self.selection.cursor.row;
        let col = self.selection.cursor.col;
        let value = CellValue::parse_auto(&self.edit_buffer);

        if let Some(doc) = &mut self.document {
            let _ = doc.set_cell(row, col, value);
        }

        self.editing = false;
        self.edit_buffer.clear();
        self.modified = true;
    }

    /// 取消编辑
    pub fn cancel_edit(&mut self) {
        self.editing = false;
        self.edit_buffer.clear();
    }

    /// 删除选中单元格内容
    pub fn delete_selection(&mut self) {
        if let Some(doc) = &mut self.document {
            for row in self.selection.min_row()..=self.selection.max_row() {
                for col in self.selection.min_col()..=self.selection.max_col() {
                    let _ = doc.set_cell(row, col, CellValue::Empty);
                }
            }
            self.modified = true;
        }
    }

    /// 撤销
    pub fn undo(&mut self) {
        if let Some(doc) = &mut self.document {
            if doc.undo() {
                self.modified = doc.is_modified();
            }
        }
    }

    /// 重做
    pub fn redo(&mut self) {
        if let Some(doc) = &mut self.document {
            if doc.redo() {
                self.modified = doc.is_modified();
            }
        }
    }

    /// 获取选区文本（TSV 格式，用于复制到剪贴板）
    pub fn selection_as_tsv(&self) -> String {
        let doc = match &self.document {
            Some(d) => d,
            None => return String::new(),
        };

        let mut result = String::new();
        for row in self.selection.min_row()..=self.selection.max_row() {
            for col in self.selection.min_col()..=self.selection.max_col() {
                if col > self.selection.min_col() {
                    result.push('\t');
                }
                result.push_str(&doc.cell(row, col).display_string());
            }
            result.push('\n');
        }
        result
    }

    /// 计算选区统计（SUM, AVG, COUNT, MIN, MAX）
    pub fn selection_stats(&self) -> Option<SelectionStats> {
        let doc = self.document.as_ref()?;

        let mut sum = 0.0f64;
        let mut count = 0usize;
        let mut num_count = 0usize;
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;

        for row in self.selection.min_row()..=self.selection.max_row() {
            for col in self.selection.min_col()..=self.selection.max_col() {
                let cell = doc.cell(row, col);
                if !cell.is_empty() {
                    count += 1;
                }
                if let Some(n) = cell.as_number() {
                    sum += n;
                    num_count += 1;
                    if n < min {
                        min = n;
                    }
                    if n > max {
                        max = n;
                    }
                }
            }
        }

        if count == 0 {
            return None;
        }

        Some(SelectionStats {
            sum,
            avg: if num_count > 0 {
                sum / num_count as f64
            } else {
                0.0
            },
            count,
            num_count,
            min: if num_count > 0 { min } else { 0.0 },
            max: if num_count > 0 { max } else { 0.0 },
        })
    }
}

/// 选区统计信息
#[derive(Clone, Debug)]
pub struct SelectionStats {
    pub sum: f64,
    pub avg: f64,
    pub count: usize,
    pub num_count: usize,
    pub min: f64,
    pub max: f64,
}

impl SelectionStats {
    /// 格式化为状态栏文本
    pub fn status_text(&self) -> String {
        if self.num_count > 0 {
            format!(
                "SUM: {:.2}  AVG: {:.2}  COUNT: {}  MIN: {:.2}  MAX: {:.2}",
                self.sum, self.avg, self.count, self.min, self.max
            )
        } else {
            format!("COUNT: {}", self.count)
        }
    }
}

impl Default for GridViewState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use grido_core::{Column, Document};

    fn test_state() -> GridViewState {
        let doc = Document::from_columns(vec![
            Column::with_cells(
                "A",
                vec![
                    CellValue::Number(10.0),
                    CellValue::Number(20.0),
                    CellValue::Number(30.0),
                ],
            ),
            Column::with_cells(
                "B",
                vec![
                    CellValue::Text("x".into()),
                    CellValue::Text("y".into()),
                    CellValue::Text("z".into()),
                ],
            ),
        ]);
        let mut state = GridViewState::new();
        state.load_document(doc);
        state
    }

    #[test]
    fn test_move_cursor() {
        let mut state = test_state();
        state.move_cursor(1, 0, false);
        assert_eq!(state.selection.cursor.row, 1);

        state.move_cursor(0, 1, false);
        assert_eq!(state.selection.cursor.col, 1);

        // 不越界
        state.move_cursor(100, 0, false);
        assert_eq!(state.selection.cursor.row, 2);
    }

    #[test]
    fn test_edit_cycle() {
        let mut state = test_state();
        state.start_editing();
        assert!(state.editing);

        state.edit_buffer = "99".to_string();
        state.commit_edit();

        let doc = state.document.as_ref().unwrap();
        assert_eq!(doc.cell(0, 0), &CellValue::Number(99.0));
    }

    #[test]
    fn test_selection_stats() {
        let mut state = test_state();
        state.selection = Selection::range(CellPos::new(0, 0), CellPos::new(2, 0));
        let stats = state.selection_stats().unwrap();
        assert_eq!(stats.sum, 60.0);
        assert_eq!(stats.num_count, 3);
    }

    #[test]
    fn test_selection_as_tsv() {
        let state = test_state();
        let tsv = state.selection_as_tsv();
        assert_eq!(tsv, "10\n");
    }
}
