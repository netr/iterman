use std::fmt::Debug;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum IterManError {
    #[error("invalid index {idx}, expected at most {limits}")]
    OutOfBounds { idx: usize, limits: usize },
}
