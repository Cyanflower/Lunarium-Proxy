# Gemini 反向代理 (异步事件驱动 & 独立客户端)

本项目是一个基于 `tokio` 构建的异步反向代理，旨在统一管理和调度对 Google Gemini API (通过 API Key) 和 Google AI Studio (通过模拟浏览器 Cookie) 的访问。核心设计采用异步事件驱动模型，通过 `tokio` 通道实现组件解耦，并为每次代理请求创建独立的 HTTP 客户端以实现高效并发和隔离。

**核心设计理念:**

1.  **统一凭证管理 (`KeyManager`)**: 一个中心的、异步的 `KeyManager` 任务负责管理所有可用凭证（Gemini API Keys 和 AI Studio Cookies），维护其状态（可用、已分发、耗尽、无效），并通过通道处理凭证的分发与回收。它根据请求指定所需的凭证 *类型* (Gemini 或 AI Studio) 来分发。
2.  **消息驱动分发 (`Dispatcher`)**: API 请求处理层将解析后的任务信息（包括目标路径、查询参数、请求体、获取到的目标凭证、以及用于回传响应的通道）打包成消息，发送给中央 `Dispatcher` 任务进行路由。
3.  **类型化处理器 (`Handler`)**: `Dispatcher` 根据消息中凭证的类型，将任务（包含凭证和请求数据的所有权）转发给对应的异步处理器函数（`process_gemini_request` 或 `process_aistudio_request`），并通过 `tokio::spawn` 为每个请求启动一个短暂的处理任务。
4.  **独立客户端与提示词处理**: 每个 Handler 处理函数在处理单个请求时：
    *   **调用 `prompt.rs`**: 根据配置对传入请求体中的提示词部分进行可能的重组或修改。
    *   **创建全新的 `reqwest::Client` 实例**: 这个 Client 仅用于本次与后端服务（Gemini 或 AI Studio）的交互。
    *   **执行请求**: 使用修改后的请求体和获取到的凭证向目标服务发起请求。
    *   请求完成后，Client 实例即被销毁。这确保了请求间的完全隔离，并允许高并发处理。
5.  **状态反馈与响应转发**: Handler 完成其单次任务后：
    *   将使用的凭证及其结果状态（成功、失败原因）通过通道发回给 `KeyManager` 进行状态更新和回收。
    *   将从后端服务获取的 HTTP 响应数据（或产生的错误），包括流式响应体，通过 `oneshot` 通道直接发送回原始的 API 请求处理函数 (`proxy.rs`)。

**文件结构:**

```
gemini-proxy/
├── Cargo.toml             # 项目依赖配置文件
├── config.toml.example    # 示例配置文件 (包含 Gemini Base URL, 管理 Key 等)
├── src/
│   ├── main.rs            # 程序入口: 初始化, 启动核心任务, 启动服务器
│   ├── cli.rs             # Clap 命令行参数定义
│   ├── config.rs          # 配置数据结构 (serde), Key enum (Gemini/AIStudio), 加载/保存
│   ├── state.rs           # AppState 定义 (共享配置 Arc, 通道 Senders)
│   ├── error.rs           # 自定义错误类型 (thiserror), HandlerError
│   ├── logger.rs          # 日志系统初始化 (tracing)
│   ├── tls.rs             # TLS 配置辅助 (rustls)
│   ├── key_manager.rs     # 核心: Async Key/Cookie 池管理器任务
│   ├── dispatcher.rs      # 核心: Async 请求消息分发器任务
│   ├── handlers/          # Async 函数: 处理与后端服务的实际交互
│   │   ├── mod.rs         # 导出 Handler 处理函数
│   │   ├── gemini.rs      # process_gemini_request: 处理 Gemini API 请求 (独立 Client)
│   │   ├── aistudio.rs    # process_aistudio_request: 处理 AI Studio 请求 (独立 Client)
│   │   └── prompt.rs      # 提供提示词修改/重组的工具函数
│   ├── proxy.rs           # Axum Handler: handle_proxy_request - 代理 API Endpoint 处理逻辑
│   ├── router.rs          # Axum Router 构建, 定义 API 路由和认证逻辑层
│   └── models.rs          # 共享数据结构 (Channel 消息类型, Key 状态 Reason)
├── web/                   # 前端静态文件 (独立部署)
└── certificates/          # 用户提供的域名证书和私钥 (可选)
```

