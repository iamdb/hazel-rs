use crate::error::AppError;

mod error;
mod item;
pub mod job;
mod parser;

pub type Result<T> = std::result::Result<T, AppError>;
