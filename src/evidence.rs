use serde::{Deserialize, Serialize}; // 引入序列化库，让结构体能转成JSON/二进制传输

// Derive 宏：自动为结构体生成 Debug打印、序列化、反序列化、克隆(Clone) 的能力
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Evidence {
    // === 第一层：物理指纹 (Identity) ===
    
    // 视觉感知哈希 (pHash)
    // 作用：解决“内容同一性”。
    // 细节：比如图片经过微信压缩、裁剪，SHA256 会全变，但 pHash 依然相似。
    // 类型：String (存储为 Base64 编码，因为这种格式短小且适合数据库存储)
    pub image_phash: String, 
    
    // 密码学哈希 (SHA256)
    // 作用：解决“原始完整性”。
    // 细节：哪怕图片元数据里改了一个字节，这个值都会雪崩式变化。这是用来防止“调包”的。
    // 类型：String (存储为 64字符的 Hex 字符串)
    pub image_sha256: String,

    // === 第二层：推理指纹 (Reasoning) ===
    
    // 判决结果
    // 作用：AI 的最终结论。
    // 类型：bool (true=真/false=假，或者在你的场景里 false=伪造)
    pub verdict: bool,
    
    // 置信度
    // 作用：AI 有多大把握。
    // 类型：f32 (0.0 到 1.0 的浮点数)。如果 confidence 低于某个阈值，法证中心可能拒绝存证。
    pub confidence: f32,
    
    // 激活的提示词索引 (SAPT - 稀疏激活)
    // 作用：这是“白盒审计”的关键！
    // 解释：你的模型里有成百上千个 Prompt 组件（专家）。
    //       vec![3, 7, 12] 表示：这张图触发了 #3(医疗专家), #7(水印检测), #12(也是某个特征) 组件。
    //       如果两张图虽然都是“伪造”，但激活路径完全不同，说明造假手法不同。
    pub activated_prompts: Vec<u32>,
    
    // Prompt 池哈希
    // 作用：防投毒 / 版本控制。
    // 解释：如果黑客悄悄修改了模型里的 Prompt，这个哈希就会变。确保本次推理是基于“经过审核的那版模型”做出的。
    pub prompt_pool_hash: String,

    // === 第三层：事实指纹 (Fact Check) ===
    
    // 外部知识锚点
    // 作用：ERIC-FND 模块会去查 Wiki 或新闻。
    // 解释：AI 判定这不是谣言，是因为引用了“新华社2月5日的报道”。
    //       这里存下那篇报道文本的哈希。防止未来网页被删或被改，导致死无对证。
    pub external_knowledge_hash: String,

    // === 第四层：元数据 (Metadata) ===
    
    // 时间戳
    // 作用：数字确权的核心，证明“在该时间点，该状态已存在”。
    // 类型：i64 (Unix 时间戳，秒级或毫秒级)
    pub timestamp: i64,
}