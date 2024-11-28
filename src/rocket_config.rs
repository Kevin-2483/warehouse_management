use rocket::{Build, Rocket};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use std::collections::{HashMap, HashSet};
use rocket::figment::{providers::{Env, Serialized}, Figment};
use rocket::Config;
use crate::models::DbConn;
use rocket::http::Method;
use crate::admin_init::AdminInit;
use rocket::routes;

// 导入所有路由模块
use crate::routers::{
    role,
    user,
    warehouse,
    permission,
    operation_log,
    production_task,
    user_role,
    role_permission,
    production_cost,
    price_formula,
    product_specification,
    material,
    material_request,
};

// 将 rocket 函数移到这里
pub async fn rocket() -> Rocket<Build> {
    // 从默认配置创建 Figment 实例
    let figment = Figment::from(Config::default())
        .merge(("port", 0))
        // 合并自定义的数据库配置
        .merge(Serialized::default("databases", {
            let mut databases: HashMap<&str, HashMap<&str, &str>> = HashMap::new();
            let mut db_config: HashMap<&str, &str> = HashMap::new();
            db_config.insert("url", "sqlite://./warehouse.db");
            databases.insert("sqlite_db", db_config);
            databases
        }))
        // 合并环境变量配置，前缀为 "APP_"
        .merge(Env::prefixed("APP_"));

    // 使用自定义的配置启动 Rocket 应用程序
    let mut rocket = rocket::custom(figment)
        // 附加数据库连接
        .attach(DbConn::fairing())
        .attach(AdminInit) // 使用 AdminInit
        // 挂载路由
        .mount("/", routes![])
        .mount(
            "/api",
            routes![
                // Role routes
                role::list_roles,
                role::get_role,
                role::create_role,
                role::update_role,
                role::delete_role,
                role::search_roles,

                // User routes
                user::list_users,
                user::get_user,
                user::create_user,
                user::update_user,
                user::delete_user,
                user::search_users,

                // Warehouse routes
                warehouse::list_warehouses,
                warehouse::get_warehouse,
                warehouse::create_warehouse,
                warehouse::update_warehouse,
                warehouse::delete_warehouse,
                warehouse::search_warehouses,

                // Permission routes
                permission::list_permissions,
                permission::get_permission,
                permission::create_permission,
                permission::update_permission,
                permission::delete_permission,
                permission::search_permissions,

                // Operation Log routes
                operation_log::list_operation_logs,
                operation_log::create_operation_log,
                operation_log::get_operation_log,
                operation_log::search_operation_logs,

                // Production Task routes
                production_task::list_production_tasks,
                production_task::get_production_task,
                production_task::create_production_task,
                production_task::update_production_task,
                production_task::delete_production_task,
                production_task::search_production_tasks,

                // User Role routes
                user_role::list_user_roles,
                user_role::get_user_role,
                user_role::create_user_role,
                user_role::delete_user_role,
                user_role::get_roles_by_user,
                user_role::get_users_by_role,

                // Role Permission routes
                role_permission::list_role_permissions,
                role_permission::get_role_permission,
                role_permission::create_role_permission,
                role_permission::delete_role_permission,
                role_permission::get_permissions_by_role,
                role_permission::get_roles_by_permission,

                // Production Cost routes
                production_cost::list_production_costs,
                production_cost::get_production_cost,
                production_cost::create_production_cost,
                production_cost::update_production_cost,
                production_cost::delete_production_cost,
                production_cost::search_production_costs,

                // Price Formula routes
                price_formula::list_price_formulas,
                price_formula::get_price_formula,
                price_formula::create_price_formula,
                price_formula::update_price_formula,
                price_formula::delete_price_formula,
                price_formula::search_price_formulas,

                // Product Specification routes
                product_specification::list_product_specifications,
                product_specification::get_product_specification,
                product_specification::get_product_specification_by_name,
                product_specification::get_specifications_by_material,
                product_specification::get_specifications_by_model,
                product_specification::create_product_specification,
                product_specification::update_product_specification,
                product_specification::delete_product_specification,
                product_specification::search_product_specifications,

                // Material routes
                material::list_materials,
                material::get_material,
                material::get_material_by_name,
                material::get_materials_by_category,
                material::get_materials_by_supplier,
                material::create_material,
                material::update_material,
                material::delete_material,
                material::search_materials,
                material::list_suppliers,
                material::list_categories,

                // Material Request routes
                material_request::list_material_requests,
                material_request::get_material_request,
                material_request::get_requests_by_material,
                material_request::get_requests_by_warehouse,
                material_request::get_requests_by_status,
                material_request::get_requests_by_requester,
                material_request::create_material_request,
                material_request::search_material_requests,
            ],
        );

    // 只在 debug 模式下启用 CORS 配置
    #[cfg(debug_assertions)]
    {
        let cors = CorsOptions::default()
            .allowed_origins(AllowedOrigins::all())
            .allowed_methods(
                vec![
                    Method::Get,
                    Method::Post,
                    Method::Put,
                    Method::Delete,
                    Method::Options,
                ]
                .into_iter()
                .map(From::from)
                .collect::<HashSet<_>>(),
            )
            .allowed_headers(AllowedHeaders::all())
            .allow_credentials(true)
            .to_cors()
            .unwrap();
        rocket = rocket.attach(cors);
    }

    rocket
}