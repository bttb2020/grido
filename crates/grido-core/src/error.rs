//! 错误类型定义

use thiserror::Error;

/// 核心错误类型
#[derive(Error, Debug)]
pub enum CoreError {
    /// 行越界
    #[error("row {row} out of range (max {max})")]
    RowOutOfRange { row: usize, max: usize },
    /// 列越界
    #[error("column {col} out of range (max {max})")]
    ColumnOutOfRange { col: usize, max: usize },
}
