use crate::{AppError, Days, Modifier, Result, Rule, Specifier, Token, TokenParser};
use chrono::{Datelike, NaiveDateTime};
use file_format::{FileFormat, Kind};
use fs_extra::dir::CopyOptions;
use pest::Parser;
use std::{
    fs::{self, DirEntry},
    os::unix::prelude::MetadataExt,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Job<'d> {
    source: &'d str,
    destination: &'d str,
    pattern: &'d str,
    recursive: bool,
}

impl<'d> Job<'d> {
    /// Creates a new Job
    pub fn new(
        source: &'d str,
        destination: Option<&'d str>,
        pattern: &'d str,
        recursive: bool,
    ) -> Result<Self> {
        let destination = if let Some(dest) = destination {
            create_path(dest)?;
            dest
        } else {
            source
        };

        Ok(Self {
            destination,
            recursive,
            pattern,
            source,
        })
    }

    /// Runs a Job
    pub fn run(&self) -> Result<()> {
        process_source(self.source, self.recursive, |entry| {
            let dest = parse_pattern(self.pattern, entry)?;

            println!(
                "{}/{}/{}",
                self.destination,
                dest.to_str().unwrap(),
                entry.file_name().to_str().unwrap()
            );

            Ok(())
        })?;

        Ok(())
    }
}

/// Replaces tokens in the Job's pattern.
fn parse_pattern(pattern: &str, entry: &DirEntry) -> Result<PathBuf> {
    let mut parsed_path = PathBuf::new();

    let parsed = TokenParser::parse(Rule::path, pattern).expect("failed to parse pattern");

    for p in parsed {
        if p.as_rule() == Rule::path {
            for p in p.into_inner() {
                if p.as_rule() == Rule::component {
                    let mut component = String::new();

                    for p in p.into_inner() {
                        if p.as_rule() == Rule::item {
                            for p in p.into_inner() {
                                if p.as_rule() == Rule::variable {
                                    let mut tokens = p.into_inner();

                                    // Get the first of 3 fields in a variable (token:specifier:modifier)
                                    if let Some(token) = tokens.next() {
                                        let mut specifier: Option<Specifier> = None;
                                        let mut _modifier: Option<Modifier> = None;

                                        // Check the next two fields for either specifier or modifier.
                                        // Both types are checked because they are optional.
                                        for _ in 0..2 {
                                            if let Some(token) = tokens.next() {
                                                if token.as_rule() == Rule::specifier {
                                                    specifier = Some(token.as_str().into());
                                                } else if token.as_rule() == Rule::modifier {
                                                    _modifier = Some(token.as_str().into())
                                                } else if token.as_rule() == Rule::thresholds {
                                                    for t in token.into_inner() {
                                                        let _thresh: i32 = t
                                                            .as_str()
                                                            .parse()
                                                            .expect("failed to parse");
                                                    }
                                                }
                                            };
                                        }

                                        match token.as_str().into() {
                                            Token::Year => {
                                                if let Some(specifier) = specifier {
                                                    let time = get_entry_time(entry, specifier)?;
                                                    component.push_str(&time.year().to_string());
                                                }
                                            }
                                            Token::Month => {
                                                if let Some(specifier) = specifier {
                                                    let time = get_entry_time(entry, specifier)?;
                                                    component.push_str(&time.month().to_string());
                                                }
                                            }
                                            Token::MimeType => {
                                                if entry.path().is_file() {
                                                    let mime = mime_guess::from_path(entry.path());
                                                    let first = mime.first_or_text_plain();

                                                    component.push_str(first.type_().as_str());
                                                    component.push('/');
                                                    component.push_str(first.subtype().as_str());
                                                }
                                            }
                                            Token::Extension => {
                                                if entry.path().is_file() {
                                                    if let Some(ext) = entry.path().extension() {
                                                        component.push_str(ext.to_str().unwrap());
                                                    }
                                                }
                                            }
                                            Token::Size => {
                                                if entry.path().is_file() {
                                                    let meta = entry.metadata()?;
                                                    component.push_str(&meta.size().to_string());
                                                }
                                            }
                                            Token::Kind => {
                                                if entry.path().is_file() {
                                                    let format =
                                                        FileFormat::from_file(entry.path())?;

                                                    match format.kind() {
                                                        Kind::Application => {
                                                            let guess =
                                                                mime_guess::from_path(entry.path());

                                                            if let Some(first) = guess.first() {
                                                                for p in first.params() {
                                                                    println!("{p:?}");
                                                                }
                                                                component.push_str(
                                                                    first.type_().as_ref(),
                                                                );
                                                            }
                                                        }
                                                        Kind::Audio => component.push_str("audio"),
                                                        Kind::Font => component.push_str("font"),
                                                        Kind::Image => component.push_str("image"),
                                                        Kind::Model => component.push_str("model"),
                                                        Kind::Text => component.push_str("text"),
                                                        Kind::Video => component.push_str("video"),
                                                    }
                                                }
                                            }
                                            Token::Width => todo!(),
                                            Token::Height => todo!(),
                                            Token::Unknown => {}
                                        }
                                    }
                                } else if p.as_rule() == Rule::text {
                                    component.push_str(p.as_str());
                                }
                            }
                        }
                    }

                    if !component.is_empty() {
                        parsed_path.push(component);
                    }
                }
            }
        }
    }

    Ok(parsed_path)
}

