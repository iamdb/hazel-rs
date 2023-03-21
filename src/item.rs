use chrono::NaiveDateTime;
use file_format::{FileFormat, Kind};

use crate::{error::AppError, parser::Specifier, Result};
use std::{
    ffi::OsString,
    fs::{DirEntry, Metadata},
    os::unix::prelude::MetadataExt,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Item<'i> {
    entry: &'i DirEntry,
    meta: Metadata,
    format: Option<FileFormat>,
}

impl<'i> Item<'i> {
    pub(crate) fn new(entry: &'i DirEntry) -> Result<Item<'i>> {
        let format = if entry.path().is_dir() {
            None
        } else if let Ok(f) = FileFormat::from_file(entry.path()) {
            Some(f)
        } else {
            None
        };

        Ok(Item {
            entry,
            meta: entry.metadata()?,
            format,
        })
    }

    pub(crate) fn is_file(&self) -> bool {
        self.meta.is_file()
    }

    pub(crate) fn is_dir(&self) -> bool {
        self.meta.is_dir()
    }

    pub(crate) fn path(&self) -> PathBuf {
        self.entry.path()
    }

    pub(crate) fn file_name(&self) -> Option<OsString> {
        if self.is_file() {
            Some(self.entry.file_name())
        } else {
            None
        }
    }

    pub(crate) fn dir_name(&self) -> Option<String> {
        if self.is_dir() {
            Some(
                self.entry
                    .path()
                    .components()
                    .last()
                    .unwrap()
                    .as_os_str()
                    .to_str()?
                    .to_string(),
            )
        } else {
            None
        }
    }

    pub(crate) fn created(&self) -> Result<NaiveDateTime> {
        systemtime_to_date(&self.meta.created()?)
    }

    pub(crate) fn modified(&self) -> Result<NaiveDateTime> {
        systemtime_to_date(&self.meta.modified()?)
    }

    pub(crate) fn accessed(&self) -> Result<NaiveDateTime> {
        systemtime_to_date(&self.meta.accessed()?)
    }

    pub(crate) fn size(&self) -> u64 {
        self.meta.size()
    }

    pub(crate) fn datetime(&self, specifier: Specifier) -> Result<NaiveDateTime> {
        match specifier {
            Specifier::Created => self.created(),
            Specifier::Modified => self.modified(),
            Specifier::Accessed => self.accessed(),
            _ => Err(AppError::UnkownSpecifier),
        }
    }

    pub(crate) fn kind(&self) -> Option<Kind> {
        self.format.as_ref().map(|f| f.kind())
    }
}

/// Convert SystemTime into the DateTime it represents.
fn systemtime_to_date(time: &SystemTime) -> Result<NaiveDateTime> {
    let time_since = time.duration_since(UNIX_EPOCH)?;

    if let Some(datetime) = NaiveDateTime::from_timestamp_millis(time_since.as_millis() as i64) {
        Ok(datetime)
    } else {
        Err(AppError::ConvertTime)
    }
}
