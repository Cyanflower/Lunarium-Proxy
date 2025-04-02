**依赖项 (`Cargo.toml`):**

```toml
[package]
name = "gemini-proxy"
version = "0.1.0"
edition = "2021"

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


## 协作模式与集成点

1.  **接口优先**: 双方需要首先就 `AppState` 的结构、`manager` 模块提供的函数签名（例如 `add_key`, `select_key`, `add_cookie`, `select_cookie`, `update_status` 等）以及 `proxy::core::process_request` 的函数签名和返回值（尤其是如何传递流式响应）达成一致。
2.  **共享状态**: `AppState` 是核心共享数据。开发者 A 负责其内部逻辑和维护，开发者 B 通过 Axum 的 `State` 或 `Extension` 访问它。使用 `Arc<RwLock<AppState>>` 确保线程安全。
3.  **配置驱动**: `config.rs` 定义的配置结构由开发者 B 主要负责，但需要开发者 A 确认其中包含初始化状态所需的所有字段（如初始密钥列表、Cookie 列表等）。
4.  **Mocking/存根**: 在集成之前，开发者 B 可以暂时 Mock 开发者 A 的 `proxy::core::process_request` 调用（例如，只返回一个固定的成功或失败响应），而开发者 A 可以编写单元测试或简单的二进制程序来调用其 `manager` 和 `proxy` 模块的功能，而无需完整的 Web 服务器。
5.  **定期同步**: 建议每天进行简短同步，讨论进度、遇到的问题和接口调整。
6.  **集成测试**: 在各自单元基本完成后，进行集成测试，重点验证 API 调用能否正确触发核心代理逻辑、状态能否正确更新、流式响应是否正常。

---

## 开发者 A: 核心代理逻辑与状态管理

**主要职责**: 实现代理的核心功能，包括与外部 Gemini/AI Studio 的交互、密钥/Cookie 的选择与状态管理、以及后台任务。

### 单元 A1: 状态管理基础 (State & Managers)

*   **开发目标**:
    *   定义应用程序的核心运行时状态结构 `AppState` (`src/state.rs`)。
    *   实现 Gemini API 密钥的管理逻辑 (`src/manager/keys.rs`)，包括数据结构、增删查改、状态更新、选择逻辑（初始可选简单轮询）。
    *   实现 AI Studio Cookie 的管理逻辑 (`src/manager/cookies.rs`)，类似密钥管理。
    *   确保状态管理是线程安全的（使用 `Arc<RwLock<...>>`）。
*   **实现流程**:
    1.  在 `src/state.rs` 中定义 `AppState` 结构体，包含 `Vec` 或 `HashMap` 来存储 `ApiKey` 和 `AiStudioCookie` 实例（这些实例也需要定义结构），以及可能的全局配置。用 `RwLock` 包裹需要修改的数据。
    2.  在 `src/manager/keys.rs` 中定义 `ApiKey` 结构（包含 key 值, 状态, 使用次数, token 消耗, 最后使用时间等）。实现 `KeyManager`（或直接在 `AppState` 上实现方法）提供 `add_key`, `get_key`, `update_key_usage`, `select_available_key` 等函数。
    3.  在 `src/manager/cookies.rs` 中定义 `AiStudioCookie` 结构（类似 `ApiKey`）。实现 `CookieManager`（或 `AppState` 上的方法）提供类似 `keys.rs` 的管理功能。
    4.  所有修改状态的操作都需要获取 `AppState` 的写锁 (`write()`)，只读操作获取读锁 (`read()`)。
*   **测试方案**:
    *   **内容**: 测试状态结构的初始化、密钥/Cookie 的添加、查找、更新（次数、状态）、删除（如果需要）、以及选择逻辑是否按预期工作（例如，能否选出未达限制且状态正常的 Key/Cookie）。测试并发访问下的安全性（基础测试，非压力测试）。
    *   **工具**: 使用 Rust 的内置单元测试 (`#[cfg(test)]`, `#[test]`)。可以直接创建 `AppState` 实例进行测试。对于并发，可以 `std::thread::spawn` 或 `tokio::spawn` 几个任务同时读写状态，检查结果是否一致（注意测试中的竞争条件）。

