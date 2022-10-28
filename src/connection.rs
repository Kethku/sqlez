use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
};

use libsqlite3_sys::{
    self, sqlite3_close, sqlite3_errcode, sqlite3_errmsg, sqlite3_exec, sqlite3_open_v2, SQLITE_OK,
    SQLITE_OPEN_CREATE, SQLITE_OPEN_NOMUTEX, SQLITE_OPEN_READWRITE,
};

use crate::{
    result::{Error, Result},
    statement::Statement,
};

pub struct Connection {
    pub(crate) sqlite3: *mut libsqlite3_sys::sqlite3,
    phantom: PhantomData<libsqlite3_sys::sqlite3>,
}
unsafe impl Send for Connection {}

impl Connection {
    pub fn open(uri: &str) -> Result<Self> {
        let mut connection = Self {
            sqlite3: 0 as *mut _,
            phantom: PhantomData,
        };

        let flags = SQLITE_OPEN_CREATE | SQLITE_OPEN_NOMUTEX | SQLITE_OPEN_READWRITE;
        unsafe {
            sqlite3_open_v2(
                CString::new(uri)?.as_ptr(),
                &mut connection.sqlite3,
                flags,
                0 as *const _,
            );

            connection.last_error()?;
        }

        Ok(connection)
    }

    pub fn exec<T: AsRef<str>>(&self, query: T) -> Result<()> {
        unsafe {
            sqlite3_exec(
                self.sqlite3,
                CString::new(query.as_ref())?.as_ptr(),
                None,
                0 as *mut _,
                0 as *mut _,
            );
            self.last_error()?;
        }
        Ok(())
    }

    pub fn prepare<T: AsRef<str>>(&self, query: T) -> Result<Statement> {
        Statement::prepare(&self, query)
    }

    pub(crate) fn last_error(&self) -> Result<()> {
        unsafe {
            let code = sqlite3_errcode(self.sqlite3);
            if code == libsqlite3_sys::SQLITE_OK {
                return Ok(());
            }

            let message = sqlite3_errmsg(self.sqlite3);
            let message = if message.is_null() {
                None
            } else {
                Some(
                    String::from_utf8_lossy(CStr::from_ptr(message as *const _).to_bytes())
                        .into_owned(),
                )
            };

            Err(Error::Sqlite {
                code: Some(code as isize),
                message,
            })
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe { sqlite3_close(self.sqlite3) };
    }
}
