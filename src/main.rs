mod maid;
mod utils;
mod enums;

use crate::maid::housekeeping;
use crate::utils::connect_to_docker;

use std::env;
use std::str::FromStr;
use chrono::Local;
use cron::Schedule;
use log::{info, warn};
use crate::enums::ImagesPruneMode;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    colog::init();
    info!("Doing some checks before planning housekeeping duties...");

    // Ensure access to docker socket with bollard
    {
        let _ = connect_to_docker();
    }

    // Print a summary of the applied configuration
    {
        let mut summary = String::new();

        summary.push_str("Initializing docker-maid with the following duties:");
        summary.push_str(format!("\n\t- Containers update: `{}`", enums::ContainersUpdateMode::from_env()).as_ref());
        summary.push_str(format!("\n\t- Images prune: `{}`", ImagesPruneMode::from_env()).as_ref());
        summary.push_str(format!("\n\t- Build cache prune: `{}`", enums::BuildPruneMode::from_env()).as_ref());

        info!("{}", summary);
    }

    // Schedule initialization
    let schedule_string = env::var("MAID_SCHEDULE").unwrap_or_else(|_| {
        warn!("MAID_SCHEDULE not set, falling back to default schedule (every 6 hour): 0 0 */6 * * *");
        "0 0 */6 * * *".to_string()
    });
    let schedule = Schedule::from_str(&schedule_string).expect("MAID_SCHEDULE is not a valid cron expression");

    // Run housekeeping immediately if requested
    if env::var("MAID_RUN_ON_STARTUP").map(|v| v == "true").unwrap_or(false) {
        info!("MAID_RUN_ON_STARTUP is set to `true`, running housekeeping duties immediately.");
        housekeeping().await;
    }

    // Schedule housekeeping duties
    info!("House is quiet. Maid standing by.");
    loop {
        if let Some(next) = schedule.upcoming(Local).next() {
            info!("Next housekeeping round scheduled at {}", next.format("%H:%M:%S %d-%m-%Y"));

            if let Ok(duration) = next.signed_duration_since(Local::now()).to_std() {
                tokio::time::sleep(duration).await;
                housekeeping().await;
            }
        }
    }
}
