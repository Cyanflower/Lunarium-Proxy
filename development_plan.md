## Gemini 反向代理开发计划

**开发者职责:**

*   **Developer A:** 专注于核心业务逻辑、状态管理、异步任务协调（`KeyManager`, `Dispatcher`, `Handlers`, `models.rs`, `prompt.rs`）。
*   **Developer B:** 专注于对外接口、基础设施、配置、构建、部署、服务器设置（`main.rs`, `cli.rs`, `config.rs`, `state.rs`, `error.rs`, `logger.rs`, `tls.rs`, `proxy.rs`, `router.rs`, 管理 API）。

---

### 阶段一：项目基础架构与信息调查 (Foundation & Investigation)

**目标:** 搭建项目骨架，定义核心数据结构，配置加载，并建立最基础的 Web 服务。

**单元 1.1: 核心数据结构定义 (`models.rs`)**

*   **开发者:** A
*   **实现目标:** 定义项目核心的共享数据结构和枚举。
*   **流程:**
    1.  在 `models.rs` 中定义:
        *   `KeyType` enum (`Gemini`, `AIStudio`).
        *   `Key` enum (包含 `GeminiApiKeyInfo` struct, `AIStudioCookieInfo` struct)。考虑包含Key的ID、值、状态等。
        *   `KeyStatus` enum (`Available`, `InUse`, `Exhausted`, `Invalid`, `Disabled`).
        *   `Reason` enum (用于 `KeyManager` 回收 Key 时说明原因，如 `Success`, `RateLimited`, `AuthError`, `NetworkError`, `Unknown`).
        *   `DispatchRequest` struct (包含 `path: String`, `query: QueryParamsMap`, `body: serde_json::Value`, `target_key: Key`, `response_sender: oneshot::Sender<HandlerResponse>`, `key_return_tx: mpsc::Sender<(Key, Option<Reason>)>`). (注意 `QueryParamsMap` 可以是 `HashMap<String, String>` 或类似类型)。
        *   `HandlerResponse` type alias: `Result<reqwest::Response, HandlerError>`.
*   **如何测试:**
    *   `cargo check`: 确保类型定义编译无误。
    *   Code Review: 检查数据结构是否清晰、完整，符合设计要求。

**单元 1.2: 配置加载与命令行参数 (`config.rs`, `cli.rs`, `main.rs`)**

*   **开发者:** B
*   **实现目标:** 实现配置文件的加载（使用 `serde`）和命令行参数的解析（使用 `clap`）。
*   **流程:**
    1.  在 `config.rs` 中:
        *   定义 `Config` struct，使用 `serde::Deserialize`。包含字段如 `gemini_base_url`, `aistudio_base_url`, `management_key`, `listen_address`, `tls` (可选配置), `keys` (`Vec<KeyConfig>`) 等。
        *   定义 `KeyConfig` struct/enum 用于表示配置文件中的 Key/Cookie。
        *   实现加载 `config.toml` 的函数 `load_config(path: &Path) -> Result<Config, Error>`。
    2.  在 `cli.rs` 中:
        *   使用 `clap` 定义命令行参数结构，至少包含 `--config` 选项指定配置文件路径。
    3.  在 `main.rs` 中:
        *   解析命令行参数。
        *   调用 `load_config` 加载配置。
        *   将加载的 `Config` 存储起来（例如，放入 `Arc` 以便共享）。
*   **如何测试:**
    *   创建 `config.toml.example` 和一个测试用的 `config.toml`。
    *   运行 `cargo run -- --help` 检查 Clap 输出。
    *   运行 `cargo run -- --config test_config.toml`，在 `main.rs` 中打印加载的配置，验证其正确性。
    *   测试配置文件不存在或格式错误时的错误处理。

**单元 1.3: 日志与错误处理 (`logger.rs`, `error.rs`)**

