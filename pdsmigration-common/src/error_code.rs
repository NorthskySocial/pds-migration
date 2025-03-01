#[derive(Debug)]
pub enum CustomErrorType {
    ValidationError,
    AccountStatusError,
    LoginError,
    RuntimeError,
    CreateAccountError,
    AccountExportError,
    AccountImportError,
}

#[derive(Debug)]
pub struct CustomError {
    pub message: Option<String>,
    pub err_type: CustomErrorType,
}

impl CustomError {
    pub fn message(&self) -> String {
        match &self.message {
            Some(c) => c.clone(),
            None => String::from(""),
        }
    }
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}