# 原镜 (Yuanjing) Core v0.2.5 - API 接口文档

本文档描述了原镜后端服务的 HTTP 接口规范。

**Base URL**: `http://localhost:3000` (默认)

---

## 1. 模型治理 (Governance)

### 注册模型白名单 (Register Model)
将通过社区或管理员审核的“合法Prompt Pool哈希”注册到系统白名单中。只有白名单中的模型生成的证据才会被允许上链。

- **Endpoint**: `POST /model/register`
- **Content-Type**: `application/json`

#### 请求参数
| 字段 | 类型 | 必选 | 描述 | 示例 |
| :--- | :--- | :--- | :--- | :--- |
| `hash` | String | 是 | Prompt Pool 文件的 Blake3 哈希 (Hex) | `0fe57e48...` |
| `description` | String | 是 | 模型版本说明 | `SAPT-v2.0-Production` |

#### 响应示例 (200 OK)
```json
{
  "status": "Registered"
}
```

---

## 2. 存证服务 (Provability)

### 提交证据上链 (Prove)
接收前端/AI传来的原始图片路径及判决结果，在后端计算物理指纹(SHA256)和视觉指纹(pHash)，结合身份私钥进行签名，最后存入 Merkle Mountain Range。

- **Endpoint**: `POST /prove`
- **Content-Type**: `application/json`

#### 请求参数
| 字段 | 类型 | 必选 | 描述 | 示例 |
| :--- | :--- | :--- | :--- | :--- |
| `image_path` | String | 是 | 服务器可访问的图片绝对/相对路径 | `data/samples/news.jpg` |
| `verdict` | Bool | 是 | AI 判定结果 (true=真/false=假) | `false` |
| `confidence` | Float | 是 | AI 置信度 | `0.99` |
| `source` | String | 否 | 来源标识 | `web-client` |

> **注意**: 
> 1. 为了演示方便，目前 `prompt_pool_hash` 和 `activated_prompts` 在后端是从请求上下文或 Mock 数据中注入的，但在真实逻辑中，它们将被校验是否匹配白名单。
> 2. 如果关联的 Prompt Pool Hash 未注册，将返回 `400 Bad Request`。

#### 响应示例 (200 OK)
```json
{
  "root_hash": "a1b2c3d4...", 
  "leaf_pos": 15,
  "signature": "e4f5...",
  "evidence_dump": {
      "image_phash": "...",
      "image_sha256": "...",
      "verdict": false,
      "confidence": "0.99",
      "activated_prompts": [1, 5, 99],
      "prompt_pool_hash": "...",
      "external_knowledge_hash": "...",
      "timestamp": 1678888888
  }
}
```

---

## 3. 审计服务 (Audit)

### 获取存在性证明 (Audit Proof)
获取特定位置证据的 Merkle Proof，用于轻量级客户端验证该证据是否确实存在于当前的 Merkle Root 中。

- **Endpoint**: `GET /audit/{pos}`
- **参数**: `pos` (Uint64) - 证据在 MMR 中的叶子位置索引 (从 /prove 回执中获得)

#### 响应示例 (200 OK)
```json
{
  "proof_valid": true,
  "leaf_pos": 15,
  "proof_hex": [
      "hash_sibling_1...",
      "hash_sibling_2..."
  ]
}
```
> **注意**: `proof_valid` 仅供参考，真正的验证应在客户端使用 `proof_hex` 和当前的 `root_hash` 进行重算。

