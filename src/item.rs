use chrono::NaiveDateTime;
use file_format::{FileFormat, Kind};
use snafu::*;

#[allow(unused_imports)]
use crate::{
    error::AppError,
    mediainfo::{self, MediaInfo, StreamKind},
    parser::Specifier,
};
use std::{
    ffi::OsString,
    fs::{DirEntry, Metadata},
    os::unix::prelude::MetadataExt,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Snafu, Debug)]
pub enum ItemError {
    #[snafu(display("Item failure"))]
    Failure,
    #[snafu(display("Failed to open item"))]
    Open,
    #[snafu(display("Failed to get item metadata"))]
    Metadata,
    #[snafu(display("Unknown specifier on item"))]
    UnknownSpecifier,
    #[snafu(display("Error getting item format"))]
    Format,
    #[snafu(display("Error converting time"))]
    ConvertTime,
    #[snafu(display("I/O error"))]
    IO,
    #[snafu(display("Error parsing regex"))]
    Regex,
    #[snafu(display("Failed to parse int"))]
    ParseIntError,
}

impl From<fs_extra::error::Error> for ItemError {
    fn from(_value: fs_extra::error::Error) -> Self {
        Self::IO
    }
}

impl From<std::io::Error> for ItemError {
    fn from(_value: std::io::Error) -> Self {
        Self::IO
    }
}

impl From<regex::Error> for ItemError {
    fn from(_value: regex::Error) -> Self {
        Self::Regex
    }
}

impl From<std::num::ParseIntError> for ItemError {
    fn from(_value: std::num::ParseIntError) -> Self {
        Self::ParseIntError
    }
}

type Result<T> = std::result::Result<T, ItemError>;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Item<'i> {
    entry: &'i DirEntry,
    meta: Metadata,
    format: Option<FileFormat>,
    media_info: Option<MediaInfo>,
}

#[allow(dead_code)]
impl<'i> Item<'i> {
    pub(crate) fn new(entry: &'i DirEntry) -> Result<Item<'i>> {
        let format = if entry.path().is_dir() {
            None
        } else if let Ok(f) = FileFormat::from_file(entry.path()) {
            Some(f)
        } else {
            None
        };

        if let Ok(meta) = entry.metadata() {
            Ok(Item {
                entry,
                meta,
                format,
                media_info: None,
            })
        } else {
            Err(ItemError::Metadata)
        }
    }

    pub(crate) fn format(&mut self) -> FileFormat {
        if let Some(f) = &self.format {
            f.to_owned()
        } else {
            let format =
                FileFormat::from_file(self.entry.path()).expect("failed to get file format");
            self.format = Some(format);

            format
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
        if let Ok(meta) = &self.meta.created() {
            systemtime_to_date(meta)
        } else {
            Err(ItemError::Metadata)
        }
    }

    pub(crate) fn modified(&self) -> Result<NaiveDateTime> {
        if let Ok(meta) = &self.meta.created() {
            systemtime_to_date(meta)
        } else {
            Err(ItemError::Metadata)
        }
    }

    pub(crate) fn accessed(&self) -> Result<NaiveDateTime> {
        if let Ok(meta) = &self.meta.created() {
            systemtime_to_date(meta)
        } else {
            Err(ItemError::Metadata)
        }
    }

    pub(crate) fn size(&self) -> u64 {
        self.meta.size()
    }

    pub(crate) fn datetime(&self, specifier: Specifier) -> Result<NaiveDateTime> {
        match specifier {
            Specifier::Created => self.created(),
            Specifier::Modified => self.modified(),
            Specifier::Accessed => self.accessed(),
            _ => Err(ItemError::UnknownSpecifier),
        }
    }

    pub(crate) fn kind(&self) -> Option<Kind> {
        self.format.as_ref().map(|f| f.kind())
    }

    pub(crate) fn move_to(&self, dest: PathBuf) -> Result<()> {
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
                        let width: usize = mi.get_string(StreamKind::Image, "Width").parse()?;

                        mi.close();

                        Ok(width)
                    } else {
                        Err(ItemError::Open)
                    }
                }
                Kind::Video => {
                    let mi = MediaInfo::new();
                    if mi.open(self.entry.path().to_str().unwrap()) {
                        let width: usize = mi.get_string(StreamKind::Video, "Width").parse()?;

                        mi.close();

                        Ok(width)
                    } else {
                        Err(ItemError::Open)
                    }
                }
                _ => Err(ItemError::Failure),
            }
        } else {
            Err(ItemError::Format)
        }
    }

    pub(crate) fn height(&self) -> Result<usize> {
        if let Some(format) = &self.format {
            match format.kind() {
                Kind::Image => {
                    let mi = MediaInfo::new();
                    if mi.open(self.entry.path().to_str().unwrap()) {
                        let height: usize = mi.get_string(StreamKind::Image, "Height").parse()?;

                        mi.close();

                        Ok(height)
                    } else {
                        Err(ItemError::Open)
                    }
                }
                Kind::Video => {
                    let mi = MediaInfo::new();
                    if mi.open(self.entry.path().to_str().unwrap()) {
                        let height: usize = mi.get_string(StreamKind::Video, "Height").parse()?;

                        mi.close();

                        Ok(height)
                    } else {
                        Err(ItemError::Open)
                    }
                }
                _ => Err(ItemError::Failure),
            }
        } else {
            Err(ItemError::Format)
        }
    }
}

/// Convert SystemTime into the DateTime it represents.
fn systemtime_to_date(time: &SystemTime) -> Result<NaiveDateTime> {
    if let Ok(time_since) = time.duration_since(UNIX_EPOCH) {
        if let Some(datetime) = NaiveDateTime::from_timestamp_millis(time_since.as_millis() as i64)
        {
            Ok(datetime)
        } else {
            Err(ItemError::ConvertTime)
        }
    } else {
        Err(ItemError::Failure)
    }
}
