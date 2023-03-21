use crate::error::AppError;

pub mod error;
pub mod job;
pub mod parser;

pub type Result<T> = std::result::Result<T, AppError>;
