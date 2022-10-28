use std::ffi::{c_int, c_uint, CString};
use std::marker::PhantomData;
use std::ptr::slice_from_raw_parts;
use std::slice;

use libsqlite3_sys::{
    sqlite3_column_blob, sqlite3_column_bytes, sqlite3_finalize, sqlite3_prepare, sqlite3_reset,
    sqlite3_step, SQLITE_DONE, SQLITE_MISUSE, SQLITE_ROW,
};

use crate::connection::Connection;
use crate::result::Result;

pub struct Statement<'a> {
    statement: *mut libsqlite3_sys::sqlite3_stmt,
    connection: &'a Connection,
    phantom: PhantomData<libsqlite3_sys::sqlite3_stmt>,
}

pub enum StepResult {
    Row,
    Done,
    Misuse,
    Other(i32),
}

impl<'a> Statement<'a> {
    pub fn prepare<T: AsRef<str>>(connection: &'a Connection, query: T) -> Result<Self> {
        let mut statement = Self {
            statement: 0 as *mut _,
            connection,
            phantom: PhantomData,
        };

        unsafe {
            sqlite3_prepare(
                connection.sqlite3,
                CString::new(query.as_ref())?.as_ptr(),
                -1,
                &mut statement.statement,
                0 as *mut _,
            );

            connection.last_error()?;
        }

        Ok(statement)
    }

    pub fn step(&self) -> Result<StepResult> {
        unsafe {
            match sqlite3_step(self.statement) {
                SQLITE_ROW => Ok(StepResult::Row),
                SQLITE_DONE => Ok(StepResult::Done),
                SQLITE_MISUSE => Ok(StepResult::Misuse),
                other => self
                    .connection
                    .last_error()
                    .map(|_| StepResult::Other(other)),
            }
        }
    }

    pub fn reset(&self) -> Result<()> {
        unsafe {
            sqlite3_reset(self.statement);
            self.connection.last_error()
        }
    }

    pub fn column_blob(&mut self, index: i32) -> Result<&[u8]> {
        unsafe {
            let index = index as c_int;
            let pointer = sqlite3_column_blob(self.statement, index);
            self.connection.last_error()?;
            if pointer.is_null() {
                return Ok(&[]);
            }
            let len = sqlite3_column_bytes(self.statement, index) as usize;
            self.connection.last_error()?;
            Ok(slice::from_raw_parts(pointer as *const u8, len))
        }
    }
}

impl<'a> Drop for Statement<'a> {
    fn drop(&mut self) {
        unsafe { sqlite3_finalize(self.statement) };
    }
}
