pub const APP_ID: Option<&str> = option_env!("APP_ID");
pub const RESOURCE_FILE: Option<&str> = option_env!("RESOURCE_FILE");

pub fn app_id() -> &'static str {
    APP_ID.expect("APP_ID env var not set at compile time")
}

pub fn resource_file() -> &'static str {
    RESOURCE_FILE.expect("RESOURCE_FILE env var not set at compile time")
}
