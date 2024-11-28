use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::{get, post, put, delete};
use chrono::Utc;

use crate::models::{Material, NewMaterial, DbConn};
use crate::schema::materials;
use crate::token::TokenGuard;

#[get("/materials")]
pub async fn list_materials(conn: DbConn, _token: TokenGuard) -> Result<Json<Vec<Material>>, Status> {
    conn.run(|c| {
        materials::table
            .select(Material::as_select())
            .order(materials::created_at.desc())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[get("/materials/<material_id>")]
pub async fn get_material(
    conn: DbConn,
    _token: TokenGuard,
    material_id: i32
) -> Result<Json<Material>, Status> {
    conn.run(move |c| {
        materials::table
            .find(material_id)
            .select(Material::as_select())
            .first(c)
    }).await
    .map(Json)
    .map_err(|_| Status::NotFound)
}

#[get("/materials/by_name/<material_name>")]
pub async fn get_material_by_name(
    conn: DbConn,
    _token: TokenGuard,
    material_name: String
) -> Result<Json<Material>, Status> {
    conn.run(move |c| {
        materials::table
            .filter(materials::material_name.eq(material_name))
            .select(Material::as_select())
            .first(c)
    }).await
    .map(Json)
    .map_err(|_| Status::NotFound)
}

#[get("/materials/by_category/<category>")]
pub async fn get_materials_by_category(
    conn: DbConn,
    _token: TokenGuard,
    category: String
) -> Result<Json<Vec<Material>>, Status> {
    conn.run(move |c| {
        materials::table
            .filter(materials::category.eq(category))
            .select(Material::as_select())
            .order(materials::created_at.desc())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[get("/materials/by_supplier/<supplier>")]
pub async fn get_materials_by_supplier(
    conn: DbConn,
    _token: TokenGuard,
    supplier: String
) -> Result<Json<Vec<Material>>, Status> {
    conn.run(move |c| {
        materials::table
            .filter(materials::supplier.eq(supplier))
            .select(Material::as_select())
            .order(materials::created_at.desc())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[post("/materials", data = "<material>")]
pub async fn create_material(
    conn: DbConn,
    _token: TokenGuard,
    material: Json<NewMaterial>
) -> Result<Json<Material>, Status> {
    // 检查材料名称是否已存在
    let exists = conn.run(move |c| {
        materials::table
            .filter(materials::material_name.eq(&material.material_name))
            .count()
            .get_result::<i64>(c)
    }).await;

    if let Ok(count) = exists {
        if count > 0 {
            return Err(Status::Conflict);
        }
    }

    let material_with_timestamp = (
        materials::material_name.eq(&material.material_name),
        materials::category.eq(&material.category),
        materials::type_.eq(&material.type_),
        materials::supplier.eq(&material.supplier),
        materials::created_by.eq(material.created_by),
        materials::created_at.eq(Utc::now().naive_utc()),
    );

    conn.run(move |c| {
        diesel::insert_into(materials::table)
            .values(material_with_timestamp)
            .get_result(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[put("/materials/<material_id>", data = "<material>")]
pub async fn update_material(
    conn: DbConn,
    _token: TokenGuard,
    material_id: i32,
    material: Json<NewMaterial>
) -> Result<Json<Material>, Status> {
    // 检查新的材料名称是否与其他材料冲突
    let exists = conn.run(move |c| {
        materials::table
            .filter(materials::material_name.eq(&material.material_name))
            .filter(materials::material_id.ne(material_id))
            .count()
            .get_result::<i64>(c)
    }).await;

    if let Ok(count) = exists {
        if count > 0 {
            return Err(Status::Conflict);
        }
    }

    conn.run(move |c| {
        diesel::update(materials::table.find(material_id))
            .set((
                materials::material_name.eq(&material.material_name),
                materials::category.eq(&material.category),
                materials::type_.eq(&material.type_),
                materials::supplier.eq(&material.supplier),
                materials::created_by.eq(material.created_by),
            ))
            .get_result(c)
    }).await
    .map(Json)
    .map_err(|_| Status::NotFound)
}

#[delete("/materials/<material_id>")]
pub async fn delete_material(
    conn: DbConn,
    _token: TokenGuard,
    material_id: i32
) -> Result<Status, Status> {
    conn.run(move |c| {
        diesel::delete(materials::table.find(material_id))
            .execute(c)
    }).await
    .map(|affected| {
        if affected > 0 {
            Status::NoContent
        } else {
            Status::NotFound
        }
    })
    .map_err(|_| Status::InternalServerError)
}

// 搜索材料
#[get("/materials/search?<query>&<category>&<supplier>")]
pub async fn search_materials(
    conn: DbConn,
    _token: TokenGuard,
    query: Option<String>,
    category: Option<String>,
    supplier: Option<String>
) -> Result<Json<Vec<Material>>, Status> {
    conn.run(move |c| {
        let mut query_builder = materials::table
            .into_boxed();

        if let Some(q) = query {
            query_builder = query_builder.filter(
                materials::material_name.like(format!("%{}%", q))
                    .or(materials::type_.like(format!("%{}%", q)))
            );
        }

        if let Some(cat) = category {
            query_builder = query_builder.filter(
                materials::category.eq(cat)
            );
        }

        if let Some(sup) = supplier {
            query_builder = query_builder.filter(
                materials::supplier.eq(sup)
            );
        }

        query_builder
            .order(materials::created_at.desc())
            .select(Material::as_select())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

// 获取所有供应商列表
#[get("/materials/suppliers")]
pub async fn list_suppliers(conn: DbConn, _token: TokenGuard) -> Result<Json<Vec<String>>, Status> {
    conn.run(|c| {
        materials::table
            .select(materials::supplier)
            .filter(materials::supplier.is_not_null())
            .distinct()
            .load::<Option<String>>(c)
    }).await
    .map(|suppliers| {
        Json(suppliers.into_iter().filter_map(|s| s).collect())
    })
    .map_err(|_| Status::InternalServerError)
}

// 获取所有类别列表
#[get("/materials/categories")]
pub async fn list_categories(conn: DbConn, _token: TokenGuard) -> Result<Json<Vec<String>>, Status> {
    conn.run(|c| {
        materials::table
            .select(materials::category)
            .filter(materials::category.is_not_null())
            .distinct()
            .load::<Option<String>>(c)
    }).await
    .map(|categories| {
        Json(categories.into_iter().filter_map(|c| c).collect())
    })
    .map_err(|_| Status::InternalServerError)
}
