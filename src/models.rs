use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::warehouses)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Warehouse {
    pub warehouse_id: i32,
    pub localkey: Option<String>,
    pub warehouse_name: String,
    pub location: String,
    pub capacity: Option<i32>,
    pub current_stock: i32,
    pub last_updated: NaiveDateTime,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::warehouse_stock)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct WarehouseStock {
    pub warehouse_id: i32,
    pub material_id: i32,
    pub quantity: i32,
    pub last_updated: NaiveDateTime,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
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

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::roles)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Role {
    pub role_id: i32,
    pub role_name: String,
    pub description: Option<String>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::user_roles)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct UserRole {
    pub user_id: i32,
    pub role_id: i32,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::permissions)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Permission {
    pub permission_id: i32,
    pub permission_name: String,
    pub description: Option<String>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::role_permissions)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct RolePermission {
    pub role_id: i32,
    pub permission_id: i32,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::operation_logs)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct OperationLog {
    pub log_id: i32,
    pub user_id: i32,
    pub action: String,
    pub timestamp: NaiveDateTime,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::production_tasks)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ProductionTask {
    pub task_id: i32,
    pub product_id: i32,
    pub quantity: i32,
    pub due_date: Option<chrono::NaiveDate>,
    pub created_by: i32,
    pub created_at: NaiveDateTime,
    pub status: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::production_costs)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ProductionCost {
    pub cost_id: i32,
    pub process_type: String,
    pub cost_per_unit: rust_decimal::Decimal,
    pub created_by: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::price_formulas)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct PriceFormula {
    pub formula_id: i32,
    pub formula_name: Option<String>,
    pub base_material_cost: rust_decimal::Decimal,
    pub additional_material_cost: rust_decimal::Decimal,
    pub galvanization_cost: rust_decimal::Decimal,
    pub labor_cost: rust_decimal::Decimal,
    pub management_fee: rust_decimal::Decimal,
    pub sales_fee: rust_decimal::Decimal,
    pub manufacturing_fee: rust_decimal::Decimal,
    pub vat: rust_decimal::Decimal,
    pub profit: rust_decimal::Decimal,
    pub created_by: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::product_specifications)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
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

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::materials)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Material {
    pub material_id: i32,
    pub material_name: String,
    pub category: Option<String>,
    pub type_: Option<String>,
    pub supplier: Option<String>,
    pub created_by: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::material_requests)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct MaterialRequest {
    pub request_id: i32,
    pub material_id: i32,
    pub quantity: i32,
    pub requested_by: i32,
    pub warehouse_id: i32,
    pub request_date: NaiveDateTime,
    pub status: String,
}