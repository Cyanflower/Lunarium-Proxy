**依赖项 (`Cargo.toml`):**

```toml
[package]
name = "gemini-proxy"
version = "0.1.0"
edition = "2024"

[dependencies]
# --- Core Async & Web ---
tokio = { version = "1", features = ["full"] } # 异步运行时 (macros, rt-multi-thread, net, time, sync, io-util)
axum = { version = "0.8", features = ["json","tracing","tokio","macros","http2"] } # Web 框架
axum-server = { version = "0.6", features = ["tls-rustls"] } # Axum TLS 支持 (使用 rustls)
tower = "0.4" # Axum 依赖的服务抽象
tower-http = { version = "0.5", features = ["fs", "trace", "cors"] } # HTTP 中间件 (静态文件, 日志, CORS)

# --- HTTP Client ---
reqwest = { version = "0.12", features = ["json", "stream", "cookies", "rustls-tls", "multipart"] } # HTTP 客户端 (启用 stream, cookies, rustls)

# --- Serialization / Deserialization ---
serde = { version = "1", features = ["derive"] } # 核心序列化/反序列化 trait
serde_json = "1" # JSON 支持
toml = "0.8" # TOML 配置解析

# --- Configuration & CLI ---
clap = { version = "4", features = ["derive"] } # 命令行参数解析

# --- Logging ---
tracing = "0.1" # 结构化日志框架
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] } # 日志输出和过滤

# --- Time ---
chrono = { version = "0.4", features = ["serde"] } # 日期和时间处理 (启用 serde 支持)

# --- Crypto & Randomness ---
base64 = "0.22" # base64加密
rand = "0.8" # 随机数生成 (用于生成 Key)
rcgen = "0.12" # 生成自签名 TLS 证书

# --- Error Handling ---
thiserror = "1" # 辅助创建自定义错误类型

# --- Utilities ---
futures = "0.3" # map, filter等工具
futures-util = "0.3" # Async stream 和 future 常用工具
uuid = { version = "1", features = ["v4", "serde"] } # 可选: 生成唯一请求 ID 用于日志追踪

# --- State Management ---
# Standard library: Arc, RwLock used directly

```

**文件结构:**

```
gemini-proxy/
├── Cargo.toml         # 项目依赖配置文件
├── config.toml.example # 示例配置文件 (避免提交真实密钥)
├── src/
│   ├── main.rs        # 程序入口: 初始化, 启动服务
│   ├── cli.rs         # Clap 命令行参数定义
│   ├── config.rs      # 配置数据结构 (serde), 加载/保存逻辑
│   ├── state.rs       # AppState 定义 (Arc<RwLock<...>>), 包含运行时状态
│   ├── error.rs       # 自定义错误类型 (thiserror)
│   ├── logger.rs      # 日志系统初始化 (tracing)
│   ├── tls.rs         # TLS (rustls, rcgen) 配置辅助
│   ├── api/           # Axum API 层 (路由, 中间件, 请求/响应处理)
│   │   ├── mod.rs     # 组装 API 路由 (v1)
│   │   ├── middleware.rs# 认证中间件 (生成、验证自定义 API Key)
│   │   └── routes/    # API 路由处理函数
│   │       ├── mod.rs # 导出路由处理器
│   │       ├── proxy.rs # 处理 /api/v1/proxy (或其他类似路径) 的转发请求
│   │       └── management.rs # 处理 /api/v1/manage/* (Key/Cookie/配置管理)
│   ├── proxy/         # 核心代理逻辑 (独立于 Web 框架)
│   │   ├── mod.rs     # 导出代理功能
│   │   ├── core.rs    # 核心转发决策逻辑 (调用 gemini/aistudio/prompt)
│   │   ├── gemini.rs  # Gemini API 模式实现 (请求构建, Key 选择)
│   │   ├── aistudio.rs# AI Studio 模式实现 (请求模拟, Cookie 选择)
│   │   ├── prompt.rs  # 请求体中的提示词预处理逻辑
│   │   └── models.rs  # 可选: Gemini/AI Studio 请求/响应的内部模型
│   └── manager/       # 状态管理和后台任务
│       ├── mod.rs     # 导出管理功能
│       ├── keys.rs    # Gemini Key 管理 (状态更新, 选择)
│       ├── cookies.rs # AI Studio Cookie 管理 (状态更新, 选择)
│       └── scheduler.rs # 定时任务 (每日重置, 状态检测)
├── web/               # [解耦] 前端静态文件 (HTML, CSS, JS) - 独立构建/部署
│   ├── index.html
│   ├── styles/
│   └── scripts/
└── certificates/      # (可选) 存放用户提供的域名证书和私钥
    ├── cert.pem
    └── key.pem
```

