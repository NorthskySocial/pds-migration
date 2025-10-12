use bsky_sdk::api::types::string::Did;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAccountRequest {
    pub did: Did,
    pub email: Option<String>,
    pub handle: String,
    pub invite_code: Option<String>,
    pub password: Option<String>,
    pub recovery_key: Option<String>,
    pub verification_code: Option<String>,
    pub verification_phone: Option<String>,
    pub plc_op: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAccountWithoutPDSRequest {
    pub did: Did,
    pub email: Option<String>,
    pub handle: String,
    pub invite_code: Option<String>,
    pub password: Option<String>,
    pub recovery_key: Option<String>,
    pub verification_code: Option<String>,
    pub verification_phone: Option<String>,
    pub plc_op: Option<String>,
}
