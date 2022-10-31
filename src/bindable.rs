use anyhow::Result;

use crate::statement::Statement;

pub trait Bind {
    fn bind(&self, statement: &Statement, start_index: i32) -> Result<i32>;
}

pub trait Column: Sized {
    fn column(statement: &mut Statement, start_index: i32) -> Result<(Self, i32)>;
}

impl Bind for &[u8] {
    fn bind(&self, statement: &Statement, start_index: i32) -> Result<i32> {
        statement.bind_blob(start_index, self)?;
        Ok(start_index + 1)
    }
}

impl Bind for Vec<u8> {
    fn bind(&self, statement: &Statement, start_index: i32) -> Result<i32> {
        statement.bind_blob(start_index, self)?;
        Ok(start_index + 1)
    }
}

impl Column for Vec<u8> {
    fn column(statement: &mut Statement, start_index: i32) -> Result<(Self, i32)> {
        let result = statement.column_blob(start_index)?;
        Ok((Vec::from(result), start_index + 1))
    }
}

impl Bind for f64 {
    fn bind(&self, statement: &Statement, start_index: i32) -> Result<i32> {
        statement.bind_double(start_index, *self)?;
        Ok(start_index + 1)
    }
}

impl Column for f64 {
    fn column(statement: &mut Statement, start_index: i32) -> Result<(Self, i32)> {
        let result = statement.column_double(start_index)?;
        Ok((result, start_index + 1))
    }
}

impl Bind for i32 {
    fn bind(&self, statement: &Statement, start_index: i32) -> Result<i32> {
        statement.bind_int(start_index, *self)?;
        Ok(start_index + 1)
    }
}

impl Column for i32 {
    fn column<'a>(statement: &mut Statement, start_index: i32) -> Result<(Self, i32)> {
        let result = statement.column_int(start_index)?;
        Ok((result, start_index + 1))
    }
}

impl Bind for i64 {
    fn bind(&self, statement: &Statement, start_index: i32) -> Result<i32> {
        statement.bind_int64(start_index, *self)?;
        Ok(start_index + 1)
    }
}

impl Column for i64 {
    fn column(statement: &mut Statement, start_index: i32) -> Result<(Self, i32)> {
        let result = statement.column_int64(start_index)?;
        Ok((result, start_index + 1))
    }
}

impl Bind for usize {
    fn bind(&self, statement: &Statement, start_index: i32) -> Result<i32> {
        (*self as i64).bind(statement, start_index)
    }
}

impl Column for usize {
    fn column(statement: &mut Statement, start_index: i32) -> Result<(Self, i32)> {
        let result = statement.column_int64(start_index)?;
        Ok((result as usize, start_index + 1))
    }
}

impl Bind for () {
    fn bind(&self, statement: &Statement, start_index: i32) -> Result<i32> {
        statement.bind_null(start_index)?;
        Ok(start_index + 1)
    }
}

impl Bind for &str {
    fn bind(&self, statement: &Statement, start_index: i32) -> Result<i32> {
        statement.bind_text(start_index, self)?;
        Ok(start_index + 1)
    }
}

impl Bind for String {
    fn bind(&self, statement: &Statement, start_index: i32) -> Result<i32> {
        statement.bind_text(start_index, self)?;
        Ok(start_index + 1)
    }
}

impl Column for String {
    fn column<'a>(statement: &mut Statement, start_index: i32) -> Result<(Self, i32)> {
        let result = statement.column_text(start_index)?;
        Ok((result.to_owned(), start_index + 1))
    }
}

impl<T1: Bind, T2: Bind> Bind for (T1, T2) {
    fn bind(&self, statement: &Statement, start_index: i32) -> Result<i32> {
        let next_index = self.0.bind(statement, start_index)?;
        self.1.bind(statement, next_index)
    }
}

impl<T1: Column, T2: Column> Column for (T1, T2) {
    fn column<'a>(statement: &mut Statement, start_index: i32) -> Result<(Self, i32)> {
        let (first, next_index) = T1::column(statement, start_index)?;
        let (second, next_index) = T2::column(statement, next_index)?;
        Ok(((first, second), next_index))
    }
}

impl<T1: Bind, T2: Bind, T3: Bind> Bind for (T1, T2, T3) {
    fn bind(&self, statement: &Statement, start_index: i32) -> Result<i32> {
        let next_index = self.0.bind(statement, start_index)?;
        let next_index = self.1.bind(statement, next_index)?;
        self.2.bind(statement, next_index)
    }
}

impl<T1: Column, T2: Column, T3: Column> Column for (T1, T2, T3) {
    fn column(statement: &mut Statement, start_index: i32) -> Result<(Self, i32)> {
        let (first, next_index) = T1::column(statement, start_index)?;
        let (second, next_index) = T2::column(statement, next_index)?;
        let (third, next_index) = T3::column(statement, next_index)?;
        Ok(((first, second, third), next_index))
    }
}

impl<T1: Bind, T2: Bind, T3: Bind, T4: Bind> Bind for (T1, T2, T3, T4) {
    fn bind(&self, statement: &Statement, start_index: i32) -> Result<i32> {
        let next_index = self.0.bind(statement, start_index)?;
        let next_index = self.1.bind(statement, next_index)?;
        let next_index = self.2.bind(statement, next_index)?;
        self.3.bind(statement, next_index)
    }
}

impl<T1: Column, T2: Column, T3: Column, T4: Column> Column for (T1, T2, T3, T4) {
    fn column(statement: &mut Statement, start_index: i32) -> Result<(Self, i32)> {
        let (first, next_index) = T1::column(statement, start_index)?;
        let (second, next_index) = T2::column(statement, next_index)?;
        let (third, next_index) = T3::column(statement, next_index)?;
        let (forth, next_index) = T4::column(statement, next_index)?;
        Ok(((first, second, third, forth), next_index))
    }
}
