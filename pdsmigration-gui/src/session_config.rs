#[derive(Default)]
pub struct SessionConfig {
    pub did: Option<String>,
    pub old_session_config: Option<OldSessionConfig>,
    pub new_session_config: Option<NewSessionConfig>,
}

pub struct OldSessionConfig {
    pub old_pds_token: String,
    pub old_pds_host: String,
}

pub struct NewSessionConfig {
    pub new_pds_token: String,
    pub new_pds_host: String,
}
