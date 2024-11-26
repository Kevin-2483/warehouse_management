extern crate base64;
extern crate rocket;
use std::error::Error;
use chrono::Local;
use libp2p::floodsub::Topic;
use async_std::sync::Arc;
use log::LevelFilter;
use log::{error, info, warn};
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::Config as LogConfig;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::init_config;
use std::result::Result as StdResult;
use tokio::signal;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::task;
mod admin_init;
mod claims;
mod db;
mod migrations;
pub mod models;
mod network_setup;
mod rocket_config;
pub mod schema;
mod token;
mod warehouse;

#[tokio::main]
async fn main() -> StdResult<(), Box<dyn Error>> {
    // 获取当前时间并格式化为文件名
    let now = Local::now();
    let log_file_name = format!("log/output_{}.log", now.format("%Y-%m-%d_%H-%M-%S"));

    // 构建文件 appender
    let file_appender = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {l} - {m}{n}")))
        .build(log_file_name)
        .unwrap();

    // 构建 console appender
    let console_appender = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {l} - {m}{n}")))
        .build();

    // 创建 root 配置
    let logconfig = LogConfig::builder()
        .appender(Appender::builder().build("stdout", Box::new(console_appender)))
        .appender(Appender::builder().build("file", Box::new(file_appender)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("file")
                .build(LevelFilter::Info),
        )
        .unwrap();

    // 初始化 log4rs 配置
    init_config(logconfig).unwrap();

    let (tx, mut rx) = mpsc::channel::<(Topic, String)>(32);

    // 调用 setup_network 函数
    let swarm = network_setup::setup_network().await?;
    let swarm = Arc::new(Mutex::new(swarm)); // 包裹在 Arc 和 Mutex 中
    let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);

    // 在 swarm 处理中添加关闭信号监听
    let swarm_handle = task::spawn({
        let swarm = Arc::clone(&swarm);
        let mut shutdown_rx = shutdown_tx.subscribe();
        async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        info!("Shutting down swarm...");
                        break;
                    }
                    Some((topic, message)) = rx.recv() => {
                        info!("Processing message for topic: {:?}", topic);
                        let mut swarm = swarm.lock().await;
                        // 直接调用 send_message，不需要 match
                        let _ = swarm.behaviour_mut().send_message(topic, message);
                        info!("Message sent through swarm");
                    }
                    result = async {
                        let mut swarm_guard = swarm.lock().await;
                        network_setup::run_swarm(&mut *swarm_guard).await
                    } => {
                        if let Err(e) = result {
                            error!("Swarm launch failed: {}", e);
                            break;
                        }
                    }
                }
            }
        }
    });
    let mut swarm_handle = Some(swarm_handle);

    // 调用 rocket 函数并启动
    let rocket = rocket_config::rocket().await;

    // 将 Rocket 的启动任务交给 tokio::spawn
    let rocket_handle = tokio::spawn(async move {
        if let Err(e) = rocket.launch().await {
            error!("Rocket launch failed: {}", e);
        }
    });

    // 处理 rocket_handle
    let mut rocket_handle = Some(rocket_handle);
    #[cfg(debug_assertions)]
    {
        let topic = Topic::new("example_topic");
        let tx_clone = tx.clone();
        let mut shutdown_rx = shutdown_tx.subscribe();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        info!("Shutting down debug message sender...");
                        break;
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(10)) => {
                        info!("Sending periodic debug message..."); // 添加发送前的日志
                        match tx_clone.send((topic.clone(), "定时消息".to_string())).await {
                            Ok(_) => info!("Successfully sent periodic message"), // 添加发送成功的日志
                            Err(e) => error!("Failed to send periodic message: {}", e),
                        }
                    }
                }
            }
        });
    }
    // 处理 SIGINT 信号
    tokio::select! {
        _ = signal::ctrl_c() => {
            warn!("Received shutdown signal, shutting down...");
            let _ = shutdown_tx.send(());
        }
        _ = async { if let Some(handle) = rocket_handle.take() { handle.await.unwrap(); } } => {
            info!("Rocket task completed.");
        }
        _ = async { if let Some(handle) = swarm_handle.take() { handle.await.unwrap(); } } => {
            info!("Swarm task completed.");
        }
    }

    info!("Waiting for tasks to finish...");
    if let Some(handle) = rocket_handle {
        handle.await?;
    }
    if let Some(handle) = swarm_handle {
        handle.await?;
    }

    Ok(())
}
