# 原镜 (Yuanjing) Core v0.2.5 - API 接口文档

本文档描述了原镜后端服务的 HTTP 接口规范。

**Base URL**: `http://localhost:3000` (默认)

> 注意：访问 `/` 返回 404 属正常现象，本服务为纯 API 服务，请调用下方具体接口路径。

---

## 1. 模型治理 (Governance)

### 注册模型白名单 (Register Model)
将通过社区或管理员审核的“合法 Prompt Pool 哈希”注册到系统白名单中。只有白名单中的模型生成的证据才会被允许存证。

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
    "prompt_pool_hash": "mmfn_v1",
    "external_knowledge_hash": "...",
    "timestamp": 1678888888
  }
}
{
  "proof_valid": true,
  "leaf_pos": 15,
  "proof_hex": [
    "hash_sibling_1...",
    "hash_sibling_2..."
  ]
}


```text