### 单元 A2: Gemini API 模式实现

*   **开发目标**:
    *   实现与 Google Gemini API 的交互逻辑 (`src/proxy/gemini.rs`)。
    *   根据 `AppState` 中的可用密钥进行选择。
    *   构建发送给 Gemini API 的 HTTP 请求（包括认证头）。
    *   处理来自 Gemini API 的响应（包括普通和流式）。
    *   更新密钥的使用计数和状态。
*   **实现流程**:
    1.  创建一个函数，如 `forward_to_gemini(app_state: Arc<RwLock<AppState>>, request_body: serde_json::Value) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>, ProxyError>` (返回值类型需仔细设计以支持流)。
    2.  在该函数内部，调用 `manager::keys::select_available_key` 从 `AppState` 获取一个可用密钥。
    3.  使用 `reqwest::Client` 构建请求，设置正确的 URL、HTTP 方法、`Authorization` Header (Bearer token) 和请求体。
    4.  发送请求。如果是流式请求，获取响应的 `bytes_stream()`。
    5.  处理响应：检查状态码。如果成功，返回响应流。如果失败（如 429 Too Many Requests, 401 Unauthorized），调用 `manager::keys::update_key_status` 更新 `AppState` 中对应密钥的状态（需要写锁）。
    6.  返回结果（成功时是流，失败时是错误）。
*   **测试方案**:
    *   **内容**: 测试请求构建是否正确（URL, Header, Body）。测试能否正确处理成功的响应（普通和流式）。测试能否在收到特定错误码（如 429）时正确调用状态更新函数。
    *   **工具**:
        *   单元测试 (`#[test]`)：可以 Mock `reqwest::Client` (例如使用 `httpmock` 或 `wiremock-rs` 库) 来模拟 Gemini API 的响应，验证函数行为。
        *   集成测试（可选，谨慎使用）：配置一个真实的（测试用）Gemini Key，实际调用 API，验证交互。可能需要设置环境变量或特定配置文件来启用。`assert!` 检查返回流的内容片段。

### 单元 A3: AI Studio Cookie 模式实现

*   **开发目标**:
    *   实现模拟浏览器访问 AI Studio 后端接口的逻辑 (`src/proxy/aistudio.rs`)。
    *   根据 `AppState` 中的可用 Cookie 进行选择。
    *   构建伪装的 HTTP 请求（正确的 Headers: User-Agent, Cookie, Referer, Origin 等）。
    *   处理来自 AI Studio 后端的响应（主要是流式响应）。
    *   更新 Cookie 的使用计数和状态。
*   **实现流程**:
    1.  创建类似 `forward_to_aistudio(app_state: Arc<RwLock<AppState>>, request_body: serde_json::Value) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>, ProxyError>` 的函数。
    2.  调用 `manager::cookies::select_available_cookie` 获取 Cookie。
    3.  使用 `reqwest::Client` 构建请求。**关键**: 使用浏览器开发者工具分析 AI Studio 网页交互，复制必要的 Headers（特别是 `User-Agent`, `Cookie`, `Origin`, `Referer`, 以及可能的 `X-Goog-*` Headers）。设置正确的 URL 和请求体（可能需要根据前端请求调整）。
    4.  发送请求并处理响应流，逻辑类似 Gemini 模式，但错误处理可能需要针对 AI Studio 返回的特定错误进行调整（例如 Cookie 失效）。
    5.  失败时调用 `manager::cookies::update_cookie_status` 更新状态。
    6.  返回结果。
*   **测试方案**:
    *   **内容**: 测试请求构建是否包含所有必要的伪装 Headers。测试能否处理成功的流式响应。测试 Cookie 失效等错误场景下是否能正确更新 Cookie 状态。
    *   **工具**:
        *   单元测试 (`#[test]`)：使用 `httpmock` 或 `wiremock-rs` 模拟 AI Studio 后端响应。验证发送的请求头是否符合预期。
        *   集成测试（困难且不稳定）：需要有效 Cookie，且 AI Studio 后端接口可能变化。如果进行，重点是验证能否成功建立连接并接收到数据流。用 `assert!` 检查流内容。

