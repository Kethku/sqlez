use std::{ops::Deref, sync::Arc};

use connection::Connection;
use thread_local::ThreadLocal;

use crate::connection;

pub struct ThreadSafeConnection {
    path: Arc<str>,
    connection: Arc<ThreadLocal<Connection>>,
}

impl ThreadSafeConnection {
    fn new(path: &str) -> Self {
        Self {
            path: Arc::from(path),
            connection: Default::default(),
        }
    }

    /// Opens a new db connection with the initialized file path. This is internal and only
    /// called from the deref function.
    /// If opening fails, the connection falls back to a shared memory connection
    fn open(&self) -> Connection {
        Connection::open(self.path.as_ref()).unwrap_or_else(|_| self.open_shared_memory())
    }

    /// Opens a shared memory connection using the file path as the identifier. This unwraps
    /// as we expect it always to succeed
    fn open_shared_memory(&self) -> Connection {
        let in_memory_path = format!("file:{}?mode=memory&cache=shared", self.path);
        Connection::open(&in_memory_path).expect("Could not create fallback in memory db")
    }
}

impl Clone for ThreadSafeConnection {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            connection: self.connection.clone(),
        }
    }
}

impl Deref for ThreadSafeConnection {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        self.connection.get_or(|| self.open())
    }
}
