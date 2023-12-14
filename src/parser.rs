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
pub fn parse_pattern(pattern: &str, item: &mut Item) -> Result<PathBuf> {
    process_items(pattern, item, |variable, component, item| {
        let mut tokens = variable.into_inner();

        process_variables(&mut tokens, item, component)?;

        Ok(())
    })
}

/// Replaces variables in the pattern with values.
fn process_variables(
    tokens: &mut Pairs<Rule>,
    item: &mut Item,
    component: &mut Vec<String>,
) -> Result<()> {
    // Get the first of 3 fields in a variable (token:specifier:modifier)
    if let Some(token) = tokens.next() {
        let mut specifier: Option<Specifier> = None;
        let mut _modifier: Option<Modifier> = None;
        let mut thresholds: Vec<Pair<'_, Rule>> = Vec::new();

        // Check the next two fields for either specifier or modifier.
        // Both types are checked because they are optional.
        for _ in 0..2 {
            if let Some(token) = tokens.next() {
                if token.as_rule() == Rule::specifier {
                    specifier = Some(token.as_str().into());
                } else if token.as_rule() == Rule::modifier {
                    _modifier = Some(token.as_str().into());
                } else if token.as_rule() == Rule::thresholds {
                    for t in token.into_inner() {
                        if let Rule::threshold = t.as_rule() {
                            thresholds.push(t);
                        };
                    }
                }
            };
        }

        match token.as_str().into() {
            Token::Year => {
                if let Some(specifier) = specifier {
                    let time = item.datetime(specifier)?;
                    component.push(time.year().to_string());
                }
            }
            Token::Month => {
                if let Some(specifier) = specifier {
                    let time = item.datetime(specifier)?;
                    component.push(time.month().to_string());
                }
            }
            Token::Day => {
                if let Some(specifier) = specifier {
                    let time = item.datetime(specifier)?;
                    component.push(time.day().to_string());
                }
            }
            Token::MimeType => {
                if item.is_file() {
                    let mime = mime_guess::from_path(item.path());
                    let first = mime.first_or_text_plain();

                    component.push(first.type_().to_string());
                    component.push("/".to_string());
                    component.push(first.subtype().to_string());
                }
            }
            Token::Extension => {
                if item.is_file() {
                    if let Some(ext) = item.path().extension() {
                        component.push(ext.to_string_lossy().to_string());
                    }
                }
            }
            Token::Size => {
                if item.is_file() {
                    let mut c = "".to_string();

                    for t in thresholds {
                        let name = t.as_str();

                        let mut over_under = "";
                        let mut mult = 0;
                        let mut thresh = 0_u32;

                        for t in t.into_inner() {
                            match t.as_rule() {
                                Rule::gt => {
                                    over_under = "over";
                                }
                                Rule::lt => {
                                    over_under = "under";
                                }
                                Rule::threshold_size => {
                                    mult = match t.as_str() {
                                        "B" => 1,
                                        "K" => 1024,
                                        "M" => 1024 * 1024,
                                        "G" => 1024 * 1024 * 1024,
                                        _ => 0,
                                    };
                                }
                                Rule::threshold_amount => {
                                    thresh = t.as_str().parse().expect("error parsing");
                                }
                                _ => {}
                            };

                            let size = item.size() as u32;

                            match size.cmp(&(thresh * mult)) {
                                std::cmp::Ordering::Less => {
                                    if over_under == "under" {
                                        c = name.to_string();
                                        break;
                                    }
                                }
                                std::cmp::Ordering::Equal => {
                                    c = name.to_string();
                                }
                                std::cmp::Ordering::Greater => {
                                    if over_under == "over" {
                                        c = name.to_string();
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !c.is_empty() {
                        component.push(c);
                    }
                } else {
                    println!("**** DIR SIZE: {}", item.size());
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
                                    component.push(first.type_().to_string());
                                }
                            }
                            Kind::Audio => component.push("audio".to_string()),
                            Kind::Font => component.push("font".to_string()),
                            Kind::Image => component.push("image".to_string()),
                            Kind::Model => component.push("model".to_string()),
                            Kind::Text => component.push("text".to_string()),
                            Kind::Video => component.push("video".to_string()),
                            Kind::Archive => component.push("archive".to_string()),
                            Kind::Book => component.push("book".to_string()),
                            Kind::Certificate => component.push("certificate".to_string()),
                            Kind::Compression => component.push("compression".to_string()),
                            Kind::Disk => component.push("disk".to_string()),
                            Kind::Document => component.push("document".to_string()),
                            Kind::Executable => component.push("executable".to_string()),
                            Kind::Geospatial => component.push("geospatial".to_string()),
                            Kind::Package => component.push("package".to_string()),
                            Kind::Playlist => component.push("playlist".to_string()),
                            Kind::Rom => component.push("rom".to_string()),
                            Kind::Subtitle => component.push("subtitle".to_string()),
                        }
                    }
                } else {
                    component.push("directory".to_string());
                }
            }
            Token::Width => {
                if let Ok(width) = &item.width() {
                    component.push(width.to_string())
                }
            }
            Token::Height => {
                if let Ok(height) = &item.height() {
                    component.push(height.to_string())
                }
            }
            Token::Unknown => {}
        }
    }

    Ok(())
}

/// Process the items in the component of the path.
fn process_items<F>(pattern: &str, item: &mut Item, f: F) -> Result<PathBuf>
where
    F: Fn(Pair<Rule>, &mut Vec<String>, &mut Item) -> Result<()> + Copy,
{
    let parsed = TokenParser::parse(Rule::path, pattern).expect("failed to parse pattern");
    let mut parsed_path = PathBuf::new();

    for p in parsed {
        if p.as_rule() == Rule::path {
            for p in p.into_inner() {
                if p.as_rule() == Rule::component {
                    let mut component = Vec::new();
                    let c = p.into_inner();
                    let count = c.clone().count();

                    for p in c {
                        if p.as_rule() == Rule::variable {
                            f(p, &mut component, item)?;
                        } else if p.as_rule() == Rule::text {
                            component.push(p.as_str().to_string());
                        }
                    }

                    if !component.is_empty() && component.len() == count {
                        parsed_path.push(component.join(""));
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
    Height,
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
            "height" => Self::Height,
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