### 单元 A4: 核心代理决策与后台任务

*   **开发目标**:
    *   实现核心请求处理逻辑 (`src/proxy/core.rs`)，根据配置决定调用 Gemini 还是 AI Studio 模式。
    *   实现后台调度器 (`src/manager/scheduler.rs`)，用于每日重置密钥/Cookie 计数和状态检查。
*   **实现流程**:
    1.  **`proxy/core.rs`**: 创建 `process_request(app_state: Arc<RwLock<AppState>>, custom_api_key: String, request_data: RequestData) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>, ProxyError>` 函数。
        *   读取 `AppState` 中的配置，判断当前应使用哪个模式。
        *   调用 A2 或 A3 中实现的相应转发函数 (`forward_to_gemini` 或 `forward_to_aistudio`)。
        *   处理可选的提示词预处理逻辑 (`src/proxy/prompt.rs`)（可以先留空或简单实现）。
        *   返回转发函数的结果。
    2.  **`manager/scheduler.rs`**: 创建 `run_scheduler(app_state: Arc<RwLock<AppState>>)` 异步函数。
        *   使用 `tokio::time::interval` 创建一个定时器（例如，每小时检查一次，或者更精确地计算到下一个太平洋时间午夜）。
        *   在循环中，当到达每日重置时间（例如，检查当前时间是否为太平洋时间午夜），获取 `AppState` 写锁，遍历所有 Keys 和 Cookies，调用 `manager` 中的函数重置它们的计数。
        *   可以添加定期检查 Key/Cookie 可用性的逻辑（例如，发送一个简单请求测试）。
        *   在 `main.rs` 中使用 `tokio::spawn` 启动这个任务。
*   **测试方案**:
    *   **内容**: 测试 `process_request` 能否根据模拟的配置正确路由到 Gemini 或 AI Studio 的（Mocked）实现。测试调度器能否在预定时间触发重置逻辑（可能需要 Mock 时间或在测试中快速推进时间）。测试状态检查逻辑（如果实现）。
    *   **工具**:
        *   单元测试 (`#[test]`)：Mock `AppState` 和 A2/A3 的转发函数，验证 `process_request` 的路由逻辑。对于调度器，可以测试其内部的重置函数，或者使用 `tokio::time::pause()` 和 `advance()` (如果使用 Tokio 的测试工具) 来控制时间。
        *   集成测试：启动应用，等待调度器运行，通过 API（由开发者 B 实现）检查 Key/Cookie 计数是否被重置。

---

## 开发者 B: Web 服务与 API 接口

**主要职责**: 构建 Web 服务器，处理 HTTP 请求，实现 API 路由、认证、静态文件服务、TLS 配置，并与开发者 A 的核心逻辑进行集成。

### 单元 B1: 基础 Web 服务与配置加载

*   **开发目标**:
    *   建立基本的 Axum Web 服务器 (`src/main.rs`)。
    *   实现配置文件的加载 (`src/config.rs`) 和解析 (`toml`)。
    *   设置基本的日志系统 (`src/logger.rs`)。
    *   定义命令行参数 (`src/cli.rs`) 用于指定配置路径、端口等。
