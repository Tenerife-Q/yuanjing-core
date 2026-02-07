mod evidence;
mod fingerprint;
mod signer;
mod mmr_store;

use mmr_store::EvidenceStore;
use signer::EvidenceSigner;
use std::path::Path;
use std::fs;
use chrono::Utc;
use serde::Deserialize;

#[derive(Deserialize)]
struct MockAiResponse {
    sapt_score: f32,
    is_forged: bool,
    activated_prompts: Vec<u32>,
    prompt_pool_hash: String,
    external_knowledge_hash: String,
}

fn main() -> anyhow::Result<()> {
    println!("🛡️ [原镜] 路由级可信确证模块已就绪...");

    let img_path = Path::new("data/samples/original.jpg");
    if !img_path.exists() {
        println!("⚠️ 请在data/samples目录下放入original.jpg进行测试");
        return Ok(());
    }

    let (sha, phash) = fingerprint::generate_fingerprints(img_path)?;

    // 1. 读取 Mock AI 推理结果
    let mock_json_path = "data/mock/ai_response_valid.json";
    let mock_json = fs::read_to_string(mock_json_path)
        .map_err(|_| anyhow::anyhow!("❌ 找不到 Mock 数据: {}", mock_json_path))?;
    let ai_resp: MockAiResponse = serde_json::from_str(&mock_json)?;

    println!("🤖 AI 引擎响应: Forged={}, Confidence={:.2}", ai_resp.is_forged, ai_resp.sapt_score);

    // 2. 组装完整证据链
    let mock_evidence = evidence::Evidence {
        image_phash: phash,
        image_sha256: sha,
        verdict: !ai_resp.is_forged, // true=真图, false=伪造
        confidence: ai_resp.sapt_score,
        activated_prompts: ai_resp.activated_prompts,
        prompt_pool_hash: ai_resp.prompt_pool_hash,
        external_knowledge_hash: ai_resp.external_knowledge_hash,
        timestamp: Utc::now().timestamp(),
    };

    println!("📄 生成可审计证据包：\n{:#?}", mock_evidence);

    // Task A & C: 数字签名 + 身份持久化
    println!("✍️ 正在加载司法级身份并签名...");
    
    // 使用 load_or_generate 替代 new
    // 第一次运行会生成 yuanjing.key，之后运行会直接读取
    let signer = EvidenceSigner::load_or_generate("yuanjing.key")?;
    
    // 打印一下当前的公钥（身份ID），方便演示时证明"身份没变"
    let pub_key_bytes = signer.public_key().to_bytes();
    println!("🆔 当前法证中心身份ID (Public Key): {}", hex::encode(pub_key_bytes));

    let signature = signer.sign(&mock_evidence)?;
    
    // 签名结果展示
    println!("🔐 签名生成成功 (Bytes): {:?}", signature.to_bytes());

    // 模拟验签
    let pub_key = signer.public_key();
    let is_valid = EvidenceSigner::verify(&pub_key, &mock_evidence, &signature)?;

    if is_valid {
        println!("✅ 验签通过：证据完整且来源可信。");
    } else {
        println!("❌ 验签失败！");
    }

    // Task B: MMR 存证
    println!("📚 正在进行 MMR 存证归档...");
    let mut store = EvidenceStore::new();
    let (root_hash, leaf_pos) = store.append(&mock_evidence)?;

    println!("🌲 MMR Root Hash: {}", hex::encode(root_hash));
    println!("🍃 证据插入位置 (Leaf Pos): {}", leaf_pos);

    // 获取并打印 Proof
    let proof = store.get_proof(vec![leaf_pos])?;
    println!("🧾 包含证明 (Merkle Proof) 已生成，包含 {} 个节点。", proof.proof_items().len());

    Ok(())
}