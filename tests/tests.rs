use serde_sql::{IndexMethod, SqlDB, SqlType, SupportedDBs};

#[test]
fn test_valid() {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- Primary key with default since it is unique
            name TEXT,
            email TEXT,
            total_purchases DECIMAL(10, 2)
        );

        CREATE INDEX ON users USING hash (id) INCLUDE (email, name) WITH (fillfactor=90 /* need 90 fillfactor */) WHERE total_purchases > 1000.00 ;"#;

    let db = SqlDB::from_sql(SupportedDBs::PostgreSQL, sql).unwrap();
    let db = &db.tables.get("users").unwrap();

    assert_eq!(db.columns.get("id").unwrap().sql_type, SqlType::Uuid);
    assert_eq!(db.columns.get("name").unwrap().sql_type, SqlType::Text);
    assert_eq!(db.columns.get("email").unwrap().sql_type, SqlType::Text);
    assert_eq!(
        db.columns.get("total_purchases").unwrap().sql_type,
        SqlType::Decimal(Some(10), Some(2))
    );
    assert!({
        let idx =
            &db.columns.get("id").as_ref().unwrap().index.as_ref().unwrap();
        if let Some(IndexMethod::Hash { fillfactor }) = &idx.method {
            fillfactor.is_none()
                && idx.name.is_none()
                && idx.predicate.as_deref() == Some("total_purchases > 1000.00")
        } else {
            false
        }
    });
    assert_eq!(db.primary_key.as_ref().unwrap(), "id");
}
