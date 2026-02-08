# 🚀 原镜 (Yuanjing) Core v0.2 - 深度项目交接文档

## 1. 项目愿景与定位
**项目名称**: Yuanjing-Core
**核心目标**: 为 Python (Deepfake 检测) 模块提供“司法级”的存证能力。
**解决痛点**: 
AI 检测结果（如“此图为伪造”）本身只是易篡改的文本记录。本项目通过**密码学签名**和**MMR (Merkle Mountain Range) 结构**，将检测结果固化为不可抵赖、不可篡改、可第三方审计的电子证据。

## 2. 核心存证工作流 (The Evidence Pipeline)
当一个 `POST /prove` 请求到达时，系统内部的数据流转如下：

1.  **接入层 (api.rs)**: 
    - 接收 JSON 请求 (`image_path`, `verdict`, `confidence`).
    - **非阻塞优化**: 立即启动 `tokio::task::spawn_blocking` 线程池处理图片读取，防止阻塞 HTTP 主线程。
2.  **特征层 (fingerprint.rs)**:
    - 这里的 Worker 线程读取图片文件。
    - 计算 **SHA256** (保证文件字节级未变) 和 **pHash** (感知哈希，保证视觉内容一致性)。
3.  **数据组装**:
    - 构建 `Evidence` 结构体。注意：为了二进制安全，`confidence` 被存储为 String。
4.  **鉴权层 (signer.rs)**:
    - 使用加载的私钥 (`yuanjing.key`) 对 `Evidence` 进行签名。
    - **关键安全机制**: 签名时先将结构体通过 **BCS (Binary Canonical Serialization)** 序列化，确保字节流的绝对确定性。
5.  **存储层 (mmr_store.rs)**:
    - 将 `Evidence` 序列化后的 Hash 作为叶子节点 Append 到 Merkle Mountain Range 树中。
    - **持久化**: 写入 `sled` 嵌入式数据库，并立即 Flush 落盘。
6.  **响应**: 返回 `root_hash` (当前树顶)、`leaf_pos` (叶子位置) 和 `signature`。

## 3. 模块深度解析与价值体现

### A. `signer.rs` - 数字身份与签名
-   **功能**: 管理 Ed25519 密钥对；执行签名 (`sign`) 和验签 (`verify`)。
-   **解决问题**: **身份伪造与数据篡改**。
    -   *身份*: 通过私钥签名，证明该证据确由“原镜系统”生成，而非黑客伪造。
    -   *篡改*: 结合 BCS 序列化，数据哪怕变动 1 bit，签名验证都会失败。
-   **v0.2 升级**: 
    -   由 `serde_json` 切换为 `bcs`，彻底解决了 JSON 字段无序导致的签名校验不稳定问题。
    -   实现了 `load_or_generate`，保证服务重启后身份 ID (公钥) 不变。

### B. `mmr_store.rs` - 默克尔存证库
-   **功能**: 基于 Blake3 哈希算法维护 Merkle Mountain Range 结构；生成 Merkle Proof。
-   **解决问题**: **历史记录回溯与轻量级审计**。
    -   MMR 是一种 Append-only (只增不减) 结构，天然契合日志存证。
    -   审计员无需下载所有数据，只需获取一个轻量的 Proof (Log(N)大小) 即可验证某条证据是否存在。
-   **v0.2 升级**: 
    -   后端由 `MemStore` (内存) 迁移至 `SledStore` (磁盘 KV)。
    -   解决了“重启服务导致 Merkle Tree 归零、旧 Proof 失效”的严重 Bug。

### C. `fingerprint.rs` - 多维指纹提取
-   **功能**: 提取物理指纹 (SHA256) 和视觉指纹 (pHash)。
-   **解决问题**: **证据锚定**。
    -   将物理文件路径转化为数学特征，确保链上数据与链下实体文件的唯一对应关系。
-   **调用细节**: 被封装在 `api.rs` 的 `spawn_blocking` 中调用，避免 I/O 阻塞。

### D. `api.rs` & `main.rs` - 服务编排
-   **功能**: Axum Web 服务入口，状态管理 (`Arc<AppState>`)。
-   **解决问题**: **系统集成**。
    -   提供标准的 RESTful 接口，屏蔽底层 Rust 复杂性，方便 Python 侧调用。

## 4. 当前技术栈概览
-   **Web 框架**: Axum 0.7+
-   **异步运行时**: Tokio (fs, sync, task)
-   **数据库**: Sled (Embedded KV)
-   **序列化**: Serde (JSON 交互), BCS (内部签名/Hash)
-   **密码学**: Ed25519-dalek, Blake3, Sha2
-   **图像处理**: Img_hash, Image

## 5. 下一步迭代建议 (v0.3 Roadmap)
既然核心的“存、算、签”已经稳固，下一步应通过以下工作将项目推向“生产就绪”：

1.  **Python Client 规范 (Client SDK)**: 
    -   编写 Python 脚本，演示如何重现 Rust 端的 BCS 序列化过程，从而成功验证 Ed25519 签名。这是目前联调最大的风险点。
2.  **配置工程化**: 
    -   移除代码中的硬编码路径 (`data/db/mmr_db`, `yuanjing.key`)，改为读取配置文件或环境变量。
3.  **错误处理体系**: 
    -   定义全局错误 Enum，将 `anyhow::Result` 映射为具体的 HTTP 错误码 (如 400 vs 500)，而不是现在的统一 500。
4.  **Proof 浏览器**:
    -   (可选) 编写一个简单的 HTML 页面，输入 `leaf_pos` 可视化展示 Merkle Path。

## 6. 开发者指令
-   **启动服务**: `cargo run`
-   **测试脚本**:
```bash
curl -X POST http://localhost:3000/prove \
  -H "Content-Type: application/json" \
  -d '{"image_path": "data/samples/original.jpg", "verdict": true, "confidence": "0.99", "source": "final_test"}'
```
