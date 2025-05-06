use m_core::data::metadata::Platform;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub name: PlatformLimited,
    pub id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PlatformLimited {
    Unknown,
    Steam,
    DLSite,
}

impl Into<Platform> for PlatformLimited {
    fn into(self) -> Platform {
        match self {
            PlatformLimited::Unknown => Platform::Unknown,
            PlatformLimited::Steam => Platform::Steam,
            PlatformLimited::DLSite => Platform::DLSite,
        }
    }
}
