// src/models.rs

use super::schema::*;
use serde::{Serialize, Deserialize};
use diesel::{Queryable, Identifiable, Insertable};
use diesel::prelude::*;
// use diesel::sql_types::Integer;
use chrono::NaiveDateTime;

#[derive(Queryable, Identifiable, Serialize, Deserialize, Associations)]
#[diesel(table_name = products)]
#[diesel(belongs_to(Category))]
pub struct Product {
    pub id: Option<i32>,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<i32>,
    pub deleted: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Identifiable, Serialize, Deserialize, Insertable)]
#[diesel(table_name = categories)]
pub struct Category {
    pub id: Option<i32>,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Queryable, Identifiable, Serialize, Deserialize, Associations)]
#[diesel(table_name = inventory)]
#[diesel(belongs_to(Product))]
#[diesel(belongs_to(Warehouse))]
pub struct Inventory {
    pub id: i32,
    pub product_id: i32,
    pub warehouse_id: String,
    pub quantity: i32,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Identifiable, Serialize, Deserialize, Insertable)]

#[diesel(table_name = warehouses)]
pub struct Warehouse {
    pub id: String,
    pub localkey: String,
    pub name: String,
    pub location: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Identifiable, Serialize, Deserialize, Associations)]
#[diesel(table_name = warehouse_transfers)]
#[diesel(belongs_to(Product))]
#[diesel(belongs_to(Warehouse, foreign_key = from_warehouse_id))]
pub struct WarehouseTransfer {
    pub id: i32,
    pub product_id: i32,
    pub from_warehouse_id: String,
    pub to_warehouse_id: String,
    pub quantity: i32,
    pub transfer_date: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = administrators)]
pub struct Administrator {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
