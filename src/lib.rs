mod connection;
mod result;
mod statement;
mod thread_safe_connection;

use thread_safe_connection::ThreadSafeConnection;

pub struct TextColumn {}

pub struct IntegerColumn {}

pub struct Db {
    pub contacts: Contacts,
}

pub struct Contacts {
    connection: ThreadSafeConnection,
    pub name: TextColumn,
    pub phone_number: TextColumn,
    pub modified: IntegerColumn,
}

impl Db {
    pub fn new(path: &str) -> Self {
        unimplemented!()
    }
}