fn get_entry_time(entry: &DirEntry, specifier: Specifier) -> Result<NaiveDateTime> {
    let meta = entry.metadata()?;

    match specifier {
        Specifier::Created => {
            let created = meta.created()?;
            systemtime_to_date(&created)
        }
        Specifier::Modified => {
            let modified = meta.modified()?;
            systemtime_to_date(&modified)
        }
        Specifier::Accessed => {
            let accessed = meta.accessed()?;
            systemtime_to_date(&accessed)
        }
        Specifier::Unknown => Err(AppError::UnkownSpecifier),
    }
}

fn systemtime_to_date(time: &SystemTime) -> Result<NaiveDateTime> {
    let time_since = time.duration_since(UNIX_EPOCH)?;

    if let Some(datetime) = NaiveDateTime::from_timestamp_millis(time_since.as_millis() as i64) {
        Ok(datetime)
    } else {
        Err(AppError::ConvertTime)
    }
}

fn _group_by_date_range(entry: &DirEntry, destination: &str, thresholds: &[f64]) -> Result<()> {
    let days_since_created = _days_since_created(entry)?;
    let source = entry.path();

    for t in thresholds.windows(2) {
        let start = t[0];
        let end = t[1];

        if (start..end).contains(&days_since_created) {
            fs_extra::dir::move_dir(
                &source,
                format!("{destination}/{}", entry.file_name().to_string_lossy()),
                &CopyOptions::new(),
            )?;
        }
    }

    Ok(())
}

fn _days_since_created(entry: &DirEntry) -> Result<Days> {
    let file_meta = entry.metadata()?;

    let created = file_meta.created()?;
    match created.elapsed() {
        Ok(elapsed) => Ok(elapsed.as_secs_f64() / 60. / 60. / 24.),
        Err(error) => Err(AppError::SystemTimeError { error }),
    }
}

fn process_source<F>(path: &str, recursive: bool, f: F) -> Result<()>
where
    F: FnOnce(&DirEntry) -> Result<()> + Copy,
{
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let meta = entry.metadata()?;

        if recursive {
            if meta.is_file() {
                f(&entry)?;
            } else if meta.is_dir() {
                process_source(entry.path().as_os_str().to_str().unwrap(), recursive, f)?;
            }
        } else {
            f(&entry)?;
        }
    }

    Ok(())
}

fn create_path(path: &str) -> Result<()> {
    if !Path::new(path).exists() {
        println!("creating folder {path}");
        match fs::create_dir(path) {
            Ok(_) => Ok(()),
            Err(err) => Err(AppError::IO { error: err.kind() }),
        }
    } else {
        Ok(())
    }
}
