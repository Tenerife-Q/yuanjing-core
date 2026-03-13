#  原镜 (Yuanjing) Core - 深度项目交接文档

## 1. 项目愿景与定位
**项目名称**: Yuanjing-Core  
**核心目标**: 为 Python (Deepfake 检测) 模块提供“司法级”的存证能力。  

AI 检测结果（如“此图为伪造”）本身只是易篡改的文本记录。本项目通过密码学签名和 MMR (Merkle Mountain Range) 结构，将检测结果固化为不可抵赖、不可篡改、可第三方审计的电子证据。

---

## 2. 核心存证工作流 (The Evidence Pipeline)
当一个 `POST /prove` 请求到达时，系统内部的数据流转如下：

1. 接入层 (`api.rs`)
   - 接收 JSON 请求：`image_path`, `verdict`, `confidence`, `prompt_pool_hash`
   - 使用 `spawn_blocking` 将图片 I/O 与指纹计算移出异步主线程，避免阻塞。
2. 特征层 (`fingerprint.rs`)
   - 读取图片文件，计算 SHA256 与 pHash。
3. 数据组装
   - 构建 `Evidence` 结构体（内部存储中 `confidence` 以字符串形式落盘）。
4. 鉴权层 (`signer.rs`)
   - 使用 Ed25519 私钥对 Evidence 进行签名（BCS 序列化保证字节确定性）。
5. 存储层 (`mmr_store.rs`)
   - Append 到 MMR；sled 落盘持久化。
6. 响应
   - 返回 `root_hash`、`leaf_pos`、`signature`。

---

## 3. 本地联调与验证（最重要）
### 3.1 启动服务
```bash
cargo run
