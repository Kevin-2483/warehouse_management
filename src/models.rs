use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use rocket_sync_db_pools::database;
use rust_decimal::Decimal;

#[database("sqlite_db")]
pub struct DbConn(diesel::SqliteConnection);

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::warehouses)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Warehouse {
    pub warehouse_id: i32,
    pub localkey: Option<String>,
    pub warehouse_name: String,
    pub location: String,
    pub capacity: Option<i32>,
    pub current_stock: i32,
    pub last_updated: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::warehouses)]
pub struct NewWarehouse {
    pub localkey: Option<String>,
    pub warehouse_name: String,
    pub location: String,
    pub capacity: Option<i32>,
}

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::warehouse_stock)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct WarehouseStock {
    pub warehouse_id: i32,
    pub material_id: i32,
    pub quantity: i32,
    pub last_updated: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::warehouse_stock)]
pub struct NewWarehouseStock {
    pub warehouse_id: i32,
    pub material_id: i32,
    pub quantity: i32,
}

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub user_id: i32,
    pub username: String,
    pub password_hash: String,
    pub full_name: Option<String>,
    pub position: Option<String>,
    pub contact_info: Option<String>,
    pub status: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub username: String,
    pub password_hash: String,
    pub full_name: Option<String>,
    pub position: Option<String>,
    pub contact_info: Option<String>,
}

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::roles)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Role {
    pub role_id: i32,
    pub role_name: String,
    pub description: Option<String>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::roles)]
pub struct NewRole {
    pub role_name: String,
    pub description: Option<String>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::user_roles)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UserRole {
    pub user_id: i32,
    pub role_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::user_roles)]
pub struct NewUserRole {
    pub user_id: i32,
    pub role_id: i32,
}

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::materials)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Material {
    pub material_id: i32,
    pub material_code: String,
    pub material_name: String,
    pub description: Option<String>,
    pub unit: String,
    pub minimum_stock: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::materials)]
pub struct NewMaterial {
    pub material_code: String,
    pub material_name: String,
    pub description: Option<String>,
    pub unit: String,
    pub minimum_stock: i32,
}

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::inbound_records)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct InboundRecord {
    pub record_id: i32,
    pub warehouse_id: i32,
    pub material_id: i32,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub supplier: String,
    pub inbound_date: NaiveDateTime,
    pub operator_id: i32,
    pub status: String,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::inbound_records)]
pub struct NewInboundRecord {
    pub warehouse_id: i32,
    pub material_id: i32,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub supplier: String,
    pub operator_id: i32,
}

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::outbound_records)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct OutboundRecord {
    pub record_id: i32,
    pub warehouse_id: i32,
    pub material_id: i32,
    pub quantity: i32,
    pub recipient: String,
    pub outbound_date: NaiveDateTime,
    pub operator_id: i32,
    pub status: String,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::outbound_records)]
pub struct NewOutboundRecord {
    pub warehouse_id: i32,
    pub material_id: i32,
    pub quantity: i32,
    pub recipient: String,
    pub operator_id: i32,
}

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::transfer_records)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct TransferRecord {
    pub transfer_id: i32,
    pub from_warehouse_id: i32,
    pub to_warehouse_id: i32,
    pub material_id: i32,
    pub quantity: i32,
    pub transfer_date: NaiveDateTime,
    pub operator_id: i32,
    pub status: String,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::transfer_records)]
pub struct NewTransferRecord {
    pub from_warehouse_id: i32,
    pub to_warehouse_id: i32,
    pub material_id: i32,
    pub quantity: i32,
    pub operator_id: i32,
}

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::inventory_checks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct InventoryCheck {
    pub check_id: i32,
    pub warehouse_id: i32,
    pub check_date: NaiveDateTime,
    pub operator_id: i32,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::inventory_checks)]
pub struct NewInventoryCheck {
    pub warehouse_id: i32,
    pub operator_id: i32,
    pub notes: Option<String>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::inventory_check_details)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct InventoryCheckDetail {
    pub check_id: i32,
    pub material_id: i32,
    pub system_quantity: i32,
    pub actual_quantity: i32,
    pub difference: i32,
    pub notes: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::inventory_check_details)]
pub struct NewInventoryCheckDetail {
    pub check_id: i32,
    pub material_id: i32,
    pub system_quantity: i32,
    pub actual_quantity: i32,
    pub difference: i32,
    pub notes: Option<String>,
}

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::material_requests)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MaterialRequest {
    pub request_id: i32,
    pub material_id: i32,
    pub quantity: i32,
    pub requested_by: i32,
    pub request_date: NaiveDateTime,
    pub status: String,
    pub approved_by: Option<i32>,
    pub approval_date: Option<NaiveDateTime>,
    pub notes: Option<String>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::material_requests)]
pub struct NewMaterialRequest {
    pub material_id: i32,
    pub quantity: i32,
    pub requested_by: i32,
    pub notes: Option<String>,
}

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::permissions)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Permission {
    pub permission_id: i32,
    pub permission_name: String,
    pub description: Option<String>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::role_permissions)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct RolePermission {
    pub role_id: i32,
    pub permission_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::role_permissions)]
pub struct NewRolePermission {
    pub role_id: i32,
    pub permission_id: i32,
}

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::operation_logs)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct OperationLog {
    pub log_id: i32,
    pub user_id: i32,
    pub action: String,
    pub timestamp: NaiveDateTime,
}

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::production_tasks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ProductionTask {
    pub task_id: i32,
    pub product_id: i32,
    pub quantity: i32,
    pub due_date: Option<chrono::NaiveDate>,
    pub created_by: i32,
    pub created_at: NaiveDateTime,
    pub status: String,
}

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::production_costs)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ProductionCost {
    pub cost_id: i32,
    pub process_type: String,
    pub cost_per_unit: Decimal,
    pub created_by: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::price_formulas)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PriceFormula {
    pub formula_id: i32,
    pub formula_name: Option<String>,
    pub base_material_cost: Decimal,
    pub additional_material_cost: Decimal,
    pub galvanization_cost: Decimal,
    pub labor_cost: Decimal,
    pub management_fee: Decimal,
    pub sales_fee: Decimal,
    pub manufacturing_fee: Decimal,
    pub vat: Decimal,
    pub profit: Decimal,
    pub created_by: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::product_specifications)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ProductSpecification {
    pub product_id: i32,
    pub product_name: String,
    pub model: Option<String>,
    pub material_type: Option<String>,
    pub color: Option<String>,
    pub dimensions: Option<String>,
    pub created_by: i32,
    pub created_at: NaiveDateTime,
}