use hashbrown::HashMap;
use indexmap::IndexMap;

use crate::{
    Error, IdentType, Result, SqlColumn, SupportedDBs,
    lexer::{Created, parse_statement},
};

pub type ColMap = IndexMap<String, SqlColumn>;
pub type TableMap = HashMap<String, SqlTable>;

#[allow(dead_code)]
pub struct SqlTable {
    columns: ColMap,
    primary_key: Option<String>,
}

pub struct SqlDB {
    pub name: Option<String>,
    pub tables: TableMap,
    pub db: SupportedDBs,
}

impl SqlDB {
    pub fn from_sql(db: SupportedDBs, statements: &str) -> Result<Self> {
        let mut tables = TableMap::new();

        let mut statements = statements;

        loop {
            let (remaining, created) = parse_statement(db, statements)?;

            statements = remaining;

            match created {
                Created::Table { name, columns, primary_key } => {
                    if tables.contains_key(&name) {
                        return Err(Error::DuplicateIdent(
                            name,
                            IdentType::Table,
                        ));
                    } else {
                        unsafe {
                            tables.insert_unique_unchecked(
                                name,
                                SqlTable { columns, primary_key },
                            )
                        };
                    }
                }

                Created::Index {
                    table_name,
                    columns,
                } => {
                    let table = tables
                        .get_mut(table_name)
                        .ok_or_else(|| Error::MissingIdent(table_name.to_string(), IdentType::Table))?;

                    for (col_name, index) in columns {
                        table.columns
                            .get_mut(col_name)
                            .ok_or_else(|| Error::MissingIdent(col_name.to_string(), IdentType::Column))?
                            .index = Some(index);
                    }
                }
            }


            if statements.is_empty() {
                break;
            }
        }

        Ok(Self { name: None, tables, db })
    }
}
