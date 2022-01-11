use serde::{ Serialize, Deserialize };
use crate::structs::EmailProtector;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct WCWPConfig {
    donate_url: String,
    source_url: String,
    contact_email: EmailProtector,
    privacy_email: EmailProtector,

    #[serde(skip_deserializing)]
    commit: &'static str,
}

impl Default for WCWPConfig {
    fn default() -> Self {
        Self {
            donate_url: Default::default(),
            source_url: Default::default(),
            contact_email: Default::default(),
            commit: env!("VERGEN_GIT_SHA_SHORT"),
            privacy_email: Default::default(),
        }
    }
}