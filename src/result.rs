use std::ffi::NulError;

#[derive(Debug)]
pub enum Error {
    Sqlite {
        code: Option<isize>,
        message: Option<String>,
    },
    NulError(NulError),
}

impl From<NulError> for Error {
    fn from(nul_error: NulError) -> Self {
        Self::NulError(nul_error)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
