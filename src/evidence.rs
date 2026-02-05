use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Evidence {
    // 1. 物理层指纹（解决“这张图是不是那张图”）
    pub image_phash: String,
    pub image_sha256: String,

    // 2. 推理路径指纹（解决“AI凭什么这么判”）
    pub verdict: bool,
    pub confidence: f32,
    pub activated_prompts: Vec<u32>, // 记录SAPT激活的Prompt组件索引（如医疗、娱乐提示）
    pub prompt_pool_hash: String,    // 记录当前Prompt Pool矩阵的哈希，防投毒

    // 3. 外部知识锚点（解决“引用信息是否真实”）
    pub external_knowledge_hash: String, // 记录ERIC-FND引用的Wiki文本快照哈希

    // 4. 元数据
    pub timestamp: i64,
}