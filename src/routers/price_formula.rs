use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::{get, post, put, delete};
use chrono::Utc;

use crate::models::{PriceFormula, NewPriceFormula, DbConn};
use crate::schema::price_formulas;
use crate::token::TokenGuard;

#[get("/price_formulas")]
pub async fn list_price_formulas(conn: DbConn, _token: TokenGuard) -> Result<Json<Vec<PriceFormula>>, Status> {
    conn.run(|c| {
        price_formulas::table
            .select(PriceFormula::as_select())
            .order(price_formulas::created_at.desc())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[get("/price_formulas/<formula_id>")]
pub async fn get_price_formula(conn: DbConn, _token: TokenGuard, formula_id: i32) -> Result<Json<PriceFormula>, Status> {
    conn.run(move |c| {
        price_formulas::table
            .find(formula_id)
            .select(PriceFormula::as_select())
            .first(c)
    }).await
    .map(Json)
    .map_err(|_| Status::NotFound)
}

#[get("/price_formulas/by_name/<formula_name>")]
pub async fn get_formula_by_name(conn: DbConn, _token: TokenGuard, formula_name: String) -> Result<Json<PriceFormula>, Status> {
    conn.run(move |c| {
        price_formulas::table
            .filter(price_formulas::formula_name.eq(formula_name))
            .select(PriceFormula::as_select())
            .first(c)
    }).await
    .map(Json)
    .map_err(|_| Status::NotFound)
}

#[post("/price_formulas", data = "<formula>")]
pub async fn create_price_formula(conn: DbConn, _token: TokenGuard, formula: Json<NewPriceFormula>) -> Result<Json<PriceFormula>, Status> {
    // 检查公式名称是否已存在
    if let Some(ref name) = formula.formula_name {
        let exists = conn.run(move |c| {
            price_formulas::table
                .filter(price_formulas::formula_name.eq(name))
                .count()
                .get_result::<i64>(c)
        }).await;

        if let Ok(count) = exists {
            if count > 0 {
                return Err(Status::Conflict);
            }
        }
    }

    let formula_with_timestamp = (
        price_formulas::formula_name.eq(&formula.formula_name),
        price_formulas::base_material_cost.eq(formula.base_material_cost),
        price_formulas::additional_material_cost.eq(formula.additional_material_cost),
        price_formulas::galvanization_cost.eq(formula.galvanization_cost),
        price_formulas::labor_cost.eq(formula.labor_cost),
        price_formulas::management_fee.eq(formula.management_fee),
        price_formulas::sales_fee.eq(formula.sales_fee),
        price_formulas::manufacturing_fee.eq(formula.manufacturing_fee),
        price_formulas::vat.eq(formula.vat),
        price_formulas::profit.eq(formula.profit),
        price_formulas::created_by.eq(formula.created_by),
        price_formulas::created_at.eq(Utc::now().naive_utc()),
    );

    conn.run(move |c| {
        diesel::insert_into(price_formulas::table)
            .values(formula_with_timestamp)
            .get_result(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[put("/price_formulas/<formula_id>", data = "<formula>")]
pub async fn update_price_formula(
    conn: DbConn,
    _token: TokenGuard,
    formula_id: i32,
    formula: Json<NewPriceFormula>
) -> Result<Json<PriceFormula>, Status> {
    // 如果更新了公式名称，检查新名称是否与其他公式冲突
    if let Some(ref name) = formula.formula_name {
        let exists = conn.run(move |c| {
            price_formulas::table
                .filter(price_formulas::formula_name.eq(name))
                .filter(price_formulas::formula_id.ne(formula_id))
                .count()
                .get_result::<i64>(c)
        }).await;

        if let Ok(count) = exists {
            if count > 0 {
                return Err(Status::Conflict);
            }
        }
    }

    conn.run(move |c| {
        diesel::update(price_formulas::table.find(formula_id))
            .set((
                price_formulas::formula_name.eq(&formula.formula_name),
                price_formulas::base_material_cost.eq(formula.base_material_cost),
                price_formulas::additional_material_cost.eq(formula.additional_material_cost),
                price_formulas::galvanization_cost.eq(formula.galvanization_cost),
                price_formulas::labor_cost.eq(formula.labor_cost),
                price_formulas::management_fee.eq(formula.management_fee),
                price_formulas::sales_fee.eq(formula.sales_fee),
                price_formulas::manufacturing_fee.eq(formula.manufacturing_fee),
                price_formulas::vat.eq(formula.vat),
                price_formulas::profit.eq(formula.profit),
                price_formulas::created_by.eq(formula.created_by),
            ))
            .get_result(c)
    }).await
    .map(Json)
    .map_err(|_| Status::NotFound)
}

#[delete("/price_formulas/<formula_id>")]
pub async fn delete_price_formula(conn: DbConn, _token: TokenGuard, formula_id: i32) -> Result<Status, Status> {
    conn.run(move |c| {
        diesel::delete(price_formulas::table.find(formula_id))
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

// 获取最新的价格公式
#[get("/price_formulas/latest")]
pub async fn get_latest_formula(conn: DbConn, _token: TokenGuard) -> Result<Json<PriceFormula>, Status> {
    conn.run(|c| {
        price_formulas::table
            .order(price_formulas::created_at.desc())
            .select(PriceFormula::as_select())
            .first(c)
    }).await
    .map(Json)
    .map_err(|_| Status::NotFound)
}

// 计算总价格
#[derive(serde::Serialize)]
pub struct PriceCalculation {
    pub base_price: f64,
    pub total_price: f64,
    pub breakdown: PriceBreakdown,
}

#[derive(serde::Serialize)]
pub struct PriceBreakdown {
    pub base_material: f64,
    pub additional_material: f64,
    pub galvanization: f64,
    pub labor: f64,
    pub management: f64,
    pub sales: f64,
    pub manufacturing: f64,
    pub vat_amount: f64,
    pub profit_amount: f64,
}

#[get("/price_formulas/<formula_id>/calculate/<base_price>")]
pub async fn calculate_price(
    conn: DbConn,
    _token: TokenGuard,
    formula_id: i32,
    base_price: f64
) -> Result<Json<PriceCalculation>, Status> {
    let formula = conn.run(move |c| {
        price_formulas::table
            .find(formula_id)
            .select(PriceFormula::as_select())
            .first::<PriceFormula>(c)
    }).await
    .map_err(|_| Status::NotFound)?;

    let breakdown = PriceBreakdown {
        base_material: base_price * formula.base_material_cost.unwrap_or(0.0),
        additional_material: base_price * formula.additional_material_cost.unwrap_or(0.0),
        galvanization: base_price * formula.galvanization_cost.unwrap_or(0.0),
        labor: base_price * formula.labor_cost.unwrap_or(0.0),
        management: base_price * formula.management_fee.unwrap_or(0.0),
        sales: base_price * formula.sales_fee.unwrap_or(0.0),
        manufacturing: base_price * formula.manufacturing_fee.unwrap_or(0.0),
        vat_amount: base_price * formula.vat.unwrap_or(0.0),
        profit_amount: base_price * formula.profit.unwrap_or(0.0),
    };

    let total_price = breakdown.base_material
        + breakdown.additional_material
        + breakdown.galvanization
        + breakdown.labor
        + breakdown.management
        + breakdown.sales
        + breakdown.manufacturing
        + breakdown.vat_amount
        + breakdown.profit_amount;

    Ok(Json(PriceCalculation {
        base_price,
        total_price,
        breakdown,
    }))
}