*   **开发者:** B
*   **实现目标:** 初始化 `tracing` 日志系统，定义统一的自定义错误类型。
*   **流程:**
    1.  在 `logger.rs` 中:
        *   实现 `init_logger()` 函数，配置 `tracing_subscriber`（例如，设置日志级别、输出格式、可选的文件输出）。
    2.  在 `error.rs` 中:
        *   使用 `thiserror` 定义主要的 `AppError` enum，涵盖配置加载错误、I/O 错误、网络错误、内部通道错误等。
        *   定义 `HandlerError` enum，用于 `Handlers` 返回给 `proxy.rs` 的特定错误（如 `RequestFailed`, `PromptProcessingFailed`, `BackendAuthError`, `BackendRateLimited`).
    3.  在 `main.rs` 中调用 `init_logger()`。
*   **如何测试:**
    *   在 `main.rs` 和其他模块中添加一些 `tracing::info!`, `tracing::warn!`, `tracing::error!` 日志。
    *   运行程序，检查控制台输出是否符合预期格式和级别。
    *   尝试触发配置加载错误，检查错误类型和日志。

**单元 1.4: 基础 Web 服务与路由 (`main.rs`, `router.rs`, `state.rs`, `tls.rs`)**

*   **开发者:** B
*   **实现目标:** 启动一个基本的 Axum Web 服务器，定义应用状态 `AppState`，并设置基础路由和可选的 TLS。
*   **流程:**
    1.  在 `state.rs` 中:
        *   定义 `AppState` struct，包含共享状态，如 `config: Arc<Config>`，以及后续会添加的通道发送端 `*_tx`。
        *   实现 `AppState::new(...)`。
    2.  在 `tls.rs` 中 (可选):
        *   实现加载证书和私钥并配置 `rustls` 的辅助函数，返回 `axum_server::tls_rustls::RustlsConfig`。
    3.  在 `router.rs` 中:
        *   创建 `create_router(app_state: AppState) -> Router` 函数。
        *   添加一个简单的根路由 `/` 返回 "OK" (`axum::routing::get("/", || async { "OK" })`)。
        *   (占位) 添加代理路径前缀 `/api/v1beta/*` 的占位路由。
    4.  在 `main.rs` 中:
        *   创建 `AppState` 实例。
        *   调用 `create_router` 构建 Axum Router。
        *   根据配置决定是启动 HTTP 还是 HTTPS 服务器 (`axum::Server` 或 `axum_server::bind_rustls`)。
        *   启动服务器并 `await` 它。
*   **如何测试:**
    *   `cargo run`: 启动服务器。
    *   使用 `curl http://<listen_address>/` (或 `curl -k https://...` 如果启用了自签名 TLS)，应返回 "OK"。
    *   检查日志确认服务器已启动。

---

### 阶段二：核心异步逻辑实现 (Core Async Logic)

**目标:** 实现 `KeyManager` 和 `Dispatcher` 的核心功能，以及 Handler 的基本框架。

**单元 2.1: `KeyManager` - 状态与基础循环**

*   **开发者:** A
*   **实现目标:** 实现 `KeyManager` 的基本结构、状态存储、以及处理凭证请求和回收的异步循环框架。
*   **流程:**
    1.  在 `key_manager.rs` 中:
        *   定义 `KeyManager` struct，包含凭证存储（例如 `HashMap<KeyId, (Key, KeyStatus)>` 或按类型分开存储）、接收凭证请求的 `mpsc::Receiver<(KeyType, oneshot::Sender<Result<Key, AppError>>)>`、接收凭证回收的 `mpsc::Receiver<(Key, Option<Reason>)>`。
        *   实现 `KeyManager::new(config: Arc<Config>, key_request_rx, key_return_rx)`，从配置初始化凭证池。
        *   实现 `async fn run(mut self)` 作为主循环。
        *   在 `run` 中使用 `tokio::select!` 同时监听 `key_request_rx` 和 `key_return_rx`。
        *   实现 `handle_key_request(&mut self, key_type: KeyType, response_tx: oneshot::Sender<...>)`: 查找可用 Key，标记为 `InUse`，通过 `response_tx` 发回。如果无可用 Key，则发送错误或放入等待队列（初期可以先返回错误）。
        *   实现 `handle_key_return(&mut self, key: Key, reason: Option<Reason>)`: 根据 `reason` 更新 Key 的状态（`Available`, `Exhausted`, `Invalid` 等）。
