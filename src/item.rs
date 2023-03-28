use chrono::NaiveDateTime;
use file_format::{FileFormat, Kind};
use snafu::*;

use crate::{
    error::AppError,
    mediainfo::{self, MediaInfo, StreamKind},
    parser::Specifier,
    Result,
};
use std::{
    ffi::OsString,
    fs::{DirEntry, Metadata},
    os::unix::prelude::MetadataExt,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Snafu, Debug)]
enum ItemError {
    Failure,
}

pub struct Item<'i> {
    entry: &'i DirEntry,
    meta: Metadata,
    format: Option<FileFormat>,
    media_info: Option<MediaInfo>,
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
            media_info: None,
        })
    }

    pub(crate) fn format(&mut self) -> FileFormat {
        if let Some(f) = &self.format {
            f.to_owned()
        } else {
            let format =
                FileFormat::from_file(self.entry.path()).expect("failed to get file format");
            self.format = Some(format.clone());

            return format;
        }
    }

    pub(crate) fn media_info(&mut self) -> MediaInfo {
        if let Some(m) = &self.media_info {
            m.clone()
        } else {
            let mediainfo = MediaInfo::new();
            self.media_info = Some(mediainfo.clone());

            mediainfo
        }
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

    pub(crate) fn _move_to(&self, dest: PathBuf) -> Result<()> {
        if self.is_dir() {
            fs_extra::dir::move_dir(self.entry.path(), dest, &fs_extra::dir::CopyOptions::new())?;
        } else {
            fs_extra::file::move_file(
                self.entry.path(),
                dest,
                &fs_extra::file::CopyOptions::new(),
            )?;
        }

        Ok(())
    }

    pub(crate) fn _copy_to(&self, dest: PathBuf) -> Result<()> {
        if self.is_dir() {
            fs_extra::dir::copy(self.entry.path(), dest, &fs_extra::dir::CopyOptions::new())?;
        } else {
            fs_extra::file::copy(self.entry.path(), dest, &fs_extra::file::CopyOptions::new())?;
        }

        Ok(())
    }

    pub(crate) fn width(&self) -> Result<usize> {
        if let Some(format) = &self.format {
            match format.kind() {
                Kind::Image => {
                    let mi = MediaInfo::new();
                    if mi.open(self.entry.path().to_str().unwrap()) {
                        let width: usize = mi
                            .get_string(StreamKind::Image, "Width")
                            .parse()
                            .expect("failed to parse width");

                        mi.close();

                        Ok(width)
                    } else {
                        Err(AppError::UnkownToken)
                    }
                }
                Kind::Video => {
                    let mi = MediaInfo::new();
                    if mi.open(self.entry.path().to_str().unwrap()) {
                        let width: usize = mi
                            .get_string(StreamKind::Video, "Width")
                            .parse()
                            .expect("failed to parse width");

                        mi.close();

                        Ok(width)
                    } else {
                        Err(AppError::UnkownToken)
                    }
                }
                _ => Err(AppError::UnkownToken),
            }
        } else {
            Err(AppError::UnkownToken)
        }
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
