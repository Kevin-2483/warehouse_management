// src/models.rs

use super::schema::*;
use serde::{Serialize, Deserialize};
use diesel::{Queryable, Identifiable, Insertable};

// use diesel::sql_types::Integer;
use chrono::NaiveDateTime;
use rocket_sync_db_pools::{database, diesel};
use diesel::prelude::*;

use rocket::form::FromForm;
use rocket::http::Status;


#[derive(Queryable, Identifiable, Serialize, Deserialize, Associations)]
#[diesel(table_name = products)]
#[diesel(belongs_to(Category))]
pub struct Product {
    pub id: Option<i32>,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<i32>,
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

#[derive(Selectable, Queryable, Identifiable, Serialize, Deserialize, Associations)]
#[diesel(table_name = inventory)]
#[diesel(belongs_to(Product))]
#[diesel(belongs_to(Warehouse))]

pub struct Inventory {
    pub id: Option<i32>,
    pub product_id: i32,
    pub warehouse_id: String,
    pub quantity: i32,
    pub deleted: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Identifiable, Serialize, Deserialize, Insertable)]

#[diesel(table_name = warehouses)]
pub struct Warehouse {
    pub id: String,
    pub localkey: Option<String>,
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
    pub id: Option<i32>,
    pub username: String,
    pub password: String,
    pub superuser: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}



#[derive(Debug)]
pub struct JwtToken(pub String);

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomResponder {
    status: Status,
    message: String,
}

#[derive(FromForm, Deserialize)]
pub struct LoginCredentials {
    username: String,
    password: String,
}



#[derive(Insertable, Deserialize)]
#[diesel(table_name = warehouses)]
pub struct NewWarehouse {
    pub id: String,
    pub name: String,
    pub location: String,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = categories)]
pub struct NewCategory {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = products)]
pub struct NewProduct {
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = administrators)]
pub struct NewAdministrator {
    pub username: String,
    pub password: String,
    pub superuser: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Serialize, Deserialize)]
#[diesel(table_name = warehouses)]
pub struct GetWarehouses {
    pub id: String,
    pub name: String,
    pub location: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}



#[database("sqlite_db")]
pub struct DbConn(SqliteConnection);



