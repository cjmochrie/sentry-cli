//! Provides support for sending events to Sentry.
use std::time::SystemTime;
use std::collections::HashMap;
use std::process::Command;

use prelude::*;
use utils::to_timestamp;


/// Represents a Sentry event.
#[derive(Serialize)]
pub struct Event {
    pub tags: HashMap<String, String>,
    pub extra: HashMap<String, String>,
    pub level: String,
    #[serde(skip_serializing_if="Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub message: Option<String>,
    pub platform: String,
    pub timestamp: f64,
    #[serde(skip_serializing_if="Option::is_none")]
    pub server_name: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub release: Option<String>,
}

fn get_server_name() -> Result<String> {
    let p = Command::new("uname").arg("-n").output()?;
    Ok(String::from_utf8(p.stdout)?.trim().to_owned())
}

impl Event {
    pub fn new() -> Event {
        Event {
            tags: HashMap::new(),
            extra: HashMap::new(),
            level: "error".into(),
            fingerprint: None,
            message: None,
            platform: "other".into(),
            timestamp: to_timestamp(SystemTime::now()),
            server_name: get_server_name().ok(),
            release: None,
        }
    }
}
