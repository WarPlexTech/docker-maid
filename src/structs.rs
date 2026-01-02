/// Should docker-maid start an housekeeping round on startup?
pub struct RunOnStartup;
impl RunOnStartup {
    pub fn from_env() -> bool {
        let value = std::env::var("MAID_RUN_ON_STARTUP").unwrap_or_else(|_| "false".to_string());
        value == "true"
    }
}

/// Should docker-maid perform self-update at the end of an housekeeping round?
pub struct SelfUpdate;
impl SelfUpdate {
    pub fn from_env() -> bool {
        let value = std::env::var("MAID_DUTY_SELF_UPDATE").unwrap_or_else(|_| "true".to_string());
        value == "true"
    }
}