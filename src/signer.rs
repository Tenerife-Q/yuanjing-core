use std::fs;
use std::path::Path;
use ed25519_dalek::{Signer, Verifier, SigningKey, VerifyingKey, Signature};
use rand::rngs::OsRng; 
use crate::evidence::Evidence;

/// 模块：签名器 (Signer)
/// 
/// **职责**: 负责“身份确权”。
/// 整个系统中最核心的安全组件，如同公证处的电子印章。
/// 它利用非对称加密算法（Ed25519），对证据包进行数字签名，确保数据不可篡改且来源可信。
/// 
/// **核心原理 (数学层面)**:
/// 基于 **ECDLP (Elliptic Curve Discrete Logarithm Problem)** 椭圆曲线离散对数难题。
/// - 给定私钥 $k$ 和基点 $G$，很容易算出公钥 $P = k \times G$。
/// - 但给定公钥 $P$ 和基点 $G$，反推私钥 $k$ 在计算上是不可行的（需要耗费全宇宙能量级别的算力）。
pub struct EvidenceSigner {
    /// 签名私钥 (Private Key)
    /// 
    /// **系统最高机密**。
    /// 语法细节: `SigningKey` 是实现了 Rust RAII 模式的类型，通常包含 Drop trait 用于在销毁时自动擦除内存中的密钥信息，防止冷启动攻击。
    /// 
    /// **[⚠️ 风险预警]**: 
    /// 本项目目前将密钥存储在内存结构体中。
    /// 一旦服务器被攻破并 Dump 内存，私钥即泄露。
    keypair: SigningKey,
}

impl EvidenceSigner {
    /// 从文件加载密钥，如果不存在则自动生成 (Load or Generate)
    /// 
    /// **工程改进 (Task C)**:
    /// 解决了之前“重启即丢失身份”的问题。
    /// 系统启动时会检查指定路径是否存在私钥文件：
    /// - **存在**: 读取文件恢复身份（模拟从 KeyStore 加载）。
    /// - **不存在**: 生成新密钥并保存到磁盘（模拟系统首次初始化）。
    pub fn load_or_generate<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();

        if path.exists() {
            println!("🔑 检测到现有身份文件，正在加载: '{}'", path.display());
            let bytes = fs::read(path)?;
            
            // 校验密钥长度 (Ed25519 Seed 为 32 字节)
            if bytes.len() != 32 {
                return Err(anyhow::anyhow!("关键错误: 身份文件损坏，长度不匹配"));
            }

            // 转换 slice 到 array
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            
            let keypair = SigningKey::from_bytes(&arr);
            Ok(Self { keypair })
        } else {
            println!("✨ 未检测到身份文件，正在初始化新身份: '{}'", path.display());
            let keypair = SigningKey::generate(&mut OsRng);
            
            // 将私钥 Seed (32 bytes) 写入磁盘
            // 注意：生产环境中，这个文件权限应设为 600 (只有拥有者可读)
            fs::write(path, keypair.to_bytes())?;
            
            Ok(Self { keypair })
        }
    }

    /// 导出公钥 (Public Key)
    ///
    /// **作用**: 自证清白。可以将此公钥公开在区块链上或 API 文档中。
    /// 任何人拿到这个公钥，就能验证“这确实是原镜系统签发的证据”。
    pub fn public_key(&self) -> VerifyingKey {
        self.keypair.verifying_key()
    }

    /// 核心功能：证据签名 (Digital Signature)
    ///
    /// **输入**: 原始证据结构体 `Evidence`
    /// **输出**: 64字节的签名数据 (R || s)
    ///
    /// **数学原理解析**:
    /// 签名过程 (Sign) 本质上是在构建一个零知识证明：
    /// 1. **生成随机数 r**: 基于私钥和消息生成确定性随机数。
    /// 2. **计算承诺 R**: $ R = r \times G $
    /// 3. **计算挑战 S**: $ S = r + \text{Hash}(R, P, M) \times k $
    ///    (其中 $k$ 为私钥, $P$ 为公钥, $M$ 为消息)
    /// 最终签名就是 $(R, S)$ 对。
    ///
    /// **[⚠️ 极度危险的坑 - 序列化确定性]**: 
    /// 代码中使用了 `serde_json::to_vec`。
    /// - **问题**: JSON 标准是“无序”的。`{"a":1, "b":2}` 和 `{"b":2, "a":1}` 在逻辑上相等，但在**字节流**上完全不同。
    /// - **后果**: 哈希函数对哪怕 1 个 bit 的变化都极其敏感（雪崩效应）。如果序列化结果哪怕变了一个字节顺序，生成的哈希就会全变，导致验证失败。
    /// - **解决方案 (Prod)**: 必须使用 **Canonical Serialization (规范化序列化)**，如:
    ///   - **BCS** (Binary Canonical Serialization - Libra/Aptos利用)
    ///   - **RLP** (Recursive Length Prefix - Ethereum利用)
    ///   - **Protobuf** (Deterministic Mode)
    pub fn sign(&self, evidence: &Evidence) -> anyhow::Result<Signature> {
        let payload = serde_json::to_vec(evidence)?;

        // Ed25519 签名算法 (EdDSA) 本质流程:
        // 1. Hash = SHA512(payload)  -> (压缩信息)
        // 2. r = Hash(Hash || PrivateKey) -> (引入随机性)
        // 3. R = r * G               -> (临时公钥点)
        // 4. S = r + Hash(R, Public, msg) * PrivateKey -> (标量混淆)
        // 5. Signature = (R, S)
        let signature = self.keypair.sign(&payload);
        
        Ok(signature)
    }

    /// 静态验证函数 (Verify Signature)
    ///
    /// **作用**: “没有任何人需要相信任何人”。
    /// 这是一个纯数学过程。不管你是法官、律师还是黑客，只要拿着公钥、证据和签名，算出来的结果都是一样的。
    ///
    /// **验证原理**:
    /// 验证方程: $ S \times G \overset{?}{=} R + \text{Hash}(R, P, M) \times P $
    /// 推导证明:
    /// $$ \text{Right} = R + h \times P = (r \times G) + h \times (k \times G) = (r + h \times k) \times G = S \times G $$
    /// 只要等式成立，就能证明 $S$ 确实是由持有私钥 $k$ 的人计算出的。
    pub fn verify(verification_key: &VerifyingKey, evidence: &Evidence, signature: &Signature) -> anyhow::Result<bool> {
        let payload = serde_json::to_vec(evidence)?;
        
        // 椭圆曲线验证公式:
        // 验证点 $S \times G$ 是否等于 $R + Hash(...) \times Pub$
        // 如果等式成立，说明这个签名只能是持有私钥的人生成的。
        match verification_key.verify(&payload, signature) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