*   **实现流程**:
    1.  **`Cargo.toml`**: 确认 `axum`, `tokio`, `serde`, `serde_json`, `toml`, `tracing`, `tracing-subscriber`, `clap` 等依赖已添加。
    2.  **`src/cli.rs`**: 使用 `clap` 定义命令行参数结构，如 `--config <PATH>`, `--port <PORT>`。
    3.  **`src/config.rs`**: 定义 `Config` 结构体，使用 `serde::Deserialize` 派生。包含字段如 `server_port`, `initial_admin_key` (用于首次启动), `gemini_keys` (初始列表), `aistudio_cookies` (初始列表), `log_level`, `tls_cert_path`, `tls_key_path` 等。实现加载 `config.toml` 文件并解析到 `Config` 实例的函数。如果配置文件不存在或特定字段缺失，提供默认值或报错。
    4.  **`src/logger.rs`**: 初始化 `tracing_subscriber`，根据配置设置日志级别，配置日志输出到控制台。暂时不处理文件日志。
    5.  **`src/main.rs`**:
        *   `#[tokio::main]` 函数。
        *   解析命令行参数 (`cli.rs`)。
        *   调用 `logger::init()`。
        *   加载配置 (`config.rs`)。
        *   创建基础的 `axum::Router`，添加一个简单的根路径 `/` 处理器，返回 "Hello, World!"。
        *   绑定到配置的端口，启动服务器。
*   **测试方案**:
    *   **内容**: 测试命令行参数能否正确解析。测试配置文件（包括示例文件 `config.toml.example`）能否被正确加载和解析。测试不同日志级别配置是否生效。服务器能否在指定端口启动并响应根路径请求。
    *   **工具**:
        *   运行 `cargo run -- --help` 检查 CLI。
        *   运行 `cargo run -- --config config.toml.example` 启动服务器。
        *   使用 `curl http://localhost:3200` (或其他配置的端口) 检查响应。
        *   查看控制台输出的日志。
        *   单元测试 (`#[test]`) 测试 `config.rs` 的加载和解析逻辑，特别是处理缺失文件或字段的情况。

### 单元 B2: 自定义 API 密钥认证中间件

*   **开发目标**:
    *   实现 Axum 中间件 (`src/api/middleware.rs`) 来验证请求头中的 `Authorization: Bearer <custom_api_key>`。
    *   首次启动时，如果 `AppState`（或配置中）没有管理员密钥，生成一个并提示用户保存。
    *   验证提供的密钥是否存在于 `AppState` 中（需与开发者 A 的 `AppState` 集成）。
    *   （可选）将验证后的密钥信息添加到请求扩展中，供后续处理器使用。
*   **实现流程**:
    1.  **首次启动密钥生成**: 在 `main.rs` 加载配置后、启动服务器前检查。如果 `AppState` 中没有自定义密钥（这需要访问开发者 A 实现的状态或先在 Config 中管理），使用 `rand` 生成一个安全的随机字符串，打印到控制台，并保存到 `AppState` 或写回配置文件（写回配置较简单，但长期看存入 `AppState` 更好）。
    2.  **`src/api/middleware.rs`**: 创建 `authenticate` 异步函数，它接受 `axum::extract::Request` 和 `axum::middleware::Next`。
        *   从请求头 `headers().get(axum::http::header::AUTHORIZATION)` 提取 Bearer Token。
        *   解析出 `custom_api_key`。
        *   获取 `AppState` 的读锁 (通过 `axum::extract::State` 或 `Extension` 访问 `Arc<RwLock<AppState>>`)。
        *   查询 `AppState` 中是否存在该 `custom_api_key` 并且其状态有效（例如，未被禁用）。(调用开发者 A 的 `manager::keys::get_key` 或类似方法)。
        *   如果验证通过，可以将 key 本身或相关信息放入 `request.extensions_mut().insert(...)`，然后调用 `next.run(request).await`。
        *   如果验证失败（未提供 Token、格式错误、Key 无效），返回 `StatusCode::UNAUTHORIZED` (401) 响应。
        *   释放读锁。
