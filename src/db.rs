//////////////////////////////////////////
// Tzolkin db
//////////////////////////////////////////
use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub telegram_id: String,
    pub birth_date: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Seal {
    pub id: u8,
    pub name: String,
    pub image: String,
    pub archetype: String,
    pub archetype_description: String,
    pub portrait_description: String,
    pub type_description: String,
}

// use serde::{Deserialize, Serialize};

// #[derive(Serialize, Deserialize, sqlx::FromRow)]
// pub struct Seal {
//     id: u8,
//     name: String,
//     image: String,
//     archetype: String,
//     archetype_description: String,
//     portrait_description: String,
//     type_description: String,
// }

// #[derive(Serialize, Deserialize)]
// pub struct Seals(Vec<Seal>);

pub fn save(_user_id: i64, _kin: u32) {}
