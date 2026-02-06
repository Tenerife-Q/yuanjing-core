mod evidence;
mod fingerprint;
mod signer;
mod mmr_store;

use mmr_store::EvidenceStore;
use signer::EvidenceSigner;
use std::path::Path;
use chrono::Utc;

fn main() -> anyhow::Result<()> {
    println!("🛡️ [原镜] 路由级可信确证模块已就绪...");

    let img_path = Path::new("data/samples/original.jpg");
    if !img_path.exists() {
        println!("⚠️ 请在data/samples目录下放入original.jpg进行测试");
        return Ok(());
    }

    let (sha, phash) = fingerprint::generate_fingerprints(img_path)?;

    // 模拟一次来自王模型的推理输出
    let mock_evidence = evidence::Evidence {
        image_phash: phash,
        image_sha256: sha,
        verdict: false, // 判定为伪造
        confidence: 0.94,
        activated_prompts: vec![3, 7, 12], // 模拟激活了医疗(3)和谣言特征(12)提示
        prompt_pool_hash: "blake3_hash_of_prompt_matrix".to_string(),
        external_knowledge_hash: "hash_of_wiki_fact_check_text".to_string(),
        timestamp: Utc::now().timestamp(),
    };

    println!("📄 生成可审计证据包：\n{:#?}", mock_evidence);

    // Task A: 数字签名
    println!("✍️ 正在进行司法级数字签名 (Ed25519)...");
    let signer = EvidenceSigner::new();
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