*   **测试方案**:
    *   **内容**: 测试首次启动时是否生成并提示密钥（如果实现）。测试中间件：无 Auth 头、错误格式 Auth 头、无效 Key、有效 Key 的情况。测试有效 Key 是否能通过中间件到达下一层处理器。
    *   **工具**:
        *   单元测试 (`#[test]`): 可以 Mock `AppState` 和 `Next` 来测试中间件逻辑。
        *   集成测试: 在 B1 的基础上，创建一个需要认证的路由 `/api/v1/protected`，应用此中间件。使用 `curl` 或 Postman 发送带不同 `Authorization` 头的请求，验证响应码 (200 或 401)。
            *   `curl http://localhost:3200/api/v1/protected` -> 401
            *   `curl -H "Authorization: Bearer invalidkey" http://localhost:3200/api/v1/protected` -> 401
            *   `curl -H "Authorization: Bearer <valid_key>" http://localhost:3200/api/v1/protected` -> 200 (或被保护路由的响应)

### 单元 B3: 代理 API 端点实现

*   **开发目标**:
    *   创建处理代理请求的 API 路由 (`src/api/routes/proxy.rs`)，例如 `/api/v1/proxy`。
    *   应用 B2 中实现的认证中间件。
    *   解析来自客户端（如 SillyTavern）的 JSON 请求体。
    *   调用开发者 A 实现的核心代理逻辑 (`proxy::core::process_request`)。
    *   将 `process_request` 返回的流式响应转发给客户端。
*   **实现流程**:
    1.  **`src/api/mod.rs`**: 创建 API 路由模块，定义 `/api/v1` 路由组。
    2.  **`src/api/routes/proxy.rs`**: 创建 `handle_proxy_request` 异步处理函数。
        *   函数签名应能接收 `axum::extract::State<Arc<RwLock<AppState>>>` (或通过 `Extension`)，以及 `axum::Json<YourRequestPayload>` 来解析请求体。`YourRequestPayload` 是一个需要定义的结构体，匹配 SillyTavern 或预期客户端发送的 JSON 格式。
        *   从请求扩展中获取认证通过的 `custom_api_key`（如果 B2 中添加了）。
        *   调用 `proxy::core::process_request(app_state, custom_api_key, request_payload)`。**这是关键集成点**。
        *   处理 `process_request` 的 `Result`：
            *   如果是 `Ok(stream)`，将 `reqwest` 的 `Bytes` 流转换为 `axum::body::Body`。设置正确的响应头（如 `Content-Type: text/event-stream`），并返回一个包含此 Body 的 `axum::response::Response`。使用 `Body::from_stream(stream)`。
            *   如果是 `Err(proxy_error)`，根据错误类型记录日志，并返回相应的 HTTP 错误码（如 500 Internal Server Error, 503 Service Unavailable）。
    3.  **`src/api/mod.rs`**: 将 `/api/v1/proxy` 路由（`POST` 方法）绑定到 `handle_proxy_request`，并确保应用了 `authenticate` 中间件。
*   **测试方案**:
    *   **内容**: 测试无认证/错误认证时访问代理端点是否返回 401。测试有效认证下，发送合法的 JSON 请求体：
        *   能否成功调用（Mocked）`process_request`。
        *   能否正确处理 `process_request` 返回的成功（流式）响应，并将流数据传回客户端。
        *   能否正确处理 `process_request` 返回的错误，并转换为适当的 HTTP 错误响应。
    *   **工具**:
        *   集成测试: 使用 `curl` 或 Postman 发送 POST 请求到 `/api/v1/proxy`。
            *   `curl -N -H "Authorization: Bearer <valid_key>" -H "Content-Type: application/json" -d '{"prompt": "Hello"}' http://localhost:3200/api/v1/proxy` (`-N` 用于查看流式输出)。
            *   验证响应头和（流式）响应体是否符合预期。
            *   可以暂时让 Mocked `process_request` 返回一个简单的 SSE 流（几条 `data: ...\n\n` 消息）或错误，来验证 Axum 端的处理。
        *   需要一个模拟客户端发送请求的脚本或工具 (Postman, curl)。

### 单元 B4: Web 前端与管理 API

*   **开发目标**:
    *   提供静态文件服务，托管 `web/` 目录下的前端资源。
    *   实现用于管理自定义 API 密钥、AI Studio Cookie 和查看状态的 RESTful API 端点 (`src/api/routes/management.rs`)，例如 `/api/v1/manage/keys`, `/api/v1/manage/cookies`, `/api/v1/manage/status`。
    *   为这些管理 API 应用认证中间件（可能需要区分管理员权限）。
    *   （可选）实现 Web 前端的简单 Direct Chat 功能所需的 API（可能复用 B3 的代理端点）。
