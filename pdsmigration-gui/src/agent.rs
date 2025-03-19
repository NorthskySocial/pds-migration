use bsky_sdk::api::agent::atp_agent::AtpSession;
use bsky_sdk::api::agent::Configure;
use bsky_sdk::BskyAgent;
use pdsmigration_common::error_code::{CustomError, CustomErrorType};

pub async fn login_helper(
    agent: &BskyAgent,
    pds_host: &str,
    username: &str,
    password: &str,
) -> Result<AtpSession, CustomError> {
    agent.configure_endpoint(pds_host.to_string());
    match agent.login(username, password).await {
        Ok(res) => Ok(res),
        Err(_e) => Err(CustomError {
            message: Some("Failed to login".to_string()),
            err_type: CustomErrorType::LoginError,
        }),
    }
}
