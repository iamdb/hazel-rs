use crate::{AppError, Days, Modifier, Result, Rule, Specifier, Token, TokenParser};
use chrono::{Datelike, NaiveDateTime};
use pest::Parser;
use std::{
    fs::{self, DirEntry},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Job<'d> {
    source: &'d str,
    destination: &'d str,
    pattern: &'d str,
}

impl<'d> Job<'d> {
    /// Creates a new Job
    pub fn new(source: &'d str, destination: Option<&'d str>, pattern: &'d str) -> Result<Self> {
        let destination = if let Some(dest) = destination {
            create_path(dest)?;
            dest
        } else {
            source
        };

        Ok(Self {
            destination,
            pattern,
            source,
        })
    }

    /// Runs a Job
    pub fn run(&self) -> Result<()> {
        process_source(self.source, |entry| {
            let dest = parse_pattern(self.pattern, entry)?;

            println!("{}/{}", self.destination, dest.to_str().unwrap());

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
                if p.as_rule() == Rule::variable {
                    let mut tokens = p.into_inner();

                    // Get the first of 3 fields in a variable (token:specifier:modifier)
                    if let Some(token) = tokens.next() {
                        let mut specifier = None;
                        let mut modifier = None;

                        // Check the next two fields for either specifier or modifier.
                        // Both types are checked against because a specifier isn't required.
                        for _ in 0..1 {
                            if let Some(token) = tokens.next() {
                                if token.as_rule() == Rule::specifier {
                                    specifier = Some(token.as_str().into());
                                } else if token.as_rule() == Rule::modifier {
                                    modifier = Some(token.as_str().into())
                                }
                            };
                        }
                        handle_tokens(
                            token.as_str().into(),
                            &mut parsed_path,
                            entry,
                            specifier,
                            modifier,
                        )?;
                    }
                } else if p.as_rule() == Rule::text {
                    parsed_path.push(p.as_str());
                }
            }
        }
    }

    Ok(parsed_path)
}

fn handle_tokens(
    token: Token,
    path: &mut PathBuf,
    entry: &DirEntry,
    specifier: Option<Specifier>,
    _modifier: Option<Modifier>,
) -> Result<()> {
    match token {
        Token::Year => {
            if let Some(specifier) = specifier {
                let time = get_entry_time(entry, specifier)?;
                path.push(time.year().to_string());
                Ok(())
            } else {
                Err(AppError::UnkownSpecifier)
            }
        }
        Token::Month => {
            if let Some(specifier) = specifier {
                let time = get_entry_time(entry, specifier)?;
                path.push(time.month().to_string());
                Ok(())
            } else {
                Err(AppError::UnkownSpecifier)
            }
        }
        Token::MimeType => {
            if let Ok(Some(kind)) = infer::get_from_path(entry.path()) {
                path.push(kind.mime_type());
            }
            Ok(())
        }
        Token::Unknown => Err(AppError::UnkownToken),
    }
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
            fs::rename(
                &source,
                format!("{destination}/{}", entry.file_name().to_string_lossy()),
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

fn process_source<F>(path: &str, f: F) -> Result<()>
where
    F: FnOnce(&DirEntry) -> Result<()> + Copy,
{
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        f(&entry)?;
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