**说明:**

*   **`web/` 目录:** 这个目录包含了所有的前端代码。它与 `src/` 完全分离。Rust 后端通过 `tower-http::services::ServeDir` 提供该目录下的静态文件服务。部署时，可以独立构建前端资源，然后让 Rust 程序指向构建后的静态文件目录。这为将来使用 Tauri 提供了便利，Tauri 可以直接加载 `web/` 下的内容或其构建产物。
*   **`api/` vs `proxy/`:** `api/` 负责处理 HTTP 请求的接入、认证、解析和响应构建（使用 Axum）。`proxy/` 负责具体的业务逻辑：如何选择密钥/Cookie、如何构建目标请求、如何处理提示词、如何与外部服务交互。这种分离使得 `proxy/` 模块更容易测试，并且理论上可以替换 `api/` 层（例如换成 gRPC 接口）而无需修改核心代理逻辑。
*   **`manager/`:** 包含与状态（Keys, Cookies）直接相关的管理逻辑（增删改查、状态更新）和后台任务（定时器）。`proxy/` 模块会调用 `manager/` 中的函数来选择 Key/Cookie 或更新其状态。

**详细业务流 (以 AI Studio Cookie 模式处理流式响应为例):**

1.  **启动 (`main.rs`):**
    *   解析命令行参数 (`cli.rs`) 获取配置路径、端口等。
    *   加载配置文件 (`config.rs`) 到 `Config` 结构体。若初始自定义 API Key 不存在，生成并保存。
    *   初始化日志系统 (`logger.rs`)。
    *   创建 `AppState` (`state.rs`)，用 `Arc<RwLock<AppState>>` 包裹，填充来自 `Config` 的初始数据 (Keys, Cookies, 设置)。
    *   在 `tokio::spawn` 中启动后台调度器 (`manager::scheduler::run_scheduler`)，传入 `AppState` 的克隆。
    *   配置 TLS (`tls.rs`)。
    *   构建 `axum` Router (`api::mod.rs`):
        *   `/api/v1` 路由组应用 CORS 中间件 (`tower_http::cors`) 和追踪日志中间件 (`tower_http::trace`)。
        *   `/api/v1/proxy` (或其他代理路径) 应用 `api::middleware::authenticate` 中间件，然后路由到 `api::routes::proxy::handle_proxy_request`。
        *   `/api/v1/manage/*` 路由到 `api::routes::management` 中的相应处理函数 (也需要认证，可能是不同的权限)。
        *   `/` (根路径) 和其他非 API 路径使用 `ServeDir` 路由到 `web/` 目录。
    *   使用 `axum-server` 绑定端口并启动 HTTPS 服务器，运行 `app` (Router)。

2.  **客户端请求:**
    *   用户通过 Web UI (或其他客户端) 发送 POST 请求到 `https://your-proxy.com/api/v1/proxy` (或其他配置的代理端点)。
    *   请求头包含 `Authorization: Bearer <your_custom_api_key>`。
    *   请求体是 JSON 格式，包含提示词等信息，并可能指示需要流式响应 (例如，请求体中包含 `stream: true` 字段，或者通过特定参数)。

3.  **认证 (`api::middleware::authenticate`):**
    *   从请求头提取 `custom_api_key`。
    *   获取 `AppState` 的读锁。
    *   验证 Key 是否存在且有效。
    *   将验证通过的 Key 信息存入请求扩展 (request extensions)，用于后续日志记录或处理。
    *   添加包含 `custom_api_key` 的 `tracing` span。
    *   释放读锁。如果验证失败，返回 401。

