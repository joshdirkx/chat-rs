use serde::Serialize;

#[derive(Serialize, Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Option<i32>,
    pub first_name: String,
    pub last_name: String,
}