<h1 align="center">serde-sql</h1>

A fast, nom-based parser for SQL `CREATE` statements that builds a typed Rust AST from raw DDL and serializes it back to SQL.

If [sqlparser-rs](https://github.com/apache/datafusion-sqlparser-rs) is `syn` for SQL, a full round-trip parser for everything, `serde-sql` is `serde`.

**Parse DDL → Typed Rust schema → Modify programmatically → Serialize back to SQL**

---

## Features

* Parses `CREATE TABLE` and `CREATE INDEX` statements
* Builds a fully typed AST covering every major SQL type in PostgreSQL, with MySQL and SQLite specifics support soon
* Index metadata: Method (BTree, Hash, GIN, GiST, BRIN, SP-GiST), storage parameters, sort/null order, opclass
* Column constraints: `PRIMARY KEY`, `NOT NULL`, `UNIQUE`
* Schema-level validation: Duplicate table detection, missing table/column references on index creation
* Insertion-order preserved for columns via [IndexMap](https://github.com/indexmap-rs/indexmap) with fast insertions and lookups

---

## Usage

```rust
use serde_sql::{SqlDB, SqlType, SupportedDBs};

let sql = r#"
    CREATE TABLE IF NOT EXISTS users (
        id    UUID PRIMARY KEY,
        name  TEXT NOT NULL,
        email TEXT
    );
    CREATE INDEX ON users USING hash (id);
"#;

let db = SqlDB::from_sql(SupportedDBs::PostgreSQL, sql)?;
let users = db.tables.get("users").expect("No table named 'users' in DB");

assert_eq!(users.columns.get("id").expect("No column named 'id' in table 'users'").sql_type, SqlType::Uuid);
assert_eq!(users.primary_key.as_deref(), Some("id"));
```

---

## Supported Databases

* PostgreSQL (full support)
* MySQL and SQLite type aliases map to the same AST (lexer support soon)

---

## Non-Goals

* No `SELECT`, `INSERT`, `UPDATE`, or `DELETE` parsing, DDL only
* No comment preservation in serialized output
* No formatting preservation, output is generated SQL, not a round-trip of the input text
* Not a query planner or validator beyond schema structure

---

## Error Handling

`serde_sql::error::Error` covers:

* `InvalidCommand`: Non-`CREATE` statement
* `InvalidType`: Unrecognized SQL type
* `UnexpectedToken`: Malformed syntax
* `UnexpectedEOF`: Input ended mid-statement
* `ParseFailure`: Failed numeric/bool parse in index storage parameters
* `InvalidMethod`: Unrecognized index method
* `InvalidParam`: Unrecognized index storage parameter
* `DuplicateIdent`: Table or column defined more than once
* `MissingIdent`: Referencing a table or column that doesn't exist

---

## Links

* License: [MIT License](LICENSE)
* Changelog: [CHANGELOG.md](CHANGELOG.md)
* Contributing: [CONTRIBUTING.md](CONTRIBUTING.md)
