pub mod create;
pub mod read;
pub mod update;
pub mod delete;

pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