*   **如何测试:**
    *   **单元测试:**
        *   创建 `KeyManager` 实例（使用模拟配置和通道）。
        *   发送请求消息，检查是否能通过 `oneshot` 接收到预期的 Key，并检查内部状态是否变为 `InUse`。
        *   发送回收消息，检查内部状态是否按预期更新 (e.g., `Available`, `Exhausted`)。
        *   测试无可用 Key 时的情况。
    *   **集成 (初步):**
        *   (B 协助) 在 `main.rs` 中创建 `KeyManager` 所需的通道 (`mpsc::channel`, `oneshot::channel` 用于测试请求)。
        *   (B 协助) `tokio::spawn` `KeyManager::run` 任务。
        *   (B 协助) 在 `main.rs` 中手动发送请求和回收消息到通道，观察 `KeyManager` 的日志输出和行为。

**单元 2.2: `Dispatcher` - 接收与任务分发框架**

*   **开发者:** A
*   **实现目标:** 实现 `Dispatcher` 的基本结构和接收请求并根据 Key 类型分发任务的框架（此时 Handler 仍是空实现或打印日志）。
*   **流程:**
    1.  在 `dispatcher.rs` 中:
        *   定义 `Dispatcher` struct，包含接收分发请求的 `mpsc::Receiver<DispatchRequest>`。
        *   实现 `Dispatcher::new(dispatch_rx)`。
        *   实现 `async fn run(mut self, key_return_tx: mpsc::Sender<(Key, Option<Reason>)>, config: Arc<Config>)` 作为主循环。
        *   在 `run` 中循环接收 `DispatchRequest`。
        *   根据 `msg.target_key` 的类型 (`Key::Gemini` 或 `Key::AIStudio`)，准备调用相应的 Handler 函数。
        *   **核心:** 使用 `tokio::spawn` 启动一个新的异步任务来执行 Handler 函数，并将 `msg` 中的数据 (Key, path, query, body, response_sender) 以及 `key_return_tx.clone()` 和 `config.clone()` **移动** (move) 到新任务中。
*   **如何测试:**
    *   **单元测试:**
        *   创建 `Dispatcher` 实例（使用模拟通道）。
        *   创建模拟的 `DispatchRequest` 消息 (包含有效的 `oneshot::Sender` 和 `mpsc::Sender`)。
        *   发送消息到 `Dispatcher`。
        *   **难点:** 直接测试 `tokio::spawn` 比较困难。可以通过**日志**确认 `Dispatcher` 接收到消息并尝试根据类型进行分发。或者，在 Handler 的 dummy 实现中发送一个简单的确认信号回 `oneshot` 通道，并在测试中等待这个信号。
    *   **集成 (初步):**
        *   (B 协助) 在 `main.rs` 中创建 `Dispatcher` 所需的通道 (`mpsc::channel`)。
        *   (B 协助) `tokio::spawn` `Dispatcher::run` 任务，传入 `KeyManager` 的 `key_return_tx`。
        *   (B 协助) 在 `main.rs` 中手动构造 `DispatchRequest` (需要 `oneshot` 和来自 `KeyManager` 的 `key_return_tx`) 并发送给 `Dispatcher`，观察日志。

**单元 2.3: `Handlers` - 基础框架与提示词处理桩 (`handlers/mod.rs`, `gemini.rs`, `aistudio.rs`, `prompt.rs`)**

