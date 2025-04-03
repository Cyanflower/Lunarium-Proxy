use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Gemini 代理服务
#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Cli {
    /// 配置文件路径
    #[clap(short, long, default_value = "config.toml")]
    pub config: PathBuf,

    /// 日志级别
    #[clap(short, long, default_value = "info")]
    pub log_level: String,

    /// 子命令
    #[clap(subcommand)]
    pub command: Option<Commands>,
}

/// 子命令
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 生成默认配置文件
    #[clap(name = "generate-config")]
    GenerateConfig {
        /// 输出文件路径
        #[clap(short, long, default_value = "config.toml.example")]
        output: PathBuf,
    },

    /// 验证配置文件
    #[clap(name = "validate")]
    Validate {
        /// 配置文件路径
        #[clap(short, long)]
        config: Option<PathBuf>,
    },

    /// 服务模式
    #[clap(name = "serve")]
    Serve,
}

/// 解析命令行参数
pub fn parse_args() -> Cli {
    Cli::parse()
}
