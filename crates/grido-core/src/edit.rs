//! 编辑操作与撤销/重做

use crate::cell_value::CellValue;

/// 编辑命令 — 用于撤销/重做
#[derive(Clone, Debug)]
pub enum EditCommand {
    /// 设置单个单元格
    SetCell {
        row: usize,
        col: usize,
        old: CellValue,
        new: CellValue,
    },
    /// 设置矩形范围
    SetRange {
        start_row: usize,
        start_col: usize,
        old_values: Vec<Vec<CellValue>>,
        new_values: Vec<Vec<CellValue>>,
    },
    /// 插入行
    InsertRow {
        row: usize,
        values: Vec<CellValue>,
    },
    /// 删除行
    DeleteRow {
        row: usize,
        values: Vec<CellValue>,
    },
    /// 插入列
    InsertColumn {
        col: usize,
        name: String,
        values: Vec<CellValue>,
    },
    /// 删除列
    DeleteColumn {
        col: usize,
        name: String,
        values: Vec<CellValue>,
    },
}

/// 编辑历史 — 管理撤销/重做栈
#[derive(Clone, Debug)]
pub struct EditHistory {
    /// 撤销栈
    undo_stack: Vec<EditCommand>,
    /// 重做栈
    redo_stack: Vec<EditCommand>,
    /// 上次保存时撤销栈的长度
    saved_at: usize,
}

impl EditHistory {
    /// 创建空的编辑历史
    pub fn new() -> Self {
        EditHistory {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            saved_at: 0,
        }
    }

    /// 记录一个编辑操作
    pub fn push(&mut self, cmd: EditCommand) {
        self.undo_stack.push(cmd);
        self.redo_stack.clear();
    }

    /// 撤销 — 返回要反转的命令
    pub fn undo(&mut self) -> Option<&EditCommand> {
        if let Some(cmd) = self.undo_stack.pop() {
            self.redo_stack.push(cmd);
            self.redo_stack.last()
        } else {
            None
        }
    }

    /// 重做 — 返回要重放的命令
    pub fn redo(&mut self) -> Option<&EditCommand> {
        if let Some(cmd) = self.redo_stack.pop() {
            self.undo_stack.push(cmd);
            self.undo_stack.last()
        } else {
            None
        }
    }

    /// 标记当前为已保存状态
    pub fn mark_saved(&mut self) {
        self.saved_at = self.undo_stack.len();
    }

    /// 是否有未保存的修改
    pub fn is_modified(&self) -> bool {
        self.undo_stack.len() != self.saved_at
    }

    /// 是否可以撤销
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// 是否可以重做
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// 清空历史
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.saved_at = 0;
    }
}

impl Default for EditHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_redo() {
        let mut history = EditHistory::new();
        assert!(!history.can_undo());

        history.push(EditCommand::SetCell {
            row: 0,
            col: 0,
            old: CellValue::Empty,
            new: CellValue::Number(42.0),
        });
        assert!(history.can_undo());
        assert!(!history.can_redo());

        history.undo();
        assert!(!history.can_undo());
        assert!(history.can_redo());

        history.redo();
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_modified_tracking() {
        let mut history = EditHistory::new();
        assert!(!history.is_modified());

        history.push(EditCommand::SetCell {
            row: 0,
            col: 0,
            old: CellValue::Empty,
            new: CellValue::Number(1.0),
        });
        assert!(history.is_modified());

        history.mark_saved();
        assert!(!history.is_modified());

        history.push(EditCommand::SetCell {
            row: 0,
            col: 1,
            old: CellValue::Empty,
            new: CellValue::Number(2.0),
        });
        assert!(history.is_modified());
    }
}
