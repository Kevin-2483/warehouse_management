// @generated automatically by Diesel CLI.



diesel::table! {
    administrators (id) {
        id -> Nullable<Integer>,
        username -> Text,
        password -> Text,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    categories (id) {
        id -> Nullable<Integer>,
        name -> Text,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    inventory (id) {
        id -> Nullable<Integer>,
        product_id -> Integer,
        warehouse_id -> Integer,
        quantity -> Integer,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    products (id) {
        id -> Nullable<Integer>,
        name -> Text,
        description -> Nullable<Text>,
        category_id -> Nullable<Integer>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    warehouse_transfers (id) {
        id -> Nullable<Integer>,
        product_id -> Integer,
        from_warehouse_id -> Integer,
        to_warehouse_id -> Integer,
        quantity -> Integer,
        transfer_date -> Nullable<Timestamp>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    warehouses (id) {
        id -> Integer,
        name -> Text,
        location -> Text,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(inventory -> products (product_id));
diesel::joinable!(inventory -> warehouses (warehouse_id));
diesel::joinable!(products -> categories (category_id));
diesel::joinable!(warehouse_transfers -> products (product_id));

diesel::allow_tables_to_appear_in_same_query!(
    administrators,
    categories,
    inventory,
    products,
    warehouse_transfers,
    warehouses,
);
