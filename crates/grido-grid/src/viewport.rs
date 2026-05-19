//! 可视区域计算 — 虚拟滚动核心

/// 视口状态 — 管理可视区域
#[derive(Clone, Debug)]
pub struct Viewport {
    /// 水平滚动偏移 (像素)
    pub scroll_x: f64,
    /// 垂直滚动偏移 (像素)
    pub scroll_y: f64,
    /// 视口宽度 (像素)
    pub width: f64,
    /// 视口高度 (像素)
    pub height: f64,
    /// 行高 (像素)
    pub row_height: f64,
    /// 行号列宽度
    pub row_number_width: f64,
    /// 表头高度
    pub header_height: f64,
    /// 列宽数组
    pub col_widths: Vec<f64>,
    /// 总行数
    pub total_rows: usize,
    /// 总列数
    pub total_cols: usize,
}

impl Viewport {
    /// 创建默认视口
    pub fn new() -> Self {
        Viewport {
            scroll_x: 0.0,
            scroll_y: 0.0,
            width: 800.0,
            height: 600.0,
            row_height: 24.0,
            row_number_width: 60.0,
            header_height: 28.0,
            col_widths: Vec::new(),
            total_rows: 0,
            total_cols: 0,
        }
    }

    /// 首个可见行索引
    pub fn first_visible_row(&self) -> usize {
        (self.scroll_y / self.row_height).floor().max(0.0) as usize
    }

    /// 可见行数（含两端多绘一行缓冲）
    pub fn visible_row_count(&self) -> usize {
        let data_height = self.height - self.header_height;
        let count = (data_height / self.row_height).ceil() as usize + 2;
        count.min(self.total_rows.saturating_sub(self.first_visible_row()))
    }

    /// 首个可见列索引
    pub fn first_visible_col(&self) -> usize {
        let mut x = 0.0;
        for (i, &w) in self.col_widths.iter().enumerate() {
            if x + w > self.scroll_x {
                return i;
            }
            x += w;
        }
        0
    }

    /// 可见列数
    pub fn visible_col_count(&self) -> usize {
        let start = self.first_visible_col();
        let data_width = self.width - self.row_number_width;
        let mut x = 0.0;
        let mut count = 0;

        // 先扣掉开始列前面已滚过的部分
        let mut offset = 0.0;
        for &w in self.col_widths.iter().take(start) {
            offset += w;
        }
        let partial = offset - self.scroll_x;

        for &w in self.col_widths.iter().skip(start) {
            if x - partial >= data_width {
                break;
            }
            x += w;
            count += 1;
        }
        count.min(self.total_cols.saturating_sub(start))
    }

    /// 列的 X 坐标（相对于视口）
    pub fn col_x(&self, col: usize) -> f64 {
        let mut x = self.row_number_width;
        for &w in self.col_widths.iter().take(col) {
            x += w;
        }
        x - self.scroll_x
    }

    /// 行的 Y 坐标（相对于视口）
    pub fn row_y(&self, row: usize) -> f64 {
        self.header_height + (row as f64 * self.row_height) - self.scroll_y
    }

    /// 获取列宽
    pub fn col_width(&self, col: usize) -> f64 {
        self.col_widths.get(col).copied().unwrap_or(100.0)
    }

    /// 总内容宽度
    pub fn content_width(&self) -> f64 {
        self.col_widths.iter().sum::<f64>() + self.row_number_width
    }

    /// 总内容高度
    pub fn content_height(&self) -> f64 {
        (self.total_rows as f64) * self.row_height + self.header_height
    }

    /// 最大垂直滚动
    pub fn max_scroll_y(&self) -> f64 {
        (self.content_height() - self.height).max(0.0)
    }

    /// 最大水平滚动
    pub fn max_scroll_x(&self) -> f64 {
        (self.content_width() - self.width).max(0.0)
    }

    /// 限制滚动到有效范围
    pub fn clamp_scroll(&mut self) {
        self.scroll_x = self.scroll_x.clamp(0.0, self.max_scroll_x());
        self.scroll_y = self.scroll_y.clamp(0.0, self.max_scroll_y());
    }

    /// 从像素坐标获取单元格位置
    pub fn cell_at_pos(&self, x: f64, y: f64) -> Option<(usize, usize)> {
        // 检查是否在数据区域
        if y < self.header_height || x < self.row_number_width {
            return None;
        }

        let row = ((y - self.header_height + self.scroll_y) / self.row_height) as usize;
        if row >= self.total_rows {
            return None;
        }

        // 找列
        let target_x = x - self.row_number_width + self.scroll_x;
        let mut col_x = 0.0;
        for (ci, &w) in self.col_widths.iter().enumerate() {
            if target_x >= col_x && target_x < col_x + w {
                return Some((row, ci));
            }
            col_x += w;
        }

        None
    }

    /// 确保指定单元格可见（自动滚动）
    pub fn ensure_visible(&mut self, row: usize, col: usize) {
        // 垂直方向
        let row_top = row as f64 * self.row_height;
        let row_bottom = row_top + self.row_height;
        let view_top = self.scroll_y;
        let view_bottom = self.scroll_y + self.height - self.header_height;

        if row_top < view_top {
            self.scroll_y = row_top;
        } else if row_bottom > view_bottom {
            self.scroll_y = row_bottom - (self.height - self.header_height);
        }

        // 水平方向
        let mut col_left = 0.0f64;
        for &w in self.col_widths.iter().take(col) {
            col_left += w;
        }
        let col_right = col_left + self.col_width(col);
        let view_left = self.scroll_x;
        let view_right = self.scroll_x + self.width - self.row_number_width;

        if col_left < view_left {
            self.scroll_x = col_left;
        } else if col_right > view_right {
            self.scroll_x = col_right - (self.width - self.row_number_width);
        }

        self.clamp_scroll();
    }

    /// 设置默认列宽（根据数据自动计算）
    pub fn set_default_col_widths(&mut self, col_count: usize) {
        self.total_cols = col_count;
        self.col_widths = vec![120.0; col_count]; // 默认每列 120px
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visible_rows() {
        let mut vp = Viewport::new();
        vp.total_rows = 1_000_000;
        vp.scroll_y = 240.0; // 第 10 行开始

        assert_eq!(vp.first_visible_row(), 10);
        assert!(vp.visible_row_count() > 0);
    }

    #[test]
    fn test_cell_at_pos() {
        let mut vp = Viewport::new();
        vp.total_rows = 100;
        vp.total_cols = 5;
        vp.col_widths = vec![120.0; 5];

        // 点击第一个数据单元格区域
        let pos = vp.cell_at_pos(70.0, 30.0);
        assert_eq!(pos, Some((0, 0)));
    }

    #[test]
    fn test_ensure_visible() {
        let mut vp = Viewport::new();
        vp.total_rows = 1000;
        vp.total_cols = 10;
        vp.col_widths = vec![120.0; 10];

        vp.ensure_visible(500, 0);
        assert!(vp.scroll_y > 0.0);
    }
}
