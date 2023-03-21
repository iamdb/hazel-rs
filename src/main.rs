use hazel_rs::{job::Job, Result};

fn main() -> Result<()> {
    let jobs = Job::from_file("jobs.sample.yaml")?;

    for job in jobs {
        job.run()?;
    }

    Ok(())
}
