// src/lib.rs

pub mod schema;
pub mod models;

pub mod admin_init;
pub mod rocket_config; // 导入新的模块
pub mod claims; // 添加这一行以导入 claims 模块
pub mod token; // 导入新的模块
pub mod migrations; // 导入新的模块
pub mod db; // 导入新的模块
pub mod warehouse; // 导入新的模块


extern crate diesel;
extern crate serde;