*   **开发者:** A
*   **实现目标:** 创建 Handler 函数的基本签名和结构，实现提示词处理函数的桩（stub）。Handler 暂时不执行实际 HTTP 请求，仅模拟成功或失败，并正确回传响应和凭证状态。
*   **流程:**
    1.  在 `handlers/prompt.rs` 中:
        *   定义 `process_gemini_prompt(body: serde_json::Value, _prompt_config: &PromptConfig) -> Result<serde_json::Value, HandlerError>` 函数（`PromptConfig` 可以在 `config.rs` 中定义）。
        *   初始实现：直接返回 `Ok(body)`。
        *   类似地定义 `process_aistudio_prompt`。
    2.  在 `handlers/gemini.rs` 中:
        *   定义 `async fn process_gemini_request(api_key_info: GeminiApiKeyInfo, path: String, query: QueryParamsMap, mut body: serde_json::Value, response_sender: oneshot::Sender<HandlerResponse>, key_return_tx: mpsc::Sender<(Key, Option<Reason>)>, config: Arc<Config>)`。
        *   调用 `handlers::prompt::process_gemini_prompt` (虽然现在是 stub)。
        *   **模拟处理:**
            *   打印日志说明正在处理 Gemini 请求。
            *   **不创建 `reqwest::Client` 或发送请求。**
            *   随机或固定地决定一个结果（模拟成功或失败）。
            *   构造 `reason: Option<Reason>` (e.g., `Some(Reason::Success)` 或 `Some(Reason::AuthError)`)。
            *   发送回收消息: `let _ = key_return_tx.send((Key::Gemini(api_key_info), reason)).await;`
            *   发送响应消息:
                *   成功: `let _ = response_sender.send(Err(HandlerError::RequestFailed("Not Implemented Yet".to_string())));` // 暂时发送错误，因为无法构造 Response
                *   失败: `let _ = response_sender.send(Err(HandlerError::RequestFailed("Simulated Failure".to_string())));`
    3.  在 `handlers/aistudio.rs` 中:
        *   类似地实现 `process_aistudio_request` 的框架和模拟处理。
    4.  在 `handlers/mod.rs` 中导出这两个函数。
*   **如何测试:**
    *   **单元测试:**
        *   测试 `prompt.rs` 中的 stub 函数。
        *   **困难:** 直接单元测试 Handler 比较复杂，因为它依赖 `oneshot` 和 `mpsc` 通道以及异步运行时。可以通过创建模拟的 Sender/Receiver 来进行，但这比较繁琐。
        *   **主要依赖集成测试:** 依赖下一阶段的集成。
    *   **集成 (通过 Dispatcher):**
        *   在 `Dispatcher` 单元测试或 `main.rs` 集成测试中，发送 `DispatchRequest` 后，等待 `oneshot` 通道返回结果，并检查结果是否符合 Handler 模拟的成功/失败情况。
        *   检查 `KeyManager` 是否收到了正确的回收消息和 `Reason`。

---

### 阶段三：端到端流程打通与实际请求 (End-to-End Flow & Real Requests)

**目标:** 连接 `proxy.rs` 到 `KeyManager` 和 `Dispatcher`，实现 Handler 中实际的 HTTP 请求，并将响应流式传输回客户端。

**单元 3.1: `proxy.rs` - 连接核心逻辑**

