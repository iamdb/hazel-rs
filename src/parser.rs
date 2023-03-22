use crate::item::Item;
use crate::{error::AppError, Result};
use chrono::Datelike;
use file_format::Kind;
use fs_extra::dir::CopyOptions;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use std::{fs::DirEntry, path::PathBuf};

#[derive(Parser)]
#[grammar = "pathspec.pest"]
struct TokenParser;

/// Replaces tokens in the Job's pattern.
pub fn parse_pattern(pattern: &str, item: &Item) -> Result<PathBuf> {
    process_items(pattern, |variable, component| {
        let tokens = variable.into_inner();

        process_variables(tokens, item, component)?;

        Ok(())
    })
}

/// Replaces variables in the pattern with values.
fn process_variables(mut tokens: Pairs<Rule>, item: &Item, component: &mut String) -> Result<()> {
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
                        let _thresh: i32 = t.as_str().parse().expect("failed to parse");
                    }
                }
            };
        }

        match token.as_str().into() {
            Token::Year => {
                if let Some(specifier) = specifier {
                    let time = item.datetime(specifier)?;
                    component.push_str(&time.year().to_string());
                }
            }
            Token::Month => {
                if let Some(specifier) = specifier {
                    let time = item.datetime(specifier)?;
                    component.push_str(&time.month().to_string());
                }
            }
            Token::Day => {
                if let Some(specifier) = specifier {
                    let time = item.datetime(specifier)?;
                    component.push_str(&time.day().to_string());
                }
            }
            Token::MimeType => {
                if item.is_file() {
                    let mime = mime_guess::from_path(item.path());
                    let first = mime.first_or_text_plain();

                    component.push_str(first.type_().as_str());
                    component.push('/');
                    component.push_str(first.subtype().as_str());
                }
            }
            Token::Extension => {
                if item.is_file() {
                    if let Some(ext) = item.path().extension() {
                        component.push_str(ext.to_str().unwrap());
                    }
                }
            }
            Token::Size => {
                if item.is_file() {
                    component.push_str(&item.size().to_string());
                }
            }
            Token::Kind => {
                if item.is_file() {
                    if let Some(kind) = item.kind() {
                        match kind {
                            Kind::Application => {
                                let guess = mime_guess::from_path(item.path());

                                if let Some(first) = guess.first() {
                                    for p in first.params() {
                                        println!("{p:?}");
                                    }
                                    component.push_str(first.type_().as_ref());
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
            }
            Token::Width => component.push_str(&item.width()?.to_string()),
            Token::_Height => todo!(),
            Token::Unknown => {}
        }
    }

    Ok(())
}

/// Process the items in the component of the path.
fn process_items<F>(pattern: &str, f: F) -> Result<PathBuf>
where
    F: Fn(Pair<Rule>, &mut String) -> Result<()> + Copy,
{
    let parsed = TokenParser::parse(Rule::path, pattern).expect("failed to parse pattern");
    let mut parsed_path = PathBuf::new();

    for p in parsed {
        if p.as_rule() == Rule::path {
            for p in p.into_inner() {
                if p.as_rule() == Rule::component {
                    let mut component = String::new();

                    for p in p.into_inner() {
                        if p.as_rule() == Rule::item {
                            for p in p.into_inner() {
                                if p.as_rule() == Rule::variable {
                                    f(p, &mut component)?;
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

fn _days_since_created(entry: &DirEntry) -> Result<f64> {
    let file_meta = entry.metadata()?;

    let created = file_meta.created()?;
    match created.elapsed() {
        Ok(elapsed) => Ok(elapsed.as_secs_f64() / 60. / 60. / 24.),
        Err(error) => Err(AppError::SystemTimeError { error }),
    }
}

#[derive(Debug)]
pub enum Token {
    Month,
    Year,
    Day,
    MimeType,
    Size,
    Extension,
    Width,
    _Height,
    Kind,
    Unknown,
}

impl From<&str> for Token {
    fn from(value: &str) -> Self {
        match value {
            "year" => Self::Year,
            "month" => Self::Month,
            "day" => Self::Day,
            "mime" => Self::MimeType,
            "extension" => Self::Extension,
            "size" => Self::Size,
            "kind" => Self::Kind,
            "width" => Self::Width,
            _ => Self::Unknown,
        }
    }
}

pub enum Specifier {
    Created,
    Modified,
    Accessed,
    _Subpath,
    _Type,
    Unknown,
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

pub enum Modifier {
    LowerCase,
    UpperCase,
    Names,
    Unkown,
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
