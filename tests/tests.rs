use schema_sql::{
    FkAction, ForeignKey, IndexMethod, PrimaryKey, SqlDB, SqlType,
    SupportedDBs, error::ErrorKind,
};

#[test]
fn test_valid() {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- Primary key with default since it is unique
            name TEXT,
            email TEXT,
            total_purchases DECIMAL(10, 2) CHECK (total_purchases > 0.00 AND (total_purchases < 9999999999.00)),
            login_times DATE[]
        );
        
        CREATE TABLE IF NOT EXISTS purchases (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID REFERENCES users ON DELETE SET NULL ON UPDATE CASCADE,
            user_email TEXT REFERENCES users(email) ON DELETE SET NULL,
            username TEXT,
            name TEXT,
            amount DECIMAL(4, 2), -- Most expensive item is $1299.99
            FOREIGN KEY (username) REFERENCES users(name) ON UPDATE RESTRICT
        );

        CREATE INDEX ON users USING hash (id) INCLUDE (email, name) WITH (fillfactor=90 /* need 90 fillfactor */) WHERE total_purchases > 1000.00 ;"#;

    let database = SqlDB::from_sql(SupportedDBs::PostgreSQL, sql).unwrap();
    let db = &database.tables["users"];
    let db2 = &database.tables["purchases"];

    assert_eq!(db.columns["id"].sql_type, SqlType::Uuid);
    assert_eq!(db.columns["name"].sql_type, SqlType::Text);
    assert_eq!(db.columns["email"].sql_type, SqlType::Text);
    assert_eq!(
        db.columns["total_purchases"].sql_type,
        SqlType::Decimal(Some(10), Some(2))
    );
    assert_eq!(
        db.columns["login_times"].sql_type,
        SqlType::Array(Box::new(SqlType::Date), 1)
    );

    assert!({
        let idx = &db.columns["id"].index.as_ref().unwrap();
        if let Some(IndexMethod::Hash { fillfactor }) = &idx.method {
            fillfactor.is_none()
                && idx.name.is_none()
                && idx.predicate.as_deref() == Some("total_purchases > 1000.00")
        } else {
            false
        }
    });

    assert_eq!(db.primary_key, Some(PrimaryKey::Single("id".to_string())));
    assert_eq!(db.columns["id"].default.as_deref(), Some("gen_random_uuid()"));
    assert_eq!(
        db.columns["total_purchases"].check.as_deref(),
        Some("total_purchases > 0.00 AND (total_purchases < 9999999999.00)")
    );

    assert_eq!(
        db2.columns["user_email"].foreign_key,
        Some(ForeignKey {
            table: "users".to_string(),
            column: Some("email".to_string()),
            on_delete: Some(FkAction::SetNull),
            on_update: None
        })
    );
    assert_eq!(
        db2.columns["user_id"].foreign_key,
        Some(ForeignKey {
            table: "users".to_string(),
            column: None,
            on_delete: Some(FkAction::SetNull),
            on_update: Some(FkAction::Cascade)
        })
    );
    assert_eq!(
        db2.columns["username"].foreign_key,
        Some(ForeignKey {
            table: "users".to_string(),
            column: Some("name".to_string()),
            on_delete: None,
            on_update: Some(FkAction::Restrict),
        })
    );
}

#[test]
fn test_invalid() {
    let res = SqlDB::from_sql(
        SupportedDBs::PostgreSQL,
        "CREATE table should_fail ON users WITH",
    );

    assert_eq!(
        unsafe { res.unwrap_err_unchecked() },
        schema_sql::error::Error::new(
            ErrorKind::UnexpectedToken {
                found: "ON".to_string(),
                expected: "column definitions".to_string()
            },
            26
        )
    );
}

#[test]
fn test_invalid_fk_missing_table() {
    let res = SqlDB::from_sql(
        SupportedDBs::PostgreSQL,
        "CREATE TABLE IF NOT EXISTS purchases (user_id UUID PRIMARY KEY REFERENCES users(id));",
    );

    assert_eq!(
        unsafe { res.unwrap_err_unchecked() },
        schema_sql::error::Error::new(
            ErrorKind::MissingIdent(
                "users".to_string(),
                schema_sql::error::IdentType::Table
            ),
            75
        )
    )
}

