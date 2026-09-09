const APP_ID: Option<&str> = option_env!("APP_ID");

pub fn app_id() -> &'static str {
    APP_ID.expect("APP_ID env var not set at compile time")
}

pub fn resource_path() -> &'static str {
    "/com/openworship/app/"
}
