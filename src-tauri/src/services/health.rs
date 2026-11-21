use std::path::Path;

use crate::infrastructure::database::{self, DatabaseError};

pub fn check_storage(data_dir: &Path) -> Result<i32, DatabaseError> {
    let connection = database::open(data_dir)?;
    connection
        .prepare("SELECT key, value FROM settings LIMIT 0")
        .map_err(|_| DatabaseError::ReadSchema)?;
    database::schema_version(&connection)
}
