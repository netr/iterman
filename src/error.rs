use std::fmt::Debug;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum IterManError {
    #[error("invalid line_index: {line_index}, expected at most {max_len} bytes")]
    MemoryOutOfBounds { line_index: usize, max_len: usize },
    #[error(
        "invalid line_index: {line_index} and bytes_offset: {bytes_offset}, expected at most {max_len} bytes"
    )]
    StreamOutOfBounds {
        line_index: usize,
        bytes_offset: usize,
        max_len: usize,
    },
}