*   **实现流程**:
    1.  **静态文件服务**: 在 `src/main.rs` 或 `src/api/mod.rs` 中，配置 `tower_http::services::ServeDir::new("web")` 来处理所有未被 API 路由匹配的 GET 请求，并将其指向 `web/` 目录。设置 fallback 到 `web/index.html` 以支持 SPA (Single Page Application) 路由。
    2.  **`src/api/routes/management.rs`**: 创建多个处理函数：
        *   `list_keys()`: GET `/api/v1/manage/keys` -> 调用 `manager::keys::get_all_keys` (需开发者 A 实现)，返回 JSON 列表。
        *   `create_key()`: POST `/api/v1/manage/keys` -> (可能需要管理员权限) 解析请求体（如 key 名称、限制），调用 `manager::keys::add_key`，返回新创建的 key 信息。
        *   `delete_key()`: DELETE `/api/v1/manage/keys/:key_id` -> (管理员权限) 调用 `manager::keys::delete_key`。
        *   类似地实现 Cookie 的 List, Add, Delete API (`/api/v1/manage/cookies`)。
        *   `get_status()`: GET `/api/v1/manage/status` -> 从 `AppState` 读取并聚合状态信息（如模式、Key/Cookie 数量和状态概览），返回 JSON。
    3.  **`src/api/mod.rs`**: 将这些管理路由添加到 Router 中，并应用认证中间件。可以考虑为写操作（POST, DELETE）添加额外的权限检查（例如，只允许初始管理员 Key 操作）。
    4.  **前端 (`web/`)**: 创建简单的 HTML, CSS, JS 文件。JS 需要能：
        *   提示用户输入他们的自定义 API Key 并保存到 localStorage。
        *   使用保存的 Key 发送请求到 `/api/v1/manage/*` 端点来获取和展示数据。
        *   提供表单来添加新的 Key/Cookie。
        *   提供一个简单的聊天输入框和显示区域，将用户输入包装成符合代理端点（B3）要求的 JSON，发送到 `/api/v1/proxy`，并处理流式响应展示结果。
*   **测试方案**:
    *   **内容**: 测试访问 `/` 或其他不存在的路径是否返回 `web/index.html`。测试浏览器能否加载 HTML, CSS, JS。测试管理 API：无认证/错误认证返回 401。使用有效 Key 能否成功调用 List, Add, Delete 操作并影响 `AppState`（通过再次 List 验证）。测试状态 API 返回的数据是否合理。前端页面能否与后端 API 正确交互。
    *   **工具**:
        *   浏览器: 直接访问 `http://localhost:3200`，测试前端功能。使用开发者工具查看网络请求和响应。
        *   `curl` / Postman: 测试管理 API 的 CRUD 操作。
            *   `curl -H "Authorization: Bearer <admin_key>" http://localhost:3200/api/v1/manage/keys`
            *   `curl -X POST -H "Authorization: Bearer <admin_key>" -H "Content-Type: application/json" -d '{"name": "test-key"}' http://localhost:3200/api/v1/manage/keys`
        *   前端自动化测试（如果项目规模允许，例如使用 Cypress 或 Playwright），但对于 Rust 初学者项目，手动测试可能更实际。

### 单元 B5: TLS 配置与日志文件

*   **开发目标**:
    *   为服务器启用 HTTPS。支持使用自签名证书（自动生成）或用户提供的证书文件。
    *   实现将日志输出到文件的功能，特别是按自定义 API Key 分割的请求/回复日志。
