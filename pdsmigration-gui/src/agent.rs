use crate::errors::GuiError;
use atrium_xrpc::error::XrpcErrorKind;
use bsky_sdk::api::agent::atp_agent::AtpSession;
use bsky_sdk::api::agent::Configure;
use bsky_sdk::api::com::atproto::server::create_session::Error;
use bsky_sdk::BskyAgent;

pub async fn describe_server(
    agent: &BskyAgent,
    pds_host: &str,
) -> Result<bsky_sdk::api::com::atproto::server::describe_server::OutputData, String> {
    agent.configure_endpoint(pds_host.to_string());
    let result = agent.api.com.atproto.server.describe_server().await;
    match result {
        Ok(output) => Ok(output.data),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn login_helper2(
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
                        XrpcErrorKind::Custom(e) => match e {
                            Error::AccountTakedown(_) => {
                                return Err(GuiError::InvalidLogin);
                            }
                            Error::AuthFactorTokenRequired(_) => {
                                return Err(GuiError::AuthFactorTokenRequired);
                            }
                        },
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

pub async fn confirm_email_token(
    agent: &BskyAgent,
    pds_host: &str,
    handle: &str,
    password: &str,
    token: &str,
) -> Result<AtpSession, GuiError> {
    use bsky_sdk::api::com::atproto::server::create_session::{Input, InputData};
    use ipld_core::ipld::Ipld;
    agent.configure_endpoint(pds_host.to_string());
    match agent
        .api
        .com
        .atproto
        .server
        .create_session(Input {
            data: InputData {
                allow_takendown: Some(true),
                auth_factor_token: Some(token.to_string()),
                identifier: handle.to_string(),
                password: password.to_string(),
            },
            extra_data: Ipld::Null,
        })
        .await
    {
        Ok(res) => Ok(res),
        Err(_e) => Err(GuiError::Runtime),
    }
    // match agent.login(username, password).await {
    //     Ok(res) => Ok(res),
    //     Err(e) => {
    //         match e {
    //             atrium_xrpc::Error::HttpClient(_e) => return Err(GuiError::InvalidPdsEndpoint),
    //             atrium_xrpc::Error::XrpcResponse(e) => match e.error {
    //                 None => {
    //                     return Err(GuiError::InvalidPdsEndpoint);
    //                 }
    //                 Some(e) => match e {
    //                     XrpcErrorKind::Custom(e) => {
    //                         match e {
    //                             Error::AccountTakedown(_) => {
    //                                 return Err(GuiError::InvalidLogin);
    //                             }
    //                             Error::AuthFactorTokenRequired(_) => {
    //
    //                             }
    //                         }
    //                         return Err(GuiError::InvalidLogin);
    //                     }
    //                     XrpcErrorKind::Undefined(e) => {
    //                         let error = e.error.unwrap();
    //                         let message = e.message.unwrap();
    //
    //                         if error == "AuthenticationRequired"
    //                             && message == "Invalid identifier or password"
    //                         {
    //                             return Err(GuiError::InvalidLogin);
    //                         }
    //                     }
    //                 },
    //             },
    //             _ => {}
    //         }
    //         Err(GuiError::Runtime)
    //     }
    // }
}
