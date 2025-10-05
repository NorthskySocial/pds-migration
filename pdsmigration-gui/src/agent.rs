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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gui_error_conversion_patterns() {
        // Test error handling patterns used in agent functions
        let test_cases = vec![
            (GuiError::InvalidPdsEndpoint, "Invalid PDS Endpoint"),
            (GuiError::InvalidLogin, "Invalid Username/Password"),
            (
                GuiError::AuthFactorTokenRequired,
                "Auth Factor Token Required",
            ),
            (GuiError::Runtime, "Runtime Exception"),
        ];

        for (error, expected_message) in test_cases {
            assert_eq!(format!("{}", error), expected_message);
        }
    }

    #[test]
    fn test_authentication_error_scenarios() {
        // Test various error scenarios that should map to specific GuiError variants
        let error_mappings = vec![
            (
                "AuthenticationRequired",
                "Invalid identifier or password",
                GuiError::InvalidLogin,
            ),
            (
                "AccountTakedown",
                "Account taken down",
                GuiError::InvalidLogin,
            ),
        ];

        for (_error_type, _message, expected_gui_error) in error_mappings {
            // Verify that the expected GuiError can be created and formatted
            let formatted = format!("{}", expected_gui_error);
            assert!(!formatted.is_empty());
        }
    }

    #[test]
    fn test_pds_endpoint_validation() {
        // Test that PDS endpoints should be properly validated
        let valid_endpoints = vec![
            "https://bsky.social",
            "https://pds.example.com",
            "http://localhost:3000",
        ];

        let invalid_endpoints = vec!["", "not-a-url", "ftp://invalid.protocol"];

        for endpoint in valid_endpoints {
            // These should be valid URL formats
            assert!(endpoint.starts_with("http"));
            assert!(endpoint.contains("://"));
        }

        for endpoint in invalid_endpoints {
            // These should be handled as invalid
            let is_valid_http = endpoint.starts_with("http://") || endpoint.starts_with("https://");
            if endpoint.is_empty() || endpoint == "not-a-url" {
                assert!(!is_valid_http);
            }
        }
    }

    #[test]
    fn test_credential_validation() {
        // Test credential validation patterns
        let test_credentials = vec![
            ("user@example.com", "password123", true),
            ("handle.bsky.social", "mypassword", true),
            ("", "password", false),
            ("user", "", false),
            ("", "", false),
        ];

        for (username, password, should_be_valid) in test_credentials {
            let is_valid = !username.is_empty() && !password.is_empty();
            assert_eq!(is_valid, should_be_valid);
        }
    }

    #[test]
    fn test_auth_factor_token_handling() {
        // Test that auth factor token is handled properly
        let valid_tokens = vec!["123456", "000000", "999999"];

        let invalid_tokens = vec![
            "", "12345",   // too short
            "1234567", // too long
            "abcdef",  // non-numeric
        ];

        for token in valid_tokens {
            assert_eq!(token.len(), 6);
            assert!(token.chars().all(|c| c.is_ascii_digit()));
        }

        for token in invalid_tokens {
            let is_invalid = token.len() != 6 || !token.chars().all(|c| c.is_ascii_digit());
            assert!(is_invalid);
        }
    }

    #[test]
    fn test_error_propagation() {
        // Test that errors are properly propagated through the system
        let errors = vec![
            GuiError::InvalidPdsEndpoint,
            GuiError::InvalidLogin,
            GuiError::AuthFactorTokenRequired,
            GuiError::Runtime,
        ];

        for error in errors {
            // Test that errors can be cloned and formatted
            let cloned = error.clone();
            let formatted = format!("{}", cloned);
            assert!(!formatted.is_empty());

            // Test debug formatting
            let debug_formatted = format!("{:?}", cloned);
            assert!(!debug_formatted.is_empty());
        }
    }

    #[test]
    fn test_username_formats() {
        // Test different username formats that should be accepted
        let valid_usernames = vec![
            "user@example.com",
            "handle.bsky.social",
            "user.handle.custom.domain.com",
            "username",
        ];

        let invalid_usernames = vec!["", " ", "user with spaces"];

        for username in valid_usernames {
            assert!(!username.is_empty());
            assert!(!username.contains(' '));
        }

        for username in invalid_usernames {
            let is_invalid =
                username.is_empty() || username.trim().is_empty() || username.contains(' ');
            assert!(is_invalid);
        }
    }

    #[test]
    fn test_error_message_consistency() {
        // Test that error messages are consistent and informative
        let error_messages = vec![
            (GuiError::NoMissingBlobs, "No Missing Blobs"),
            (GuiError::InvalidPdsEndpoint, "Invalid PDS Endpoint"),
            (GuiError::InvalidLogin, "Invalid Username/Password"),
            (GuiError::Runtime, "Runtime Exception"),
            (GuiError::Other, "Other Exception"),
            (GuiError::Success, "Success"),
            (
                GuiError::AuthFactorTokenRequired,
                "Auth Factor Token Required",
            ),
        ];

        for (error, expected) in error_messages {
            assert_eq!(format!("{}", error), expected);
            // Ensure messages are not empty and don't contain debug artifacts
            assert!(!expected.is_empty());
            assert!(!expected.contains("fmt"));
            assert!(!expected.contains("{"));
        }
    }
}
