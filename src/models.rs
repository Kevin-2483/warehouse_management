use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};

use crate::schema::*;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = warehouses)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(primary_key(warehouse_id))]
pub struct Warehouse {
    pub warehouse_id: i32,
    pub localkey: Option<String>,
    pub warehouse_name: String,
    pub location: String,
    pub capacity: Option<i32>,
    pub current_stock: i32,
    pub last_updated: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = warehouses)]
pub struct NewWarehouse {
    pub localkey: Option<String>,
    pub warehouse_name: String,
    pub location: String,
    pub capacity: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(Warehouse))]
#[diesel(belongs_to(Material))]
#[diesel(primary_key(warehouse_id, material_id))]
#[diesel(table_name = warehouse_stock)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct WarehouseStock {
    pub warehouse_id: i32,
    pub material_id: i32,
    pub quantity: i32,
    pub last_updated: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = warehouse_stock)]
pub struct NewWarehouseStock {
    pub warehouse_id: i32,
    pub material_id: i32,
    pub quantity: i32,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(primary_key(user_id))]
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

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub password_hash: String,
    pub full_name: Option<String>,
    pub position: Option<String>,
    pub contact_info: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = roles)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(primary_key(role_id))]
pub struct Role {
    pub role_id: i32,
    pub role_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = roles)]
pub struct NewRole {
    pub role_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Role))]
#[diesel(primary_key(user_id, role_id))]
#[diesel(table_name = user_roles)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UserRole {
    pub user_id: i32,
    pub role_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = user_roles)]
pub struct NewUserRole {
    pub user_id: i32,
    pub role_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = permissions)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(primary_key(permission_id))]
pub struct Permission {
    pub permission_id: i32,
    pub permission_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = permissions)]
pub struct NewPermission {
    pub permission_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(Role))]
#[diesel(belongs_to(Permission))]
#[diesel(primary_key(role_id, permission_id))]
#[diesel(table_name = role_permissions)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct RolePermission {
    pub role_id: i32,
    pub permission_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = role_permissions)]
pub struct NewRolePermission {
    pub role_id: i32,
    pub permission_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(User))]
#[diesel(table_name = operation_logs)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(primary_key(log_id))]
pub struct OperationLog {
    pub log_id: i32,
    pub user_id: i32,
    pub action: String,
    pub timestamp: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = operation_logs)]
pub struct NewOperationLog {
    pub user_id: i32,
    pub action: String,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(ProductSpecification, foreign_key = product_id))]
#[diesel(belongs_to(User, foreign_key = created_by))]
#[diesel(table_name = production_tasks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(primary_key(task_id))]
pub struct ProductionTask {
    pub task_id: i32,
    pub product_id: i32,
    pub quantity: i32,
    pub due_date: Option<NaiveDate>,
    pub created_by: i32,
    pub created_at: NaiveDateTime,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = production_tasks)]
pub struct NewProductionTask {
    pub product_id: i32,
    pub quantity: i32,
    pub due_date: Option<NaiveDate>,
    pub created_by: i32,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(User, foreign_key = created_by))]
#[diesel(table_name = production_costs)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(primary_key(cost_id))]
pub struct ProductionCost {
    pub cost_id: i32,
    pub process_type: String,
    pub cost_per_unit: Decimal,
    pub created_by: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = production_costs)]
pub struct NewProductionCost {
    pub process_type: String,
    pub cost_per_unit: Decimal,
    pub created_by: i32,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(User, foreign_key = created_by))]
#[diesel(table_name = price_formulas)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(primary_key(formula_id))]
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

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = price_formulas)]
pub struct NewPriceFormula {
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
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(User, foreign_key = created_by))]
#[diesel(table_name = product_specifications)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(primary_key(product_id))]
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

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = product_specifications)]
pub struct NewProductSpecification {
    pub product_name: String,
    pub model: Option<String>,
    pub material_type: Option<String>,
    pub color: Option<String>,
    pub dimensions: Option<String>,
    pub created_by: i32,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(User, foreign_key = created_by))]
#[diesel(table_name = materials)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(primary_key(material_id))]
pub struct Material {
    pub material_id: i32,
    pub material_name: String,
    pub category: Option<String>,
    pub type_: Option<String>,
    pub supplier: Option<String>,
    pub created_by: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = materials)]
pub struct NewMaterial {
    pub material_name: String,
    pub category: Option<String>,
    pub type_: Option<String>,
    pub supplier: Option<String>,
    pub created_by: i32,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(Material))]
#[diesel(belongs_to(User, foreign_key = requested_by))]
#[diesel(belongs_to(Warehouse))]
#[diesel(table_name = material_requests)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(primary_key(request_id))]
pub struct MaterialRequest {
    pub request_id: i32,
    pub material_id: i32,
    pub quantity: i32,
    pub requested_by: i32,
    pub warehouse_id: i32,
    pub request_date: NaiveDateTime,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = material_requests)]
pub struct NewMaterialRequest {
    pub material_id: i32,
    pub quantity: i32,
    pub requested_by: i32,
    pub warehouse_id: i32,
    pub status: String,
}