**说明:**

*   **`config.rs`**: 除了 Keys/Cookies，还应包含目标服务的 Base URL（例如 Gemini API 的 `https://generativelanguage.googleapis.com`）、代理自身的管理/认证 Key，以及可能的提示词修改配置。
*   **`router.rs`**: 构建 Axum Router，定义所有 API 路由。**代理请求的认证逻辑**（例如，检查查询参数 `?key=proxy_pwd`）应在此处作为 Axum Layer 或 Middleware 实现，应用于需要认证的路由（如 `/api/v1beta/*`）。认证失败则直接返回 401/403。
*   **`proxy.rs`**: 包含 `handle_proxy_request` 函数，这是经过认证后的代理请求入口点。它负责解析请求、向 `KeyManager` 请求凭证、将任务发送给 `Dispatcher`、并等待 `Handler` 通过 `oneshot` 通道返回的最终 `reqwest::Response` 或错误，然后将其转换为 `axum::Response` 返回给客户端（支持流式传输）。
*   **`handlers/gemini.rs` / `aistudio.rs`**: Handler 函数接收凭证和请求数据的**所有权**。它们会：
    1.  调用 `handlers::prompt::process_prompt(...)` (如果需要) 来修改请求体。
    2.  创建一个临时的 `reqwest::Client`。
    3.  根据凭证类型（API Key 或 Cookie）和修改后的请求体，构造并发送请求到 Google 服务。例如，Gemini Handler 会将 API Key 添加为 URL 查询参数 `?key=<GEMINI_API_KEY>`。
    4.  将收到的 `reqwest::Response` (成功时) 或 `HandlerError` (失败时) 通过 `oneshot` 通道发回给 `proxy.rs`。
    5.  将使用过的凭证及状态发回给 `KeyManager`。
*   **`handlers/prompt.rs`**: 包含纯函数，接收请求体数据和配置，返回修改后的请求体数据。不执行 I/O 操作。
*   **`models.rs`**: 定义如 `DispatchRequest` (包含原始路径、查询、体、目标 Key、响应 Sender) 和 `HandlerResponse` (`Result<reqwest::Response, HandlerError>`) 等通道间传递的消息结构。

**详细业务流 (以处理 Gemini `generateContent` 请求为例):**

1.  **启动 (`main.rs`):**
    *   解析命令行参数, 加载配置 (`Config`), 初始化日志。
    *   创建通道: `key_request_tx/rx`, `key_return_tx/rx`, `dispatch_tx/rx`。
    *   启动核心后台任务: `key_manager::run_key_manager`, `dispatcher::run_dispatcher`。
    *   创建 `AppState` (含 `Arc<Config>`, Senders)。
    *   配置 TLS (可选)。
    *   构建 Axum Router (`router.rs`):
        *   定义 `/api/v1beta/*` 路由指向 `proxy::handle_proxy_request`。
        *   **应用认证层**: 检查请求 URI 查询参数中的 `key` 是否匹配 `app_state.config.management_key`。
        *   定义其他管理 API 路由（见静态前端部分）。
    *   启动 Axum 服务器。

2.  **客户端请求:**
    *   客户端发送 POST 请求到 `http://127.0.0.1:3200/api/v1beta/models/gemini-pro:generateContent?key=proxy_pwd`。

3.  **路由与认证 (`router.rs`):**
    *   Axum 接收请求。
    *   认证层/中间件提取 `?key=proxy_pwd`，验证通过。请求被传递给 `proxy::handle_proxy_request`。

4.  **代理入口处理 (`proxy.rs::handle_proxy_request`):**
    *   获取 `AppState`。
    *   解析请求: 提取原始路径、查询参数 (过滤掉认证 `key`)、请求体。
    *   判断所需凭证类型: 根据路径或配置确定为 `KeyType::Gemini`。
    *   **请求凭证:** 向 `KeyManager` 发送 `(KeyType::Gemini, key_response_tx)`，并等待 `key_response_rx` 返回 `Ok(Key::Gemini(acquired_api_key))`。
    *   **准备分发:** 创建 `oneshot::channel` (`http_response_tx`, `http_response_rx`) 用于接收 Handler 的最终结果。
    *   创建 `DispatchRequest` 消息，包含获取到的 `acquired_api_key` (凭证)、原始请求信息和 `http_response_tx`。
    *   **发送到 Dispatcher:** `app_state.dispatch_tx.send(dispatch_request).await?` (将所有权转移给 Dispatcher)。
    *   **等待 Handler 响应:** `let handler_result = http_response_rx.await?` 接收 `Result<reqwest::Response, HandlerError>`。
    *   **构建 Axum 响应:**
        *   如果 `Ok(response)`，将其状态码、头信息和 Body (可能是流) 转换为 `axum::Response` 并返回。
        *   如果 `Err(handler_error)`，记录错误并返回相应的 HTTP 错误响应。