*   **实现流程**:
    1.  **TLS (`src/tls.rs`)**:
        *   添加 `rcgen` 依赖用于生成自签名证书。
        *   添加 `rustls`, `tokio-rustls`, `rustls-pemfile` 依赖。
        *   在 `tls.rs` 中创建函数，检查配置中是否指定了证书和私钥路径 (`tls_cert_path`, `tls_key_path`)。
        *   如果指定了路径且文件存在，加载它们。使用 `rustls_pemfile` 读取 PEM 文件。
        *   如果没有指定路径或文件不存在，使用 `rcgen` 生成一个自签名证书和私钥。可以将生成的证书/私钥保存到磁盘（例如 `certificates/generated_cert.pem`, `certificates/generated_key.pem`）供下次启动使用，或者仅在内存中使用。
        *   返回一个 `rustls::ServerConfig`。
    2.  **`main.rs`**:
        *   导入 `axum_server::tls_rustls::RustlsConfig`。
        *   调用 `tls.rs` 中的函数获取 `rustls::ServerConfig`。
        *   将 `rustls::ServerConfig` 包装在 `RustlsConfig::from_config()` 中。
        *   在启动服务器时使用 `.bind_rustls(addr, rustls_config)` 而不是 `.bind(addr)`。
    3.  **文件日志 (`src/logger.rs`)**:
        *   添加 `tracing-appender` 依赖。
        *   修改 `logger::init`：除了控制台输出 (`fmt::layer()`)，再添加一个文件输出层。
        *   **系统日志**: 使用 `tracing_appender::rolling::daily("./logs", "proxy.log")` 创建一个每日轮转的日志文件写入器，用于记录应用自身的运行日志（启动、配置加载、错误等）。将其与 `fmt::layer().with_writer(...)` 结合。
        *   **请求/回复日志**: 这比较复杂，因为需要在请求处理过程中动态决定写入哪个文件。
            *   **方案一 (简单，可能有并发问题)**: 在认证中间件 (B2) 或代理处理器 (B3) 中获取 `custom_api_key`。使用 `std::fs::OpenOptions` 以追加模式打开或创建对应的 `{key}_request.log` 和 `{key}_reply.log` 文件。直接写入格式化的日志字符串（包含时间戳和分隔符 `------`）。需要注意文件句柄的管理和并发写入冲突。
            *   **方案二 (推荐，使用 Tracing)**: 创建一个自定义的 `tracing_subscriber::Layer`。在这个 Layer 的 `on_event` 或 `on_record` 方法中，检查事件的 target 或字段是否包含 `custom_api_key` 信息（需要在记录日志时手动加入，例如 `tracing::info!(target: "api_log", key = %custom_api_key, direction = "request", ...)`）。根据 `key` 和 `direction` (request/reply) 将格式化后的日志写入对应的文件。这需要更深入理解 `tracing` 生态。可以使用 `tracing_appender` 来管理文件写入器。
        *   将所有配置的 Layer 通过 `tracing_subscriber::registry().with(layer1).with(layer2)...` 组合起来。
*   **测试方案**:
    *   **内容**: 测试服务器是否能以 HTTPS 启动。使用自签名证书时，浏览器是否提示不安全（正常现象）。使用用户提供的证书时是否正常工作。系统日志文件 (`proxy.log`) 是否生成并记录了启动信息。发送代理请求后，对应的 `{key}_request.log` 和 `{key}_reply.log` 是否生成，内容格式是否正确（包含分隔符），多次请求是否追加写入。
    *   **工具**:
        *   浏览器: 访问 `https://localhost:3200` (注意是 https)。接受自签名证书风险。
        *   `curl`: 使用 `curl -k https://localhost:3200` (`-k` 忽略证书验证)。
        *   文件系统: 检查 `logs/` 目录下是否生成了预期的日志文件，查看其内容。
        *   检查控制台输出，确认没有 TLS 或日志相关的错误。

---

开发者 A 专注于核心业务逻辑和状态维护，开发者 B 专注于对外接口和基础设施。在单元 A1 和 B1/B2 完成后，就可以开始初步集成 `AppState` 和认证。在 A2/A3 和 B3 完成后，就可以进行端到端的代理功能测试。A4 和 B4/B5 可以在主体功能完成后继续完善。