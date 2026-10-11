use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("git error: {0}")]
    Git(String),
    #[error("invalid transition: {0}")]
    InvalidTransition(String),
    #[error("jev error: {0}")]
    Jev(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