*   **开发者:** B
*   **实现目标:** 实现 `handle_proxy_request` Axum Handler，使其能与 `KeyManager` 和 `Dispatcher` 正确交互。
*   **流程:**
    1.  修改 `handle_proxy_request(State(app_state): State<AppState>, req: Request<Body>) -> impl IntoResponse`:
    2.  从 `app_state` 获取 `key_request_tx` 和 `dispatch_tx`。
    3.  解析 `req`: 获取路径、查询参数（过滤掉认证 `key`）、读取请求体 (`axum::body::to_bytes` 或流式处理，取决于后端是否需要完整 Body）。
    4.  **确定 KeyType:** 根据路径或配置决定需要 `KeyType::Gemini` 还是 `KeyType::AIStudio`。
    5.  **请求凭证:**
        *   创建 `oneshot::channel` (`key_response_tx`, `key_response_rx`)。
        *   向 `KeyManager` 发送 `(key_type, key_response_tx)`: `app_state.key_request_tx.send(...).await?`。
        *   等待凭证返回: `let acquired_key_result = key_response_rx.await?`。处理获取凭证失败的情况 (e.g., 返回 503 Service Unavailable)。
    6.  **准备分发:**
        *   创建 `oneshot::channel` (`http_response_tx`, `http_response_rx`) 用于接收 Handler 结果。
        *   获取 `KeyManager` 的 `key_return_tx` (需要从 `AppState` 获取或克隆)。
    7.  创建 `DispatchRequest` 消息，包含获取到的 `Key`、请求信息、`http_response_tx` 和 `key_return_tx.clone()`。
    8.  **发送到 Dispatcher:** `app_state.dispatch_tx.send(dispatch_request).await?`。处理发送失败的情况。
    9.  **等待 Handler 响应:** `let handler_result = http_response_rx.await?`。处理接收失败的情况。
    10. **构建 Axum 响应:**
        *   根据 `handler_result: Result<reqwest::Response, HandlerError>` 构建响应。
        *   成功 (`Ok(response)`): 将 `reqwest::Response` 的状态码、头信息和 Body 转换为 `axum::Response`。**关键:** 正确处理流式 Body (`response.bytes_stream()`) 转换为 Axum 的 Body 类型。
        *   失败 (`Err(handler_error)`): 记录错误并返回合适的 HTTP 错误码（如 500, 502, 400 等）。
*   **如何测试:**
    *   **集成测试 (关键):** 这是第一个可以进行较完整端到端流程测试的单元。
        *   启动包含 `KeyManager` (单元 2.1) 和 `Dispatcher` (单元 2.2, 调用单元 2.3 的模拟 Handler) 的完整应用。
        *   使用 `curl` 发送代理请求到 `/api/v1beta/...` (带认证 key)。
        *   **验证点:**
            *   请求是否通过认证。
            *   `proxy.rs` 是否成功向 `KeyManager` 请求并获取到 Key (检查日志)。
            *   `proxy.rs` 是否成功将请求发送给 `Dispatcher` (检查日志)。
            *   `Dispatcher` 是否 spawn 了正确的 (模拟) Handler (检查日志)。
            *   (模拟) Handler 是否将 Key 返回给 `KeyManager` (检查 `KeyManager` 日志)。
            *   `proxy.rs` 是否收到了来自 (模拟) Handler 的响应 (检查日志)。
            *   `curl` 是否收到了预期的 HTTP 响应（目前应该是来自模拟 Handler 的错误信息）。

**单元 3.2: `Handlers` - 实现实际 HTTP 请求 (`gemini.rs`, `aistudio.rs`)**

*   **开发者:** A
*   **实现目标:** 在 Handler 函数中创建独立的 `reqwest::Client`，构造并发送实际的 HTTP 请求到后端服务，处理响应或错误。
*   **流程:**
    1.  在 `process_gemini_request` (及 `process_aistudio_request`) 中:
    2.  **(可选) 调用 `prompt.rs` 处理函数。** (如果 `prompt.rs` 逻辑已初步实现)
    3.  **创建独立 Client:** `let client = reqwest::Client::new();` (或配置一些超时、代理等)。
    4.  **构造目标 URL:** 组合 `config.gemini_base_url` (或 `aistudio_base_url`)、传入的 `path`、`query`。
    5.  **添加认证:**
        *   Gemini: 将 `api_key_info.key_value` 添加为 URL 查询参数 `?key=...`。注意处理已有查询参数的情况。
        *   AI Studio: 将 `cookie_info.cookie_value` 添加到请求的 `Cookie` Header 中。
    6.  **处理流式请求参数:** 检查原始请求中是否有 `alt=sse` 等参数，并确保它们被传递到目标请求。
    7.  **执行请求:**
        ```rust
        let result = client
            .request(original_method, target_url) // Use original method
            .headers(original_headers) // Forward relevant headers (Content-Type, Accept, etc.)
            .body(body_bytes) // Send the potentially modified body
            .send()
            .await;
        ```
        (需要调整 Handler 签名以接收原始请求方法、头信息、处理过的 Body 字节)。
    8.  **处理结果:**
        *   `match result { Ok(response) => {...}, Err(e) => {...} }`
    9.  **判断凭证状态 `Reason`:**
        *   `Ok(response)`: 检查 `response.status()`:
            *   `2xx`: `Some(Reason::Success)`
            *   `401/403`: `Some(Reason::AuthError)`
            *   `429`: `Some(Reason::RateLimited)`
            *   其他 `4xx/5xx`: `Some(Reason::Unknown)` (或更具体的错误)
        *   `Err(e)`: 判断是否是网络/超时错误 (`Some(Reason::NetworkError)`) 或其他 (`Some(Reason::Unknown)`).
    10. **返回凭证给 KeyManager:** `let _ = key_return_tx.send((Key::Gemini(...), reason)).await;`
    11. **发送结果给 `proxy.rs`:**
        *   `Ok(response)`: `let _ = response_sender.send(Ok(response));` (将 `reqwest::Response` 发回)。
        *   `Err(e)`: `let _ = response_sender.send(Err(HandlerError::RequestFailed(e.to_string())));`
*   **如何测试:**
    *   **Mocking (推荐):** 使用 `wiremock-rs` 或类似的库创建一个模拟的 Gemini/AI Studio 后端服务。
        *   配置 Mock 服务器响应不同的状态码 (200, 401, 429) 和 Body (包括流式)。
        *   在测试中将 `config.gemini_base_url` 指向 Mock 服务器。
        *   运行完整的集成测试 (`curl` -> proxy -> handler -> mock)。
        *   验证:
            *   Mock 服务器是否收到了带有正确认证信息（API Key/Cookie）和 Body 的请求。
            *   Handler 是否根据 Mock 响应正确判断了 `Reason` 并返回给 `KeyManager`。
            *   Handler 是否将 Mock 的 `reqwest::Response` 或 `HandlerError` 正确发送回 `proxy.rs`。
    *   **Real Services (谨慎):** 使用 **测试用** 的 Gemini API Key 或 AI Studio Cookie。
        *   向真实后端发送请求。
        *   **风险:** 消耗配额，可能不稳定。
        *   **验证:** 检查 `curl` 的输出是否是来自 Google 的真实响应（或错误）。检查 `KeyManager` 中的 Key 状态是否符合预期（需要添加查询 Key 状态的接口或增强日志）。

**单元 3.3: `proxy.rs` - 流式响应处理**

*   **开发者:** B
*   **实现目标:** 确保 `proxy.rs` 能正确地将从 Handler 收到的 `reqwest::Response`（特别是流式 Body）转换为 `axum::Response` 并流式传输给客户端。
*   **流程:**
    1.  在 `handle_proxy_request` 处理 `Ok(response)` 的分支中:
    2.  获取 `response.status()` 和 `response.headers()`。
    3.  获取流式 Body: `let body_stream = response.bytes_stream();`
    4.  创建 Axum Body: `let axum_body = axum::body::Body::from_stream(body_stream);` (可能需要处理 `reqwest::Error` in stream)。
    5.  构建 `axum::Response`:
        ```rust
        let mut axum_response = axum::Response::builder()
            .status(status)
            .body(axum_body)
            .unwrap(); // Or handle error
        *axum_response.headers_mut() = headers; // Copy headers
        Ok(axum_response)
        ```
*   **如何测试:**
    *   **集成测试 (使用 Mock 或 Real Service):**
        *   确保后端 (Mock 或 Real) 配置为返回 SSE (Server-Sent Events) 或其他流式响应 (e.g., `alt=sse` for Gemini)。
        *   使用 `curl -N http://...` ( `-N` 禁用缓冲) 发送请求。
        *   **验证:** 观察 `curl` 的输出是否是持续的、分块的流式数据，而不是一次性返回所有内容。检查响应头是否正确 (e.g., `Content-Type: text/event-stream`)。

**单元 3.4: `Handlers` - 实现 `prompt.rs` 逻辑**

*   **开发者:** A
*   **实现目标:** 在 `prompt.rs` 中实现实际的提示词修改/重组逻辑，并由 Handler 调用。
*   **流程:**
    1.  在 `config.rs` 中定义 `PromptConfig` 结构，包含规则（例如，前缀、后缀、替换规则等）。
    2.  在 `prompt.rs` 中实现 `process_gemini_prompt` 和 `process_aistudio_prompt` 的具体逻辑，根据 `PromptConfig` 修改传入的 `serde_json::Value` (通常是请求体中的 `contents` 字段)。注意错误处理，返回 `Result<_, HandlerError>`。
    3.  确保 Handler (`gemini.rs`, `aistudio.rs`) 正确调用这些函数并处理其结果。
*   **如何测试:**
    *   **单元测试:** 编写详细的单元测试覆盖 `prompt.rs` 中的各种修改规则和边缘情况。
    *   **集成测试:**
        *   配置 `config.toml` 中的提示词修改规则。
        *   使用 Mock 服务器验证发送到后端的请求体是否已被正确修改。
        *   使用 `curl` 发送请求，检查最终响应是否符合预期（如果修改影响了 LLM 的回答）。

---

### 阶段四：高级功能与管理接口 (Advanced Features & Management APIs)

**目标:** 实现更精细的 Key 管理逻辑，并为前端提供管理 API。

**单元 4.1: `KeyManager` - 高级状态管理**

*   **开发者:** A
*   **实现目标:** 实现更复杂的 Key 状态转换逻辑，如冷却期（Cooldown）和自动禁用。
*   **流程:**
    1.  修改 `KeyStatus` enum，可能添加 `CoolingDown(Instant)` 状态。
    2.  修改 `handle_key_return`:
        *   当收到 `RateLimited` 或某些网络错误时，将 Key 状态设为 `CoolingDown(Instant::now() + cooldown_duration)`.
        *   当收到 `AuthError` 时，将 Key 状态设为 `Invalid` 并可能记录日志。
    3.  修改 `handle_key_request`:
        *   在查找可用 Key 时，跳过状态为 `Invalid`, `Disabled`, `CoolingDown` (且未到期) 的 Key。
    4.  **(可选) 添加后台任务:** `KeyManager` 可以有一个定时的内部任务（使用 `tokio::time::interval`）来检查 `CoolingDown` 的 Key 是否已到期，并将其状态改回 `Available`。
*   **如何测试:**
    *   **单元测试:** 重点测试状态转换逻辑：
        *   模拟收到 RateLimit，检查状态是否变为 CoolingDown 及时间戳。
        *   模拟等待超过冷却期，再请求 Key，检查是否能获取到。
        *   模拟收到 AuthError，检查状态是否变为 Invalid。
    *   **集成测试:** 通过 Mock 服务器模拟 429 或 401 响应，观察 `KeyManager` 日志和后续请求的行为。

**单元 4.2: 管理 API - 配置与凭证 CRUD**

*   **开发者:** B
*   **实现目标:** 在 `router.rs` 中添加 Axum 路由和处理函数，用于管理配置和凭证。
*   **流程:**
    1.  **思考:** 如何安全地更新运行时配置和 Key 列表？
        *   **简单方法:** API 仅读取当前配置和 Key 状态（来自 `AppState` 和查询 `KeyManager`）。修改需要重启服务并更改 `config.toml`。
        *   **复杂方法:** API 需要与 `KeyManager` 交互（通过新的管理通道）来动态添加/删除/禁用 Key，并可能需要更新 `AppState` 中的 `Arc<Config>`（这比较棘手，可能需要 `RwLock` 或其他同步机制）。还需要考虑持久化更改到 `config.toml`。 **建议初期采用简单方法。**
    2.  在 `router.rs` 中添加新的路由，例如:
        *   `GET /admin/config`: 返回部分配置信息 (过滤敏感信息)。
        *   `GET /admin/keys`: 返回所有 Key 的列表及其当前状态 (需要向 `KeyManager` 发送查询请求)。
        *   `POST /admin/keys/gemini`: (复杂方法) 添加新的 Gemini Key。
        *   `PUT /admin/keys/gemini/:id/status`: (复杂方法) 启用/禁用 Key。
        *   ... 其他 CRUD 操作 ...
    3.  **实现 Handler:** 编写对应的 Axum Handler 函数。
        *   读取配置: 从 `AppState` 获取 `Arc<Config>`。
        *   获取 Key 状态: 需要为 `KeyManager` 添加一个新的请求类型（例如 `GetStatusRequest`），通过管理通道发送给 `KeyManager`，并等待 `oneshot` 响应返回 Key 状态列表。
        *   修改操作 (复杂方法): 发送特定命令消息给 `KeyManager`。
    4.  **认证:** 确保所有 `/admin/*` 路由都受到严格的认证保护（例如，使用与代理不同的、更强的管理 Key 或其他认证机制）。
*   **如何测试:**
    *   使用 `curl` 或 Postman 等工具。
    *   测试 GET 请求是否能返回正确的配置和 Key 状态信息。
    *   测试认证是否有效，未授权请求是否被拒绝。
    *   (复杂方法) 测试添加/修改 Key 的 API 是否能正确更新 `KeyManager` 的状态（通过日志或后续的 GET 请求验证）。

**单元 4.3: 管理 API - 模式切换与状态监控**

*   **开发者:** B (可能需要 A 配合修改 `KeyManager`)
*   **实现目标:** 提供 API 用于切换 Key 选择策略和查看 KeyManager 状态统计。
*   **流程:**
    1.  **模式切换:**
        *   在 `Config` 或 `AppState` 中添加一个字段表示当前的 Key 选择模式 (e.g., `PreferGemini`, `PreferAIStudio`).
        *   添加 `PUT /admin/mode` API 来修改这个模式。
        *   (A 配合) 修改 `KeyManager` 的 `handle_key_request` 逻辑，使其根据当前模式优先选择对应类型的 Key。
    2.  **状态监控:**
        *   (A 配合) 增强 `KeyManager` 的 `GetStatusRequest` 处理逻辑，使其能计算并返回各类 Key 的数量统计 (Available, InUse, Exhausted, Invalid, etc.)。
        *   添加 `GET /admin/status` API，调用 `KeyManager` 获取状态统计并返回 JSON。
*   **如何测试:**
    *   使用 `curl` 或 Postman。
    *   测试 `/admin/mode` API 是否能成功切换模式，并通过日志或后续代理请求的行为验证模式是否生效。
    *   测试 `/admin/status` API 是否返回正确的 Key 统计数据。

**单元 4.4: 管理 API - 日志查看/导出与 Direct Chat**

*   **开发者:** B
*   **实现目标:** 提供 API 用于查看日志和直接测试聊天功能。
*   **流程:**
    1.  **日志查看:**
        *   **简单方法:** 如果日志写入文件，提供一个 API 下载日志文件。
        *   **复杂方法:** 实现 WebSocket 端点 (`/admin/logs/ws`)，当 `tracing` 记录日志时，将日志消息广播给所有连接的 WebSocket 客户端。这需要额外的 `tokio::sync::broadcast` 通道和日志层集成。
    2.  **Direct Chat:**
        *   添加 `POST /admin/chat` API。
        *   这个 Handler 函数会接收聊天请求（包含消息内容和想使用的 Key 类型）。
        *   它会执行与 `handle_proxy_request` 类似的流程：向 `KeyManager` 请求指定类型的 Key，构造 `DispatchRequest`（但路径/查询参数可能需要根据聊天内容生成），发送给 `Dispatcher`，等待响应，然后将 LLM 的响应返回给前端。
*   **如何测试:**
    *   **日志:** 测试下载 API 或连接 WebSocket 并观察实时日志流。
    *   **Direct Chat:** 使用 `curl` 或 Postman 模拟前端发送聊天请求，验证是否能成功通过代理与 LLM 交互并获得响应。

---

**通用建议:**

*   **持续集成 (CI):** 尽早设置 CI (如 GitHub Actions) 来运行 `cargo check`, `cargo fmt`, `cargo clippy`, 和 `cargo test`。
*   **代码审查:** 定期进行代码审查，确保代码质量、一致性和对设计的遵循。
*   **文档:** 随着代码的编写，同步更新 Rustdoc 文档。
*   **迭代:** 每个单元完成后，进行集成测试，确保各部分能协同工作。发现问题时及时调整计划。

这个计划提供了一个从基础到复杂的实现路径，并将任务分配给了两位开发者。请根据实际进展和遇到的挑战灵活调整。祝开发顺利！