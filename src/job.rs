use crate::{item::Item, parser, AppError, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

/// A Job defines the renaming pattern to apply to the source directory.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Job {
    name: String,
    source: String,
    destination: Option<String>,
    pattern: String,
    recursive: Option<bool>,
    watch: Option<bool>,
}

/// A list of Job definitions
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Jobs {
    jobs: Vec<Job>,
}

impl Jobs {
    pub fn run_all(&self) -> Result<()> {
        for job in &self.jobs {
            job.run()?;
        }

        Ok(())
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
            destination: Some(destination.to_string()),
            recursive: Some(recursive),
            watch: Some(watch),
            pattern: pattern.to_string(),
            source: source.to_string(),
        })
    }

    pub fn from_file(path: &str) -> Result<Jobs> {
        let file = std::fs::read(path)?;
        let job_list: Jobs = serde_yaml::from_slice(&file)?;

        Ok(job_list)
    }

    /// Runs a Job
    pub fn run(&self) -> Result<()> {
        let base_dest = self.destination.as_ref().unwrap_or(&self.source);

        process_source(&self.source, self.recursive.unwrap_or_default(), |item| {
            let pattern = parser::parse_pattern(&self.pattern, item)?;

            let mut item_name = "".to_string();

            if item.is_file() {
                item_name = item
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
            } else if item.is_dir() {
                item_name = item.dir_name().unwrap_or_default();
            }

            let dest = format!("{}/{}/{}", base_dest, pattern.to_str().unwrap(), item_name);

            //item.move_to(PathBuf::from_str(&dest).unwrap())?;

            println!("{}/{item_name}\n\t\t{dest}", self.source);

            Ok(())
        })?;

        Ok(())
    }
}

/// Read the list of entries from the source directory and process each one.
fn process_source<F>(path: &str, recursive: bool, f: F) -> Result<()>
where
    F: FnOnce(&mut Item) -> Result<()> + Copy,
{
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let mut item = Item::new(&entry)?;

        if recursive {
            if item.is_file() {
                f(&mut item)?;
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
            f(&mut item)?;
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
