//! Read-only SeaORM surface for the org web server.
//!
//! The API server owns migrations and writes. Web processes call named
//! functions here after `SET LOCAL TRANSACTION READ ONLY` and a `__web__readonly`
//! role. This module never exposes a migration runner.

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, Statement};

pub async fn healthcheck(conn: &DatabaseConnection) -> Result<(), DbErr> {
    let backend = conn.get_database_backend();
    conn.query_one(Statement::from_string(backend, "SELECT 1"))
        .await
        .map(|_| ())
}
