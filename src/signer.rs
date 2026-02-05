use ed25519_dalek::{Signer, Verifier, SigningKey, VerifyingKey, Signature};
use rand::rngs::OsRng; // 用于生成私钥的安全随机数生成器
use crate::evidence::Evidence;

/// 签名器模块
/// 负责对 Evidence 进行司法级确证
pub struct EvidenceSigner {
    keypair: SigningKey,
}

impl EvidenceSigner {
    /// 创建一个新的签名器（在实际生产中，私钥应从安全存储/HSM加载，而不是每次生成）
    pub fn new() -> Self {
        // Ed25519 基于 Twisted Edwards Curve (Curve25519)
        // 安全性依赖于椭圆曲线离散对数问题 (ECDLP) 的难解性
        let keypair = SigningKey::generate(&mut OsRng);
        Self { keypair }
    }

    /// 获取公钥（用于分发给审计方进行验签）
    pub fn public_key(&self) -> VerifyingKey {
        self.keypair.verifying_key()
    }

    /// 对证据包进行数字签名
    /// 返回 64 字节的 Ed25519 签名
    pub fn sign(&self, evidence: &Evidence) -> anyhow::Result<Signature> {
        // 1. 序列化：将结构体转为确定性的字节序
        // 注意：serde_json 默认不保证字段顺序的确定性（Canonical Serialization）。
        // 在严谨的区块链应用中，通常使用类似 POST (Protobuf) 或 RLP 的定序编码。
        // 这里为了演示方便，假设 JSON 输出是稳定的。
        let payload = serde_json::to_vec(evidence)?;

        // 2. 签名：使用私钥对消息摘要进行签名
        // Ed25519 内部先做 SHA-512 哈希，再进行椭圆曲线标量乘法
        let signature = self.keypair.sign(&payload);
        
        Ok(signature)
    }

    /// 静态验证函数（给外部验证者使用）
    pub fn verify(verification_key: &VerifyingKey, evidence: &Evidence, signature: &Signature) -> anyhow::Result<bool> {
        let payload = serde_json::to_vec(evidence)?;
        // verify 内部会重新计算哈希并验证点是否在曲线上
        match verification_key.verify(&payload, signature) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
