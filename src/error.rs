//! Top-level application error presentation.

use std::fmt;

use ccnv::rate_service::RateServiceError;

#[derive(Debug)]
pub enum AppError {
    InvalidArgument(String),
    Service(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument(message) | Self::Service(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for AppError {}

impl From<RateServiceError> for AppError {
    fn from(error: RateServiceError) -> Self {
        match error {
            RateServiceError::CurrencyNotFound(_) => Self::InvalidArgument(error.to_string()),
            error => Self::Service(error.to_string()),
        }
    }
}
