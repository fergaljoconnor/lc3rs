use std::error::Error as StdError;
use std::result::Result as StdResult;

use thiserror::Error;

pub type LC3Result<T> = StdResult<T, LC3Error>;

pub(crate) type BoxedError = Box<dyn StdError + 'static>;
pub type PublicResult<T> = StdResult<T, BoxedError>;
pub type IOResult<T> = PublicResult<T>;

#[must_use]
#[derive(Error, Debug)]
pub enum LC3Error {
    #[error("The VM encountered an Internal Error: {0}")]
    Internal(String),
    #[error("Plugin encountered an error: {source}")]
    Plugin {
        #[source]
        source: BoxedError,
    },
    #[error("IO Handle encountered an error: {source}")]
    IO { source: BoxedError },
    #[error("Bad op code {code} encountered during command parsing")]
    BadOpCode { code: u8 },
    #[error("Bad trap code {code} encountered during command parsing")]
    BadTrapCode { code: u8 },
    #[error("Program length {len} exceeds maximum allowed size {max_len}")]
    ProgramSize { len: usize, max_len: usize },
    #[error("Encountered the following error: {0}")]
    Other(String),
}

fn to_boxed_error<ErrType>(err: ErrType) -> BoxedError
where
    ErrType: std::error::Error + 'static,
{
    Box::new(err) as BoxedError
}

pub trait BoxErrors<T> {
    fn box_error(self) -> PublicResult<T>;
    fn map_plugin_error(self) -> LC3Result<T>;
    fn map_io_error(self) -> LC3Result<T>;
}

impl<T, E> BoxErrors<T> for StdResult<T, E>
where
    E: std::error::Error + 'static,
{
    fn box_error(self) -> PublicResult<T> {
        self.map_err(|err| to_boxed_error(err))
    }

    fn map_plugin_error(self) -> LC3Result<T> {
        self.box_error()
            .map_err(|source| LC3Error::Plugin { source })
    }

    fn map_io_error(self) -> LC3Result<T> {
        self.box_error().map_err(|source| LC3Error::IO { source })
    }
}
