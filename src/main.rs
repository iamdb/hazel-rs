use hazel_rs::{job::Jobs, Result};

fn main() -> Result<()> {
    let file = std::fs::read("jobs.sample.yaml")?;
    let job_list: Jobs = serde_yaml::from_slice(&file).expect("failed to parse yaml");

    for job in job_list.jobs {
        job.run()?;
    }

    Ok(())
}
