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
    SystemTimeError {
        error: SystemTimeError,
    },
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

pub type Result<T> = std::result::Result<T, AppError>;
pub type Days = f64;
