use hazel_rs::{AppError, Days, Result};

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    fs::{self, DirEntry},
    path::Path,
    sync::mpsc,
};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let path = args.next().unwrap();

    let thresholds = args
        .next()
        .unwrap()
        .split(',')
        .map(|a| {
            let f: i32 = a.parse().expect("failed to parse threshold");

            f as f64
        })
        .collect::<Vec<f64>>();

    process_dir(&path, |entry| group_by_days(entry, &thresholds))?;

    Ok(())
}

fn group_by_days(entry: &DirEntry, thresholds: &[f64]) -> Result<()> {
    let days_since_created = days_since_created(entry)?;
    let source = entry.path();
    let destination = format!(
        "{}/{days_since_created}days",
        source
            .parent()
            .expect("directory ahs no parent")
            .to_string_lossy()
    );

    for t in thresholds.windows(2) {
        let start = t[0];
        let end = t[1];

        if (start..end).contains(&days_since_created) {
            create_path(&destination)?;
            fs::rename(
                &source,
                format!("{destination}/{}", entry.file_name().to_string_lossy()),
            )?;
        }
    }

    Ok(())
}

fn days_since_created(entry: &DirEntry) -> Result<Days> {
    let file_meta = entry.metadata()?;

    let created = file_meta.created()?;
    match created.elapsed() {
        Ok(elapsed) => Ok(elapsed.as_secs_f64() / 60. / 60. / 24.),
        Err(error) => Err(AppError::SystemTimeError { error }),
    }
}

fn _watch(path: &str) -> Result<()> {
    let (tx, rx) = mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(Path::new(path), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => println!("changed: {event:?}"),
            Err(e) => println!("error: {e:?}"),
        }
    }

    Ok(())
}

fn process_dir<F>(path: &str, f: F) -> Result<()>
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
