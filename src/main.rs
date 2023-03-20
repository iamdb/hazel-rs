use hazel_rs::{job::Job, Result};

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::{path::Path, sync::mpsc};

// group by month created
// group by year created
//

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let path = args.next().unwrap();

    // let thresholds = args
    //     .next()
    //     .unwrap()
    //     .split(',')
    //     .map(|a| {
    //         let f: i32 = a.parse().expect("failed to parse threshold");

    //         f as f64
    //     })
    //     .collect::<Vec<f64>>();

    let job = Job::new(
        &path,
        None,
        "/test/{month:created}-{year:created}/{mime}",
        false,
    )?;

    job.run()?;

    Ok(())
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
