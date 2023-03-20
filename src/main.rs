use hazel_rs::{job::Job, Result};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let path = args.next().unwrap();

    let job = Job::new(&path, None, "/{year:created}/{month:created}/", false)?;

    job.run()?;

    Ok(())
}
