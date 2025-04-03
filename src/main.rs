// 模块声明
mod cli;
mod config;
mod models;
// 后续会添加更多模块

use cli::{Commands, parse_args};
use config::load_config;
use std::process;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // 解析命令行参数
    let args = parse_args();

    // 根据子命令执行不同的操作
    match args.command {
        Some(Commands::GenerateConfig { output }) => {
            println!("生成默认配置文件: {:?}", output);
            // TODO: 实现配置文件生成
            process::exit(0);
        }
        Some(Commands::Validate { config }) => {
            let config_path = config.unwrap_or(args.config.clone());
            println!("验证配置文件: {:?}", config_path);

            match load_config(&config_path) {
                Ok(_) => {
                    println!("配置文件验证通过");
                    process::exit(0);
                }
                Err(err) => {
                    eprintln!("配置文件验证失败: {}", err);
                    process::exit(1);
                }
            }
        }
        Some(Commands::Serve) | None => {
            // 加载配置文件
            let config = match load_config(&args.config) {
                Ok(config) => Arc::new(config),
                Err(err) => {
                    eprintln!("加载配置文件失败: {}", err);
                    process::exit(1);
                }
            };

            println!("加载配置文件成功");
            println!("代理服务将在 {} 上运行", config.listen_address);

            // TODO: 初始化日志系统
            // TODO: 创建通道
            // TODO: 启动核心后台任务
            // TODO: 创建 AppState
            // TODO: 配置 TLS (可选)
            // TODO: 构建 Axum Router
            // TODO: 启动 Axum 服务器
        }
    }
}
