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
}

impl PdsSession {
    pub fn new(did: Option<String>) -> Self {
        Self {
            did,
            ..Default::default()
        }
    }

    pub fn create_old_session(
        &mut self,
        did: &str,
        access_token: &str,
        refresh_token: &str,
        host: &str,
    ) {
        match self.did.clone() {
            None => {
                self.did = Some(did.to_string());
                self.old_session_config = Some(SessionConfig {
                    access_token: access_token.to_string(),
                    refresh_token: refresh_token.to_string(),
                    host: host.to_string(),
                });
            }
            Some(_) => {
                tracing::error!("Session already exists for another DID");
                panic!("Session already exists for another DID");
            }
        };
    }

    pub fn create_new_session(
        &mut self,
        did: &str,
        access_token: &str,
        refresh_token: &str,
        host: &str,
    ) {
        match self.did.clone() {
            None => {
                self.did = Some(did.to_string());
                self.old_session_config = Some(SessionConfig {
                    access_token: access_token.to_string(),
                    refresh_token: refresh_token.to_string(),
                    host: host.to_string(),
                });
            }
            Some(_) => {
                tracing::error!("Session already exists for another DID");
                panic!("Session already exists for another DID");
            }
        };
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
}
