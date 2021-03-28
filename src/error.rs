use std::error::Error as StdError;
use std::result::Result as StdResult;

use thiserror::Error;

pub(crate) type BoxedError = Box<dyn StdError + 'static>;
pub(crate) type Result<T> = StdResult<T, LC3Error>;

pub type PublicResult<T> = StdResult<T, BoxedError>;
pub type IOResult<T> = PublicResult<T>;
pub type PluginResult<T> = PublicResult<T>;

#[must_use]
#[derive(Error, Debug)]
pub enum LC3Error {
    #[error("The VM encountered an Internal Error: {0}")]
    Internal(String),
    #[error("Plugin encountered an error: {source}")]
    Plugin{source: BoxedError},
    #[error("IO Handle encountered an error: {source}")]
    IO{source: BoxedError},
    #[error("Encountered the following error: {0}")]
    Other(String)
}

fn to_boxed_error<ErrType>(err: ErrType) -> BoxedError
where ErrType: std::error::Error + 'static
{
    Box::new(err) as BoxedError
}

pub(crate) trait BoxErrors<T> {
    fn box_error(self) -> PublicResult<T>;
}

impl<T, E> BoxErrors<T> for StdResult<T, E>
where E: std::error::Error + 'static
{
    fn box_error(self) -> PublicResult<T> {
        self.map_err(|err| to_boxed_error(err))
    }
}
