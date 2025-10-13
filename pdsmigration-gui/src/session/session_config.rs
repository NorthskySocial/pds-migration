use derive_more::Display;

#[derive(Debug, Clone, Display)]
pub enum SessionError {
    #[display("DID mismatch: expected '{expected}', but got '{provided}'")]
    DidMismatch { expected: String, provided: String },
    #[display("Session already exists for DID: {did}")]
    SessionAlreadyExists { did: String },
    #[display("No DID found in session")]
    MissingDid,
    #[display("Old session configuration not found")]
    MissingOldSession,
    #[display("New session configuration not found")]
    MissingNewSession,
}

#[derive(Default, Clone)]
pub struct PdsSession {
    did: Option<String>,
    old_session_config: Option<SessionConfig>,
    new_session_config: Option<SessionConfig>,
}

#[derive(Default, Clone)]
pub struct SessionConfig {
    access_token: String,
    refresh_token: String,
    host: String,
    did: String,
}

impl SessionConfig {
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn refresh_token(&self) -> &str {
        &self.refresh_token
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn did(&self) -> &str {
        &self.did
    }
}

impl PdsSession {
    pub fn new(did: Option<String>) -> Self {
        Self {
            did,
            ..Default::default()
        }
    }

    pub fn clear(&mut self) {
        self.did = None;
        self.old_session_config = None;
        self.new_session_config = None;
    }

    pub fn create_old_session(
        &mut self,
        did: &str,
        access_token: &str,
        refresh_token: &str,
        host: &str,
    ) -> Result<(), SessionError> {
        match self.did.clone() {
            None => {
                self.did = Some(did.to_string());
                self.old_session_config = Some(SessionConfig {
                    access_token: access_token.to_string(),
                    refresh_token: refresh_token.to_string(),
                    host: host.to_string(),
                    did: did.to_string(),
                });
                Ok(())
            }
            Some(existing_did) => {
                tracing::error!("Session already exists for another DID");
                Err(SessionError::SessionAlreadyExists { did: existing_did })
            }
        }
    }

    pub fn create_new_session(
        &mut self,
        _did: &str,
        access_token: &str,
        refresh_token: &str,
        host: &str,
    ) -> Result<(), SessionError> {
        match self.did.clone() {
            None => {
                self.did = Some(_did.to_string());
                self.new_session_config = Some(SessionConfig {
                    access_token: access_token.to_string(),
                    refresh_token: refresh_token.to_string(),
                    host: host.to_string(),
                    did: _did.to_string(),
                });
                Ok(())
            }
            Some(did) => {
                if did != _did {
                    tracing::error!("DID mismatch: expected '{}', got '{}'", did, _did);
                    Err(SessionError::DidMismatch {
                        expected: did,
                        provided: _did.to_string(),
                    })
                } else {
                    self.new_session_config = Some(SessionConfig {
                        access_token: access_token.to_string(),
                        refresh_token: refresh_token.to_string(),
                        host: host.to_string(),
                        did,
                    });
                    Ok(())
                }
            }
        }
    }

    pub fn did(&self) -> &Option<String> {
        &self.did
    }

    pub fn old_session_config(&self) -> &Option<SessionConfig> {
        &self.old_session_config
    }

    pub fn new_session_config(&self) -> &Option<SessionConfig> {
        &self.new_session_config
    }

    pub fn get_did(&self) -> Result<&str, SessionError> {
        self.did.as_deref().ok_or(SessionError::MissingDid)
    }

    pub fn get_old_session_config(&self) -> Result<&SessionConfig, SessionError> {
        self.old_session_config
            .as_ref()
            .ok_or(SessionError::MissingOldSession)
    }

    pub fn get_new_session_config(&self) -> Result<&SessionConfig, SessionError> {
        self.new_session_config
            .as_ref()
            .ok_or(SessionError::MissingNewSession)
    }
}
