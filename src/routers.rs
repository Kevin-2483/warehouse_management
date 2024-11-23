

#[get("/warehouses")]
async fn get_warehouses(
    conn: DbConn,
    // token: JwtToken
) -> Result<Json<Vec<GetWarehouses>>, Json<CustomResponder>> {
    // use schema::administrators::dsl::*;
    conn.run(move |c| {
        // let admin = administrators
        //     .filter(username.eq(token.0.clone()))
        //     .first::<Administrator>(c)
        //     .optional()
        //     .map_err(|_| Json(CustomResponder{ status:rocket::http::Status::InternalServerError, message: "Error checking for administrator".to_string()}))?;
        // if admin.is_none() {
        //     return Err(Json(CustomResponder{ status:rocket::http::Status::Unauthorized, message: "Permission Denied".to_string()}));
        // }
        use schema::warehouses::dsl::*;

        let result = warehouses
            .select((id, name, location, created_at, updated_at)) // 选择除去localkey之外的列
            .load::<GetWarehouses>(c)
            .map(Json)
            .expect("Error loading warehouses");
        Ok(result)
    })
    .await
}


#[post("/warehouses", format = "json", data = "<new_warehouse>")]
async fn create_warehouse(
    new_warehouse: Json<NewWarehouse>,
    conn: DbConn,
    token: JwtToken,
) -> Result<Json<Warehouse>, Json<CustomResponder>> {
    let db = conn;
    use schema::administrators::dsl::*;
    let new_warehouse = db
        .run(move |c| {
            let admin = administrators
                .filter(username.eq(token.0.clone()))
                .first::<Administrator>(c)
                .optional()
                .map_err(|_| {
                    Json(CustomResponder {
                        status: rocket::http::Status::InternalServerError,
                        message: "Error checking for administrator".to_string(),
                    })
                })?;

            if admin.is_none() {
                return Err(Json(CustomResponder {
                    status: rocket::http::Status::Unauthorized,
                    message: "Permission Denied".to_string(),
                }));
            }

            use self::schema::warehouses::dsl::*;

            let existing_warehouse = warehouses
                .filter(location.eq(&new_warehouse.location))
                .first::<Warehouse>(c)
                .optional()
                .map_err(|_| {
                    Json(CustomResponder {
                        status: rocket::http::Status::InternalServerError,
                        message: "Error checking for warehouses".to_string(),
                    })
                })?;

            if let Some(_) = existing_warehouse {
                return Err(Json(CustomResponder {
                    status: rocket::http::Status::BadRequest,
                    message: "Warehouse with this location already exists".to_string(),
                }));
            }

            let new_warehouse = Warehouse {
                id: new_warehouse.id.clone(),
                localkey: Some("".to_string()),
                name: new_warehouse.name.clone(),
                location: new_warehouse.location.clone(),
                created_at: Some(Utc::now().naive_utc()),
                updated_at: Some(Utc::now().naive_utc()),
            };

            diesel::insert_into(warehouses)
                .values(&new_warehouse)
                .execute(c)
                .map_err(|_| {
                    Json(CustomResponder {
                        status: rocket::http::Status::InternalServerError,
                        message: "Error inserting new warehouse".to_string(),
                    })
                })?;

            Ok(Json(new_warehouse))
        })
        .await?;
    Ok(new_warehouse)
}


// 登录接口
#[post("/login", data = "<credentials>")]
async fn login(
    credentials: Json<LoginCredentials>,
    conn: DbConn,
    cookies: &CookieJar<'_>,
) -> Result<Json<CustomResponder>, Json<CustomResponder>> {
    use schema::administrators::dsl::*;

    let input_username = credentials.username.clone(); // 提取用户名
    let input_password = credentials.password.clone(); // 提取密码，并用不同的变量名

    let result = conn
        .run(move |c| {
            administrators
                .filter(username.eq(&input_username))
                .first::<Administrator>(c)
                .optional()
        })
        .await;

    match result {
        Ok(Some(administrator)) if administrator.password == input_password => {
            let token = generate_token(&administrator.username);
            // 将 token 存入 Cookie 中
            let cookie: Cookie = Cookie::build(("token", token.clone()))
                .path("/")
                .secure(false)
                .http_only(true)
                .build();
            cookies.add(cookie);
            Ok(Json(CustomResponder {
                status: rocket::http::Status::Ok,
                message: "Success".to_string(),
            }))
        }
        _ => Err(Json(CustomResponder {
            status: rocket::http::Status::InternalServerError,
            message: "Error checking for administrator".to_string(),
        })),
    }
}

#[post("/signup", data = "<new_administrator>")]
async fn signup(
    new_administrator: Json<NewAdministrator>,
    conn: DbConn,
    token: JwtToken,
) -> Result<Json<CustomResponder>, Json<CustomResponder>> {
    use schema::administrators::dsl::*;
    let input_username = new_administrator.username.clone(); // 提取用户名
    let input_password = new_administrator.password.clone(); // 提取密码，并用不同的变量名
    let input_superuser = new_administrator.superuser.clone();
    let result = conn
        .run(move |c| {
            let admin = administrators
                .filter(username.eq(token.0))
                .filter(superuser.eq(true))
                .first::<Administrator>(c)
                .optional()
                .map_err(|_| {
                    Json(CustomResponder {
                        status: rocket::http::Status::InternalServerError,
                        message: "Error checking for administrator".to_string(),
                    })
                })?;
            if admin.is_none() {
                return Err(Json(CustomResponder {
                    status: rocket::http::Status::Unauthorized,
                    message: "Permission Denied".to_string(),
                }));
            }
            let existing_admin = administrators
                .filter(username.eq(input_username.clone()))
                .first::<Administrator>(c)
                .optional()
                .map_err(|_| {
                    Json(CustomResponder {
                        status: rocket::http::Status::Unauthorized,
                        message: "Error checking for administrator".to_string(),
                    })
                })?;
            if let Some(_) = existing_admin {
                return Err(Json(CustomResponder {
                    status: rocket::http::Status::BadRequest,
                    message: "Administrator with this username already exists".to_string(),
                }));
            }
            let new_administrator = NewAdministrator {
                username: input_username,
                password: input_password,
                superuser: input_superuser,
                created_at: Some(Utc::now().naive_utc()),
                updated_at: Some(Utc::now().naive_utc()),
            };
            diesel::insert_into(administrators)
                .values(&new_administrator)
                .execute(c)
                .expect("Error inserting admin");
            Ok(Json(CustomResponder {
                status: rocket::http::Status::Ok,
                message: "Administrator created".to_string(),
            }))
        })
        .await;
    result
}

#[rocket::options("/login")]
fn options_login() -> Status {
    Status::Ok // 返回一个允许的状态码，如 200 OK
}

#[rocket::options("/signup")]
fn options_signup() -> Status {
    Status::Ok // 返回一个允许的状态码， 200 OK
}

// 受保护的路由
#[get("/protected")]
fn protected_route(token: JwtToken) -> String {
    format!("Hello, {}! This is a protected route.", token.0)
}
