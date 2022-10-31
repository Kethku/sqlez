use std::ffi::{c_int, CString};
use std::marker::PhantomData;
use std::{slice, str};

use anyhow::{anyhow, Context, Result};
use libsqlite3_sys::*;

use crate::bindable::{Bind, Column};
use crate::connection::Connection;

pub struct Statement<'a> {
    statement: *mut sqlite3_stmt,
    connection: &'a Connection,
    ready: bool,
    phantom: PhantomData<sqlite3_stmt>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StepResult {
    Row,
    Done,
    Misuse,
    Other(i32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SqlType {
    Text,
    Integer,
    Blob,
    Float,
    Null,
}

impl<'a> Statement<'a> {
    pub fn prepare<T: AsRef<str>>(connection: &'a Connection, query: T) -> Result<Self> {
        let mut statement = Self {
            statement: 0 as *mut _,
            connection,
            ready: true,
            phantom: PhantomData,
        };

        unsafe {
            sqlite3_prepare_v2(
                connection.sqlite3,
                CString::new(query.as_ref())?.as_ptr(),
                -1,
                &mut statement.statement,
                0 as *mut _,
            );

            connection.last_error().context("Prepare call failed.")?;
        }

        Ok(statement)
    }

    pub fn step(&mut self) -> Result<StepResult> {
        unsafe {
            self.ready = false;
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

    pub fn reset(&mut self) -> Result<()> {
        if self.ready {
            return Ok(());
        }

        unsafe {
            sqlite3_reset(self.statement);
        }

        self.ready = true;
        self.connection
            .last_error()
            .context("Could not reset prepared statement")
    }

    pub fn bind_blob(&self, index: i32, blob: &[u8]) -> Result<()> {
        let index = index as c_int;
        let blob_pointer = blob.as_ptr() as *const _;
        let len = blob.len() as c_int;
        unsafe {
            sqlite3_bind_blob(self.statement, index, blob_pointer, len, SQLITE_TRANSIENT());
        }
        self.connection.last_error()
    }

    pub fn column_blob<'b>(&'b mut self, index: i32) -> Result<&'b [u8]> {
        let index = index as c_int;
        let pointer = unsafe { sqlite3_column_blob(self.statement, index) };

        self.connection.last_error()?;
        if pointer.is_null() {
            return Ok(&[]);
        }
        let len = unsafe { sqlite3_column_bytes(self.statement, index) as usize };
        self.connection.last_error()?;
        unsafe { Ok(slice::from_raw_parts(pointer as *const u8, len)) }
    }

    pub fn bind_double(&self, index: i32, double: f64) -> Result<()> {
        let index = index as c_int;

        unsafe {
            sqlite3_bind_double(self.statement, index, double);
        }
        self.connection.last_error()
    }

    pub fn column_double(&self, index: i32) -> Result<f64> {
        let index = index as c_int;
        let result = unsafe { sqlite3_column_double(self.statement, index) };
        self.connection.last_error()?;
        Ok(result)
    }

    pub fn bind_int(&self, index: i32, int: i32) -> Result<()> {
        let index = index as c_int;

        unsafe {
            sqlite3_bind_int(self.statement, index, int);
        }
        self.connection.last_error()
    }

    pub fn column_int(&self, index: i32) -> Result<i32> {
        let index = index as c_int;
        let result = unsafe { sqlite3_column_int(self.statement, index) };
        self.connection.last_error()?;
        Ok(result)
    }

    pub fn bind_int64(&self, index: i32, int: i64) -> Result<()> {
        let index = index as c_int;
        unsafe {
            sqlite3_bind_int64(self.statement, index, int);
        }
        self.connection.last_error()
    }

    pub fn column_int64(&self, index: i32) -> Result<i64> {
        let index = index as c_int;
        let result = unsafe { sqlite3_column_int64(self.statement, index) };
        self.connection.last_error()?;
        Ok(result)
    }

    pub fn bind_null(&self, index: i32) -> Result<()> {
        let index = index as c_int;
        unsafe {
            sqlite3_bind_null(self.statement, index);
        }
        self.connection.last_error()
    }

    pub fn bind_text(&self, index: i32, text: &str) -> Result<()> {
        let index = index as c_int;
        let text_pointer = text.as_ptr() as *const _;
        let len = text.len() as c_int;
        unsafe {
            sqlite3_bind_blob(self.statement, index, text_pointer, len, SQLITE_TRANSIENT());
        }
        self.connection.last_error()
    }

    pub fn column_text<'b>(&'b mut self, index: i32) -> Result<&'b str> {
        let index = index as c_int;
        let pointer = unsafe { sqlite3_column_text(self.statement, index) };

        self.connection.last_error()?;
        if pointer.is_null() {
            return Ok("");
        }
        let len = unsafe { sqlite3_column_bytes(self.statement, index) as usize };
        self.connection.last_error()?;

        let slice = unsafe { slice::from_raw_parts(pointer as *const u8, len) };
        Ok(str::from_utf8(slice)?)
    }

    pub fn bind<T: Bind>(&self, value: T) -> Result<()> {
        value.bind(self, 1)?;
        Ok(())
    }

    pub fn column<T: Column>(&mut self) -> Result<T> {
        let (result, _) = T::column(self, 0)?;
        Ok(result)
    }

    pub fn column_type(&mut self, index: i32) -> Result<SqlType> {
        let result = unsafe { sqlite3_column_type(self.statement, index) }; // SELECT <FRIEND> FROM TABLE
        self.connection.last_error()?;
        match result {
            SQLITE_INTEGER => Ok(SqlType::Integer),
            SQLITE_FLOAT => Ok(SqlType::Float),
            SQLITE_TEXT => Ok(SqlType::Text),
            SQLITE_BLOB => Ok(SqlType::Blob),
            SQLITE_NULL => Ok(SqlType::Null),
            _ => Err(anyhow!("Column type returned was incorrect ")),
        }
    }

    pub fn bound(&mut self, bindings: impl Bind) -> Result<&mut Self> {
        self.reset()?;
        self.bind(bindings)?;
        Ok(self)
    }

    pub fn run(&mut self) -> Result<()> {
        self.reset()?;
        while self.step()? == StepResult::Row {}
        Ok(())
    }

    pub fn map<R>(
        &mut self,
        mut callback: impl FnMut(&mut Statement) -> Result<R>,
    ) -> Result<Vec<R>> {
        let mut results = Vec::new();
        self.reset()?;
        while self.step()? == StepResult::Row {
            results.push(callback(self)?);
        }
        Ok(results)
    }

    pub fn rows<R: Column>(&mut self) -> Result<Vec<R>> {
        self.map(|s| s.column::<R>())
    }

    pub fn single<R>(&mut self, callback: impl FnOnce(&mut Statement) -> Result<R>) -> Result<R> {
        self.reset()?;
        if self.step()? != StepResult::Row {
            return Err(anyhow!(
                "Single(Map) called with query that returns no rows."
            ));
        }
        let result = callback(self)?;
        Ok(result)
    }

    pub fn row<R: Column>(&mut self) -> Result<R> {
        self.single(|this| this.column::<R>())
    }

    pub fn maybe<R>(
        &mut self,
        callback: impl FnOnce(&mut Statement) -> Result<R>,
    ) -> Result<Option<R>> {
        self.reset()?;
        if self.step()? != StepResult::Row {
            return Ok(None);
        }
        let result = callback(self)?;
        Ok(Some(result))
    }

    pub fn maybe_row<R: Column>(&mut self) -> Result<Option<R>> {
        self.maybe(|this| this.column::<R>())
    }
}

impl<'a> Drop for Statement<'a> {
    fn drop(&mut self) {
        unsafe { sqlite3_finalize(self.statement) };
    }
}
