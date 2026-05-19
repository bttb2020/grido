//! 选区逻辑

/// 单元格位置
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellPos {
    pub row: usize,
    pub col: usize,
}

impl CellPos {
    pub fn new(row: usize, col: usize) -> Self {
        CellPos { row, col }
    }
}

/// 矩形选区
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    /// 锚点（选区开始位置）
    pub anchor: CellPos,
    /// 当前位置（选区结束位置）
    pub cursor: CellPos,
}

impl Selection {
    /// 创建单个单元格选区
    pub fn single(row: usize, col: usize) -> Self {
        let pos = CellPos::new(row, col);
        Selection {
            anchor: pos,
            cursor: pos,
        }
    }

    /// 创建范围选区
    pub fn range(anchor: CellPos, cursor: CellPos) -> Self {
        Selection { anchor, cursor }
    }

    /// 获取选区的最小行
    pub fn min_row(&self) -> usize {
        self.anchor.row.min(self.cursor.row)
    }

    /// 获取选区的最大行
    pub fn max_row(&self) -> usize {
        self.anchor.row.max(self.cursor.row)
    }

    /// 获取选区的最小列
    pub fn min_col(&self) -> usize {
        self.anchor.col.min(self.cursor.col)
    }

    /// 获取选区的最大列
    pub fn max_col(&self) -> usize {
        self.anchor.col.max(self.cursor.col)
    }

    /// 判断单元格是否在选区内
    pub fn contains(&self, row: usize, col: usize) -> bool {
        row >= self.min_row()
            && row <= self.max_row()
            && col >= self.min_col()
            && col <= self.max_col()
    }

    /// 选中的行数
    pub fn row_count(&self) -> usize {
        self.max_row() - self.min_row() + 1
    }

    /// 选中的列数
    pub fn col_count(&self) -> usize {
        self.max_col() - self.min_col() + 1
    }

    /// 是否为单个单元格
    pub fn is_single(&self) -> bool {
        self.anchor == self.cursor
    }
}

impl Default for Selection {
    fn default() -> Self {
        Selection::single(0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_selection() {
        let sel = Selection::single(5, 3);
        assert!(sel.contains(5, 3));
        assert!(!sel.contains(5, 4));
        assert!(sel.is_single());
    }

    #[test]
    fn test_range_selection() {
        let sel = Selection::range(CellPos::new(1, 1), CellPos::new(3, 4));
        assert!(sel.contains(2, 2));
        assert!(!sel.contains(0, 0));
        assert_eq!(sel.row_count(), 3);
        assert_eq!(sel.col_count(), 4);
    }
}
