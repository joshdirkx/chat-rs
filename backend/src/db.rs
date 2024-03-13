use lazy_static::lazy_static;
use sqlx_postgres::{PgPool, PgPoolOptions};

lazy_static!(
    pub static ref DATABASE_POOL: PgPool = {
        let database_url = "postgres://postgres:password@localhost/messaging";

        PgPoolOptions::new()
            .max_connections(5)
            .connect_lazy(&database_url)
            .expect("Failed to create pool")
    };
);