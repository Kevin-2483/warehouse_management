// @generated automatically by Diesel CLI.

diesel::table! {
    warehouses (warehouse_id) {
        warehouse_id -> Integer,
        localkey -> Nullable<Text>,
        warehouse_name -> Text,
        location -> Text,
        capacity -> Nullable<Integer>,
        current_stock -> Integer,
        last_updated -> Timestamp,
    }
}

diesel::table! {
    warehouse_stock (warehouse_id, material_id) {
        warehouse_id -> Integer,
        material_id -> Integer,
        quantity -> Integer,
        last_updated -> Timestamp,
    }
}

diesel::table! {
    users (user_id) {
        user_id -> Integer,
        username -> Text,
        password_hash -> Text,
        full_name -> Nullable<Text>,
        position -> Nullable<Text>,
        contact_info -> Nullable<Text>,
        status -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    roles (role_id) {
        role_id -> Integer,
        role_name -> Text,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    user_roles (user_id, role_id) {
        user_id -> Integer,
        role_id -> Integer,
    }
}

diesel::table! {
    permissions (permission_id) {
        permission_id -> Integer,
        permission_name -> Text,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    role_permissions (role_id, permission_id) {
        role_id -> Integer,
        permission_id -> Integer,
    }
}

diesel::table! {
    operation_logs (log_id) {
        log_id -> Integer,
        user_id -> Integer,
        action -> Text,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    production_tasks (task_id) {
        task_id -> Integer,
        product_id -> Integer,
        quantity -> Integer,
        due_date -> Nullable<Date>,
        created_by -> Integer,
        created_at -> Timestamp,
        status -> Text,
    }
}

diesel::table! {
    production_costs (cost_id) {
        cost_id -> Integer,
        process_type -> Text,
        cost_per_unit -> Decimal,
        created_by -> Integer,
        created_at -> Timestamp,
    }
}

diesel::table! {
    price_formulas (formula_id) {
        formula_id -> Integer,
        formula_name -> Nullable<Text>,
        base_material_cost -> Decimal,
        additional_material_cost -> Decimal,
        galvanization_cost -> Decimal,
        labor_cost -> Decimal,
        management_fee -> Decimal,
        sales_fee -> Decimal,
        manufacturing_fee -> Decimal,
        vat -> Decimal,
        profit -> Decimal,
        created_by -> Integer,
        created_at -> Timestamp,
    }
}

diesel::table! {
    product_specifications (product_id) {
        product_id -> Integer,
        product_name -> Text,
        model -> Nullable<Text>,
        material_type -> Nullable<Text>,
        color -> Nullable<Text>,
        dimensions -> Nullable<Text>,
        created_by -> Integer,
        created_at -> Timestamp,
    }
}

diesel::table! {
    materials (material_id) {
        material_id -> Integer,
        material_name -> Text,
        category -> Nullable<Text>,
        type_ -> Nullable<Text>,
        supplier -> Nullable<Text>,
        created_by -> Integer,
        created_at -> Timestamp,
    }
}

diesel::table! {
    material_requests (request_id) {
        request_id -> Integer,
        material_id -> Integer,
        quantity -> Integer,
        requested_by -> Integer,
        warehouse_id -> Integer,
        request_date -> Timestamp,
        status -> Text,
    }
}

diesel::joinable!(warehouse_stock -> warehouses (warehouse_id));
diesel::joinable!(warehouse_stock -> materials (material_id));
diesel::joinable!(user_roles -> users (user_id));
diesel::joinable!(user_roles -> roles (role_id));
diesel::joinable!(role_permissions -> roles (role_id));
diesel::joinable!(role_permissions -> permissions (permission_id));
diesel::joinable!(operation_logs -> users (user_id));
diesel::joinable!(production_tasks -> product_specifications (product_id));
diesel::joinable!(production_tasks -> users (created_by));
diesel::joinable!(production_costs -> users (created_by));
diesel::joinable!(price_formulas -> users (created_by));
diesel::joinable!(product_specifications -> users (created_by));
diesel::joinable!(materials -> users (created_by));
diesel::joinable!(material_requests -> materials (material_id));
diesel::joinable!(material_requests -> users (requested_by));
diesel::joinable!(material_requests -> warehouses (warehouse_id));

diesel::allow_tables_to_appear_in_same_query!(
    warehouses,
    warehouse_stock,
    users,
    roles,
    user_roles,
    permissions,
    role_permissions,
    operation_logs,
    production_tasks,
    production_costs,
    price_formulas,
    product_specifications,
    materials,
    material_requests,
);