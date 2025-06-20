use crate::errors::GuiError;
use atrium_xrpc::error::XrpcErrorKind;
use bsky_sdk::api::agent::atp_agent::AtpSession;
use bsky_sdk::api::agent::Configure;
use bsky_sdk::BskyAgent;

pub async fn login_helper(
    agent: &BskyAgent,
    pds_host: &str,
    username: &str,
    password: &str,
) -> Result<AtpSession, GuiError> {
    agent.configure_endpoint(pds_host.to_string());
    match agent.login(username, password).await {
        Ok(res) => Ok(res),
        Err(e) => {
            match e {
                atrium_xrpc::Error::HttpClient(_e) => return Err(GuiError::InvalidPdsEndpoint),
                atrium_xrpc::Error::XrpcResponse(e) => match e.error {
                    None => {
                        return Err(GuiError::InvalidPdsEndpoint);
                    }
                    Some(e) => match e {
                        XrpcErrorKind::Custom(_e) => {
                            return Err(GuiError::InvalidLogin);
                        }
                        XrpcErrorKind::Undefined(e) => {
                            let error = e.error.unwrap();
                            let message = e.message.unwrap();

                            if error == "AuthenticationRequired"
                                && message == "Invalid identifier or password"
                            {
                                return Err(GuiError::InvalidLogin);
                            }
                        }
                    },
                },
                _ => {}
            }
            Err(GuiError::Runtime)
        }
    }
}
