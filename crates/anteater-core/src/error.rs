use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Nix(#[from] nix::Error),

    #[error("{0}")]
    Readline(#[from] rustyline::error::ReadlineError),

    #[error("{0}")]
    NulError(#[from] std::ffi::NulError),

    #[error("{0}")]
    InvalidArg(String),

    #[error("{0}")]
    NoSuchRegister(String),
}

pub type Result<T> = std::result::Result<T, Error>;
