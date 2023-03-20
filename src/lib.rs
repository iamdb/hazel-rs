use pest_derive::Parser;
use snafu::prelude::*;
use std::{io, time::SystemTimeError};

pub mod job;

#[derive(Snafu, Debug)]
pub enum AppError {
    PathExists,
    #[snafu(display("{error:?}"))]
    Watcher {
        error: notify::ErrorKind,
    },
    #[snafu(display("{error}"))]
    IO {
        error: io::ErrorKind,
    },
    #[snafu(display("There was an error."))]
    FsExtra {
        error: fs_extra::error::ErrorKind,
    },
    #[snafu(display("{error}"))]
    SystemTimeError {
        error: SystemTimeError,
    },
    #[snafu(display("{error}"))]
    RegexError {
        error: regex::Error,
    },
    #[snafu(display("Failed to convert time."))]
    ConvertTime,
    #[snafu(display("A token used in a pattern is unknown."))]
    UnkownToken,
    #[snafu(display("A specifier used in a pattern is unknown."))]
    UnkownSpecifier,
    #[snafu(display("A modifier used in a pattern is unknown."))]
    UnkownModifier,
}

impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        Self::IO {
            error: value.kind(),
        }
    }
}

impl From<notify::Error> for AppError {
    fn from(value: notify::Error) -> Self {
        Self::Watcher { error: value.kind }
    }
}

impl From<SystemTimeError> for AppError {
    fn from(value: SystemTimeError) -> Self {
        Self::SystemTimeError { error: value }
    }
}

impl From<regex::Error> for AppError {
    fn from(value: regex::Error) -> Self {
        Self::RegexError { error: value }
    }
}

impl From<fs_extra::error::Error> for AppError {
    fn from(value: fs_extra::error::Error) -> Self {
        Self::FsExtra { error: value.kind }
    }
}

#[derive(Parser)]
#[grammar = "pathspec.pest"]
struct TokenParser;

impl From<&str> for Token {
    fn from(value: &str) -> Self {
        match value {
            "year" => Self::Year,
            "month" => Self::Month,
            "mime" => Self::MimeType,
            "extension" => Self::Extension,
            "size" => Self::Size,
            "kind" => Self::Kind,
            _ => Self::Unknown,
        }
    }
}

impl From<&str> for Modifier {
    fn from(value: &str) -> Self {
        match value {
            "lowercase" => Self::LowerCase,
            "uppercase" => Self::UpperCase,
            "names" => Self::Names,
            _ => Self::Unkown,
        }
    }
}

impl From<&str> for Specifier {
    fn from(value: &str) -> Self {
        match value {
            "created" => Self::Created,
            "modified" => Self::Modified,
            "accessed" => Self::Accessed,
            _ => Self::Unknown,
        }
    }
}

pub enum GroupBy {
    DayOfWeek,
    Month,
    Year,
    SizeRange,
    DateRange,
}

pub enum Specifier {
    Created,
    Modified,
    Accessed,
    Unknown,
}

pub enum Modifier {
    LowerCase,
    UpperCase,
    Names,
    Unkown,
}

#[derive(Debug)]
pub enum Token {
    Month,
    Year,
    MimeType,
    Size,
    Extension,
    Width,
    Height,
    Kind,
    Unknown,
}

pub type Result<T> = std::result::Result<T, AppError>;
pub type Days = f64;

pub enum AgeType {
    Created,
    Accessed,
    Modified,
}