4.  **代理路由处理 (`api::routes::proxy::handle_proxy_request`):**
    *   接收经过认证的请求。
    *   从请求扩展中获取 `custom_api_key`。
    *   从 `AppState` (读锁) 读取当前配置，确定应使用 AI Studio 模式。
    *   解析请求体 (`axum::body::Bytes` 或 `axum::Json`)。
    *   调用 `proxy::core::process_request(app_state.clone(), custom_api_key, request_data)`.

5.  **核心代理逻辑 (`proxy::core::process_request`):**
    *   根据模式 (AI Studio) 和请求数据，调用 `proxy::prompt::preprocess_prompt` 对提示词进行预处理。
    *   调用 `manager::cookies::select_cookie(app_state.clone())` 选择一个可用的 AI Studio Cookie。
        *   内部获取 `AppState` 写锁。
        *   根据轮询、状态等逻辑选择 Cookie。
        *   记录使用情况 (增加计数)。
        *   返回选定的 Cookie 值。
        *   释放写锁。(失败则返回错误)
    *   调用 `proxy::aistudio::forward_to_aistudio(selected_cookie, processed_request_data)`。

6.  **AI Studio 转发 (`proxy::aistudio::forward_to_aistudio`):**
    *   创建 `reqwest::Client` (可能需要配置 cookie store)。
    *   **模拟浏览器请求:** 构造发往 Google AI Studio 后端 API 的 `reqwest::Request`。
        *   设置目标 URL。
        *   设置 `Cookie` Header (包含 `selected_cookie` 及可能的其他必要 cookies)。
        *   设置 `User-Agent`, `Referer`, `Origin` 等 Headers 模拟浏览器。
        *   设置正确的 `Content-Type`。
        *   设置经过 `prompt.rs` 处理后的请求体。
        *   **关键:** 确保请求参数/格式与浏览器开发者工具中观察到的 AI Studio 网络请求一致。
    *   使用 `reqwest_client.execute(request).await?` 发送请求。

7.  **处理响应流 (`proxy::core` -> `api::routes::proxy`):**
    *   `forward_to_aistudio` 返回 `reqwest::Response`。
    *   检查响应状态码。如果 4xx/5xx，记录错误，可能需要调用 `manager::cookies::update_cookie_status` 更新 Cookie 状态 (需要写锁)。
    *   检查响应头 `Content-Type` 是否为 `text/event-stream` 或其他流式类型。
    *   **日志:** 记录请求 (关联 `custom_api_key`) 和响应概要信息 (状态码, headers)。
    *   **构建 Axum 响应:**
        *   获取 `reqwest` 响应的字节流: `let response_stream = response.bytes_stream();`
        *   创建 `axum::body::Body` 从该流: `let axum_body = axum::body::Body::from_stream(response_stream);`
        *   构建 `axum::response::Response`：
            ```rust
            use axum::response::{IntoResponse, Response};
            use axum::http::{StatusCode, HeaderMap};

            let mut headers = HeaderMap::new();
            // 复制必要的响应头 (如 Content-Type) 从 reqwest::Response 到 Axum Response
            if let Some(content_type) = original_response.headers().get(axum::http::header::CONTENT_TYPE) {
                headers.insert(axum::http::header::CONTENT_TYPE, content_type.clone());
            }
            // ... 其他需要转发的头

            Response::builder()
                .status(original_response.status()) // 转发原始状态码
                .header(axum::http::header::CONTENT_TYPE, "text/event-stream") // 确保 SSE 类型正确
                // 添加其他必要头
                .body(axum_body)
                .unwrap() // 在实际代码中应处理错误
                .into_response()
            ```
    *   返回这个 `axum::response::Response`。

8.  **响应流回传:**
    *   `axum` 和 `hyper` 底层处理将 `axum_body` 中的数据块 (chunks) 流式地发送回原始客户端 (Web UI 或 SillyTavern)。