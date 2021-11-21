use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[cfg(feature = "lzma")]
    #[error(transparent)]
    Lzma(#[from] lzma::LzmaError),
    #[cfg(feature = "zip")]
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
}
