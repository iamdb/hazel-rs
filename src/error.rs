use snafu::prelude::*;
use std::{io, time::SystemTimeError};

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
