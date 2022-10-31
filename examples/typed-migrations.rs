mod v1 {
    // #[derive(Table, Serialize, Deserialize)]
    // #[table_constraint("FOREIGN KEY(sequel) REFERENCES books(id)")]
    pub struct Book {
        //#[column_constraint("PRIMARY KEY"))]
        id: i64,
        title: String,
        pages: i64,
        sequel: Option<i64>,
    }

    // #[table_constraint("FOREIGN KEY(favorite_work) REFERENCES books(id)")]
    pub struct Author {
        //#[column_constraint("PRIMARY KEY"))]
        id: i64,
        name: String,
        favorite_work: Option<i64>,
    }
}

// Only allowed types: String, i64, f64, Vec<u8>, + Option variants of above

mod v2 {
    use crate::v1;

    // #[derive(Table)]
    // #[table_constraint("FOREIGN KEY(author_id) REFERENCES Author(id)")]
    pub struct Book {
        //#[column_constraint("PRIMARY KEY"))]
        id: i64,
        title: String,
        pages: i64,
        isbn: String,
        author_id: Option<i64>,
    }

    impl From<v1::Book> for Book {
        fn from(_: v1::Book) -> Self {
            todo!()
        }
    }
}

mod v3 {
    // #[derive(Table)]
    // #[table_constraint("FOREIGN KEY(author_id) REFERENCES Author(id)")]
    pub struct Author {
        //#[column_constraint("PRIMARY KEY"))]
        id: i64,
        name: String,
    }

    // #[derive(Table)]
    // #[table_constraint("FOREIGN KEY(author_id) REFERENCES Author(id)")]
    struct BookStores {
        name: String,
    }
}

fn main() {
    sqlez::migrate![
        [(sqlez::create(v1::Book), sqlez::create(v1::Author)],
        [sqlez::transfer(v1::Book, v2::Book),
        [sqlez::transfer(v1::Author, v3::Author), sqlez::create(v3::Bookstores)],
    ]
}
