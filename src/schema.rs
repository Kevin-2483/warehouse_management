// @generated automatically by Diesel CLI.

diesel::table! {
    material_requests (request_id) {
        request_id -> Nullable<Integer>,
        material_id -> Nullable<Integer>,
        quantity -> Nullable<Integer>,
        requested_by -> Nullable<Integer>,
        warehouse_id -> Nullable<Integer>,
        request_date -> Nullable<Timestamp>,
        status -> Nullable<Text>,
    }
}

diesel::table! {
    materials (material_id) {
        material_id -> Nullable<Integer>,
        material_name -> Text,
        category -> Nullable<Text>,
        #[sql_name = "type"]
        type_ -> Nullable<Text>,
        supplier -> Nullable<Text>,
        created_by -> Nullable<Integer>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    operation_logs (log_id) {
        log_id -> Nullable<Integer>,
        user_id -> Nullable<Integer>,
        action -> Nullable<Text>,
        timestamp -> Nullable<Timestamp>,
    }
}

diesel::table! {
    permissions (permission_id) {
        permission_id -> Nullable<Integer>,
        permission_name -> Text,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    price_formulas (formula_id) {
        formula_id -> Nullable<Integer>,
        formula_name -> Nullable<Text>,
        base_material_cost -> Nullable<Double>,
        additional_material_cost -> Nullable<Double>,
        galvanization_cost -> Nullable<Double>,
        labor_cost -> Nullable<Double>,
        management_fee -> Nullable<Double>,
        sales_fee -> Nullable<Double>,
        manufacturing_fee -> Nullable<Double>,
        vat -> Nullable<Double>,
        profit -> Nullable<Double>,
        created_by -> Nullable<Integer>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    product_specifications (product_id) {
        product_id -> Nullable<Integer>,
        product_name -> Text,
        model -> Nullable<Text>,
        material_type -> Nullable<Text>,
        color -> Nullable<Text>,
        dimensions -> Nullable<Text>,
        created_by -> Nullable<Integer>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    production_costs (cost_id) {
        cost_id -> Nullable<Integer>,
        process_type -> Text,
        cost_per_unit -> Double,
        created_by -> Nullable<Integer>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    production_tasks (task_id) {
        task_id -> Nullable<Integer>,
        product_id -> Nullable<Integer>,
        quantity -> Integer,
        due_date -> Nullable<Date>,
        created_by -> Nullable<Integer>,
        created_at -> Nullable<Timestamp>,
        status -> Nullable<Text>,
    }
}

diesel::table! {
    role_permissions (role_id, permission_id) {
        role_id -> Nullable<Integer>,
        permission_id -> Nullable<Integer>,
    }
}

diesel::table! {
    roles (role_id) {
        role_id -> Nullable<Integer>,
        role_name -> Text,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    user_roles (user_id, role_id) {
        user_id -> Nullable<Integer>,
        role_id -> Nullable<Integer>,
    }
}

diesel::table! {
    users (user_id) {
        user_id -> Nullable<Integer>,
        username -> Text,
        password_hash -> Text,
        full_name -> Nullable<Text>,
        position -> Nullable<Text>,
        contact_info -> Nullable<Text>,
        status -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    warehouse_stock (warehouse_id, material_id) {
        warehouse_id -> Nullable<Integer>,
        material_id -> Nullable<Integer>,
        quantity -> Nullable<Integer>,
        last_updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    warehouses (warehouse_id) {
        warehouse_id -> Nullable<Integer>,
        localkey -> Nullable<Text>,
        warehouse_name -> Text,
        location -> Text,
        capacity -> Nullable<Integer>,
        current_stock -> Nullable<Integer>,
        last_updated -> Nullable<Timestamp>,
    }
}

diesel::joinable!(material_requests -> materials (material_id));
diesel::joinable!(material_requests -> users (requested_by));
diesel::joinable!(material_requests -> warehouses (warehouse_id));
diesel::joinable!(materials -> users (created_by));
diesel::joinable!(operation_logs -> users (user_id));
diesel::joinable!(price_formulas -> users (created_by));
diesel::joinable!(product_specifications -> users (created_by));
diesel::joinable!(production_costs -> users (created_by));
diesel::joinable!(production_tasks -> product_specifications (product_id));
diesel::joinable!(production_tasks -> users (created_by));
diesel::joinable!(role_permissions -> permissions (permission_id));
diesel::joinable!(role_permissions -> roles (role_id));
diesel::joinable!(user_roles -> roles (role_id));
diesel::joinable!(user_roles -> users (user_id));
diesel::joinable!(warehouse_stock -> materials (material_id));
diesel::joinable!(warehouse_stock -> warehouses (warehouse_id));

diesel::allow_tables_to_appear_in_same_query!(
    material_requests,
    materials,
    operation_logs,
    permissions,
    price_formulas,
    product_specifications,
    production_costs,
    production_tasks,
    role_permissions,
    roles,
    user_roles,
    users,
    warehouse_stock,
    warehouses,
);