#[test]
fn test_invalid_fk_missing_pk() {
    let res = SqlDB::from_sql(
        SupportedDBs::PostgreSQL,
        r#"CREATE TABLE IF NOT EXISTS users (id UUID);
CREATE TABLE IF NOT EXISTS purchases (user_id UUID PRIMARY KEY REFERENCES users);"#,
    );

    assert_eq!(
        unsafe { res.unwrap_err_unchecked() },
        schema_sql::error::Error::new(
            ErrorKind::MissingPrimaryKey("users".to_string()),
            119
        )
    )
}

#[test]
fn test_invalid_fk_missing_col() {
    let res = SqlDB::from_sql(
        SupportedDBs::PostgreSQL,
        r#"CREATE TABLE IF NOT EXISTS users (name TEXT);
CREATE TABLE IF NOT EXISTS purchases (user_id UUID PRIMARY KEY REFERENCES users(id));"#,
    );

    assert_eq!(
        unsafe { res.unwrap_err_unchecked() },
        schema_sql::error::Error::new(
            ErrorKind::MissingIdent(
                "id".to_string(),
                schema_sql::error::IdentType::Column
            ),
            127
        )
    )
}

#[test]
fn test_invalid_table_fk_missing_col() {
    let res = SqlDB::from_sql(
        SupportedDBs::PostgreSQL,
        r#"CREATE TABLE IF NOT EXISTS users (name TEXT);
CREATE TABLE IF NOT EXISTS purchases (user_id UUID PRIMARY KEY, FOREIGN KEY (user_id) REFERENCES users(id));"#,
    );

    assert_eq!(
        unsafe { res.unwrap_err_unchecked() },
        schema_sql::error::Error::new(
            ErrorKind::MissingIdent(
                "id".to_string(),
                schema_sql::error::IdentType::Column
            ),
            150
        )
    )
}

#[test]
fn test_valid_composite_pk() {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS orders (
            order_id UUID NOT NULL,
            product_id UUID NOT NULL,
            quantity INT,
            PRIMARY KEY (order_id, product_id)
        );
    "#;

    let db = SqlDB::from_sql(SupportedDBs::PostgreSQL, sql).unwrap();
    let table = &db.tables["orders"];

    assert_eq!(
        table.primary_key,
        Some(PrimaryKey::Composite(vec![
            "order_id".to_string(),
            "product_id".to_string()
        ]))
    );
    assert_eq!(table.columns["order_id"].sql_type, SqlType::Uuid);
    assert_eq!(table.columns["product_id"].sql_type, SqlType::Uuid);
    assert_eq!(table.columns["quantity"].sql_type, SqlType::Integer);
}

#[test]
fn test_valid_composite_pk_three_cols() {
    let sql = r#"
        CREATE TABLE inventory (
            warehouse_id INT,
            product_id INT,
            batch TEXT,
            quantity DECIMAL(10, 2),
            PRIMARY KEY (warehouse_id, product_id, batch)
        );
    "#;

    let db = SqlDB::from_sql(SupportedDBs::PostgreSQL, sql).unwrap();
    let table = &db.tables["inventory"];

    assert_eq!(
        table.primary_key,
        Some(PrimaryKey::Composite(vec![
            "warehouse_id".to_string(),
            "product_id".to_string(),
            "batch".to_string()
        ]))
    );
}

#[test]
fn test_invalid_composite_pk_missing_column() {
    let res = SqlDB::from_sql(
        SupportedDBs::PostgreSQL,
        r#"CREATE TABLE orders (
            order_id UUID,
            PRIMARY KEY (order_id, missing_col)
        );"#,
    );

    assert_eq!(
        unsafe { res.unwrap_err_unchecked() },
        schema_sql::error::Error::new(
            ErrorKind::MissingIdent(
                "missing_col".to_string(),
                schema_sql::error::IdentType::Column
            ),
            84
        )
    )
}

#[test]
fn test_valid_table_level_fk_with_composite_pk() {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            name TEXT
        );

        CREATE TABLE IF NOT EXISTS orders (
            order_id UUID,
            user_id UUID,
            amount DECIMAL(10, 2),
            PRIMARY KEY (order_id),
            FOREIGN KEY (user_id) REFERENCES users(id)
        );
    "#;

    let db = SqlDB::from_sql(SupportedDBs::PostgreSQL, sql).unwrap();
    let orders = &db.tables["orders"];

    assert_eq!(
        orders.primary_key,
        Some(PrimaryKey::Composite(vec!["order_id".to_string()]))
    );
    assert_eq!(
        orders.columns["user_id"].foreign_key,
        Some(ForeignKey {
            table: "users".to_string(),
            column: Some("id".to_string()),
            on_delete: None,
            on_update: None
        })
    );
}
