use crate::TransactionConvertError;


#[derive(Debug, thiserror::Error)]
pub enum ConscriboError {
    #[error("ErrorMessages: {0:?}")]
    ErrorMessages(Vec<String>),
    #[error("ReqwestError: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("SerdeJsonError: {0:?}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Transaction Convert Error: {0}")]
    TransactionConvertError(#[from] TransactionConvertError),
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
}

pub type ConscriboResult<T> = Result<T, ConscriboError>;
