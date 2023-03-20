use crate::{parser, AppError, Result};
use std::{
    fs::{self, DirEntry},
    path::Path,
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
            let dest = parser::parse_pattern(self.pattern, entry)?;

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