5.  **KeyManager (`key_manager.rs`):**
    *   接收凭证请求，分发可用凭证，通过 `oneshot` 通道返回。
    *   监听 `key_return_rx`，接收 Handler 返回的 `(Key, Option<Reason>)`，更新凭证状态。

6.  **Dispatcher (`dispatcher.rs::run_dispatcher`):**
    *   接收 `DispatchRequest` 消息。
    *   根据 `msg.target_key` 类型，`tokio::spawn` 对应的 Handler 任务 (例如 `handlers::gemini::process_gemini_request`)，并将消息中的凭证、请求数据、`response_sender` 和 `key_return_tx` **移动**给 Handler。

7.  **Gemini Handler 处理 (`handlers::gemini.rs::process_gemini_request`):**
    *   `async fn process_gemini_request(api_key_info: GeminiApiKeyInfo, path: String, query: QueryParamsMap, mut body: JsonValue, response_sender: OneshotSender<Result<reqwest::Response, HandlerError>>, key_return_tx: ..., config: Arc<Config>)`
    *   **调用提示词处理:** `body = handlers::prompt::process_gemini_prompt(body, &config.prompt_config)?;`
    *   **创建独立 Client:** `let client = reqwest::Client::new();`
    *   **构建目标 URL 和查询参数:** 组合 `config.gemini_base_url`, `path`, `query`，并将 `api_key_info.key_value` 作为 `?key=` 参数添加。处理流式请求参数 (如 `alt=sse`)。
    *   **执行请求:** `let result = client.post(target_url).json(&body).send().await;`
    *   **判断凭证状态:** 根据 `result` 确定 `reason: Option<Reason>`。
    *   **返回凭证给 KeyManager:** `let _ = key_return_tx.send((Key::Gemini(api_key_info), reason)).await;`
    *   **发送结果给代理入口:**
        ```rust
        match result {
            Ok(response) => {
                // 将 reqwest::Response (包含潜在的流式 body) 发回
                let _ = response_sender.send(Ok(response));
            }
            Err(e) => {
                let _ = response_sender.send(Err(HandlerError::RequestFailed(e.to_string())));
            }
        }
        ```
    *   函数结束，`client` 被销毁。

8.  **响应流回传:** `proxy.rs` 接收到 `Ok(response)` 后，将其转换为 `axum::Response`，并将 Body (包括可能的 `BytesStream`) 流式传输回原始客户端。

**静态前端设计:**

本项目后端暴露 HTTP API 接口，host一个前端应作为静态 Web管理页面（使用Svelte 构建）。

**前端功能需求:**

*   **模式切换**: 提供 UI 界面选择代理的目标服务类型（例如，优先使用 Gemini API Key，或优先使用 AI Studio Cookie）。
*   **配置管理**: 查看和修改后端服务的配置项（如 Gemini Base URL、代理端口等，敏感信息如管理 Key 不应直接暴露）。
*   **凭证管理**:
    *   **Gemini API Keys**: 添加、列出、启用/禁用、删除 API Key。
    *   **AI Studio Cookies**: 添加、列出、启用/禁用、更新、删除 Cookie。
    *   **管理/路由 Keys**: 添加、列出、启用/禁用、删除用于访问代理本身的 Key (`?key=...`)。
*   **Direct Chat 测试**: 提供一个简单的聊天界面，可以直接通过代理向后端 LLM 发送请求并显示响应，用于测试配置和凭证的有效性。
*   **日志查看/导出**: 提供接口查看后端实时日志或导出历史日志文件。
*   **状态监控**: 显示当前 `KeyManager` 中各类凭证的状态统计。