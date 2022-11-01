use std::ops::Deref;

use anyhow::Result;

use crate::connection::Connection;

#[must_use]
pub struct Transaction<'a> {
    name: String,
    connection: &'a Connection,
}

impl<'a> Transaction<'a> {
    pub fn save_point(connection: &'a Connection, name: impl AsRef<str>) -> Result<Self> {
        let name = name.as_ref().to_owned();
        connection.exec(format!("SAVEPOINT {}", &name))?;
        Ok(Self { name, connection })
    }

    pub fn rollback(self) -> Result<()> {
        self.exec(format!("ROLLBACK {}", self.name))
    }

    pub fn release(self) -> Result<()> {
        self.exec(format!(""))
    }
}

impl<'a> Deref for Transaction<'a> {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

impl<'a> Drop for Transaction<'a> {
    fn drop(&mut self) {
        self.connection
            .exec(format!("ROLLBACK {}", self.name))
            .expect("Rollback of transaction failed.");
    }
}
