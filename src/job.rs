use crate::{item::Item, parser, AppError, Result};
use serde::{Deserialize, Serialize};
use std::{ffi::OsString, fs, path::Path};

/// A Job defines the renaming pattern to apply to the source directory.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Job {
    name: String,
    source: String,
    destination: String,
    pattern: String,
    recursive: Option<bool>,
    watch: Option<bool>,
}

/// A list of Job definitions
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Jobs {
    pub jobs: Vec<Job>,
}

impl IntoIterator for Jobs {
    type Item = Job;

    type IntoIter = std::vec::IntoIter<Job>;

    fn into_iter(self) -> Self::IntoIter {
        self.jobs.into_iter()
    }
}

impl Job {
    /// Creates a new Job
    pub fn new<'d>(
        name: &'d str,
        source: &'d str,
        destination: Option<&'d str>,
        pattern: &'d str,
        recursive: bool,
        watch: bool,
    ) -> Result<Self> {
        let destination = if let Some(dest) = destination {
            create_path(dest)?;
            dest
        } else {
            source
        };

        Ok(Self {
            name: name.to_string(),
            destination: destination.to_string(),
            recursive: Some(recursive),
            watch: Some(watch),
            pattern: pattern.to_string(),
            source: source.to_string(),
        })
    }

    pub fn from_file(path: &str) -> Result<Jobs> {
        let file = std::fs::read(path)?;
        let job_list: Jobs = serde_yaml::from_slice(&file).expect("failed to parse yaml");

        Ok(job_list)
    }

    /// Runs a Job
    pub fn run(&self) -> Result<()> {
        process_source(&self.source, self.recursive.unwrap_or(false), |item| {
            let dest = parser::parse_pattern(&self.pattern, item)?;

            // TODO: MISSING DIRECTORY NAME FROM OUTPUT WHEN ITEM IS DIRECTORY
            if item.is_file() {
                println!(
                    "file:\t{}/{}/{}",
                    self.destination,
                    dest.to_str().unwrap(),
                    item.file_name()
                        .unwrap_or(OsString::new())
                        .to_string_lossy()
                );
            } else if item.is_dir() {
                println!(
                    "dir:\t{}/{}/{}",
                    self.destination,
                    dest.to_str().unwrap(),
                    item.dir_name().unwrap_or(String::new())
                );
            }

            Ok(())
        })?;

        Ok(())
    }
}

/// Read the list of entries from the source directory and process each one.
fn process_source<F>(path: &str, recursive: bool, f: F) -> Result<()>
where
    F: FnOnce(&Item) -> Result<()> + Copy,
{
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let item = Item::new(&entry)?;

        if recursive {
            if item.is_file() {
                f(&item)?;
            } else if item.is_dir() {
                process_source(
                    item.path()
                        .to_str()
                        .expect("failed to convert item path to string"),
                    recursive,
                    f,
                )?;
            }
        } else {
            f(&item)?;
        }
    }

    Ok(())
}

/// Create a directory
fn create_path(path: &str) -> Result<()> {
    if !Path::new(path).exists() {
        println!("creating directory {path}");
        match fs::create_dir(path) {
            Ok(_) => Ok(()),
            Err(err) => Err(AppError::IO { error: err.kind() }),
        }
    } else {
        Ok(())
    }
}
