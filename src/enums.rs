use std::fmt::{Display, Formatter};

// region UpdateMode
/// Represents the update mode of docker-maid.
/// - `None` Dont check for updates
/// - `Notify` [Not implemented yet] Check for new digests, but only notify the user
/// - `Update` Update containers to the latest digest of their images/tags by recreating them
/// - `Label` [Not implemented yet] Label driven, will check labels of containers to know what to do with them.
#[derive(Debug, Clone, Copy, PartialEq)]

pub enum ContainersUpdateMode {
    None,
    Notify,
    Update,
    Label
}

impl ContainersUpdateMode {
    /// Reads the `MAID_DUTY_CONTAINERS_UPDATES` environment variable and returns the corresponding update mode.
    /// Defaults to `None` if the variable is not set or has an invalid value.
    pub fn from_env() -> Self {
        match std::env::var("MAID_DUTY_CONTAINERS_UPDATES")
            .map(|v| v.to_lowercase())
            .as_deref()
        {
            Ok("label") => Self::Label,
            Ok("update") => Self::Update,
            Ok("notify") => Self::Notify,
            _ => Self::None
        }
    }
}

impl Display for ContainersUpdateMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Notify => write!(f, "Notify"),
            Self::Update => write!(f, "Update"),
            Self::Label => write!(f, "Label"),
        }
    }
}
// endregion

// region ImagesPruneMode
/// Represents the images prune mode of docker-maid.
/// - `None` No images cleanup
/// - `Dangling` Prune dangling images only
/// - `All` Prune all unused images
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImagesPruneMode {
    None,
    Dangling,
    All
}

impl ImagesPruneMode {
    /// Reads the `MAID_DUTY_PRUNE_IMAGES` environment variable and returns the corresponding prune mode.
    /// Defaults to `None` if the variable is not set or has an invalid value.
    pub fn from_env() -> Self {
        match std::env::var("MAID_DUTY_PRUNE_IMAGES")
            .map(|v| v.to_lowercase())
            .as_deref()
        {
            Ok("all") => Self::All,
            Ok("dangling") => Self::Dangling,
            _ => Self::None
        }
    }
}

impl Display for ImagesPruneMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Dangling => write!(f, "Dangling"),
            Self::All => write!(f, "All"),
        }
    }
}
// endregion