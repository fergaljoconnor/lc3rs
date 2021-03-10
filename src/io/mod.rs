mod io;
mod io_handle;

pub use io_handle::IOHandle;
pub(crate) use io_handle::{RealIOHandle};
#[cfg(test)]
pub(crate) use io_handle::{TestIOHandle};
