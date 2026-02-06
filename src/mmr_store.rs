use ckb_merkle_mountain_range::{MMR, Merge, util::MemStore};
use crate::evidence::Evidence;

/// 模块：MMR 存储后端 (MMR Store)
/// 
/// **职责**: 负责“档案管理”。
/// 如同法证中心的地下档案库，负责按时间顺序存储所有证据的指纹，并提供不可篡改的存在性证明。
/// 
/// **为什么要用 MMR (Merkle Mountain Range)**?
/// - **数据结构**: 是一系列“完美二叉树”的集合。想像一下远处连绵的山峰，每座山峰都是一棵树。
///   - 比如 11 个元素 (二进制 1011)，MMR 会维护 3 座山峰：高度4 + 高度2 + 高度1。
/// - **Bagging Peaks**: 为了得到唯一的 Root，MMR 最后会将所有山峰的顶点再进行一次哈希合并。
///   $$ \text{Root} = \text{Hash}(\text{Peak}_1, \text{Hash}(\text{Peak}_2, \text{Peak}_3)) $$
/// - **追加优先 (Append-only)**: 只要不断往右添加叶子，MMR 就会自动合并生成哈希，只会增加不会修改历史，完美契合区块链日志特性。
/// - **轻量级**: 计算 Root 只需要这几座山峰的山顶 (Peaks) 哈希，无需遍历几百万个数据。

/// 合并策略 (Merge Strategy)
/// 
/// **作用**: 自定义“树枝怎么长”。告诉 MMR 库，当两个子节点碰头时，如何计算出父节点。
/// **数学原理**: Merkle Compression (默克尔压缩)
/// $$ H_{parent} = Hash(H_{left} \ || \ H_{right}) $$
/// 这一步是所有安全性的基石。只要 Root Hash 没变，根据抗碰撞性 (Collision Resistance)，整棵树下的几亿个节点就绝对没变。
pub struct MergeBlake3;

impl Merge for MergeBlake3 {
    type Item = [u8; 32]; // 每一个节点（叶子或树枝）都固定是 32 字节 (256 bit)

    /// 合并函数
    /// 
    /// **[语法细节]**: `ckb_merkle_mountain_range::Result` 是库自定义的结果类型，用于处理合并时可能出现的内存错误。
    fn merge(lhs: &Self::Item, rhs: &Self::Item) -> ckb_merkle_mountain_range::Result<Self::Item> {
        let mut hasher = blake3::Hasher::new();
        
        // 按照行业惯例 (如 Bitcoin, CKB)，父节点哈希 = Hash(左孩子 || 右孩子)
        // Blake3: 新一代哈希霸主，利用 SIMD 指令集并行加速，比 SHA256 快很多倍。
        hasher.update(lhs);
        hasher.update(rhs);
        
        // `*` 解引用: 从 slice 拷贝出 [u8;32] 数组
        Ok(*hasher.finalize().as_bytes())
    }
}

/// 证据仓库 (Evidence Store)
/// 
/// **职责**: 维护“全知全能”的账本状态。
pub struct EvidenceStore {
    /// 存储引擎
    /// 
    /// **[⚠️ 生产风险 - 数据的持久化]**: 
    /// - **当前 (Dev)**: `MemStore` 纯内存实现。程序一关，法证中心的档案全被火烧光。
    /// - **生产 (Prod)**: 必须替换为 KV 数据库 (例如 RocksDB)。
    ///   通常的做法是实现一个 Struct 包装 RocksDB，并让它 `impl Store` trait。
    store: MemStore<[u8; 32]>,
    
    /// MMR 树大小
    /// 
    /// **[核心状态]**: 
    /// 这是一个极度重要的元数据。它不仅仅是 count，更是 MMR 算法进行位运算路由的坐标系。
    /// 如果弄丢了这个值，你对着一堆哈希数据将无从下手，不知道哪是山顶，哪是山脚。
    mmr_size: u64,
}

impl EvidenceStore {
    /// 初始化仓库
    pub fn new() -> Self {
        Self {
            store: MemStore::default(),
            mmr_size: 0,
        }
    }

    /// 核心功能：证据上链入库 (Append Evidence)
    /// 
    /// **输入**: 完整的证据包 `Evidence`
    /// **输出**: (全局根 Root, 存证收据 LeafPos)
    /// 
    /// **实现流程**:
    /// 1. **瘦身**: 你的 Evidence 有点大（包含 activated_prompts 数组），MMR 不存原始数据。
    /// 2. **指纹化**: 先算一次 Hash，把 Evidence 压缩成 32 字节的“叶子”。
    /// 3. **生长**: 把叶子 Push 进树里。如果有落单的右子树，会自动触发合并 (Merge) 直到稳定。
    /// 4. **封袋**: 收集所有山峰的山顶 (Bagging Peaks)，算出最终的总 Root。
    pub fn append(&mut self, evidence: &Evidence) -> anyhow::Result<([u8; 32], u64)> {
        // [⚠️ 依然存在的序列化隐患]: 和 Signer 模块一样，这里计算叶子哈希也依赖序列化稳定性。
        let payload = serde_json::to_vec(evidence)?;
        
        // 计算叶子哈希 (Leaf Hash)
        let leaf_hash = *blake3::hash(&payload).as_bytes();

        // 实例化 MMR 操作句柄
        // -------------------
        // **[语法细节 - 泛型与借用]**: 
        // `MMR::<...>` 这里显式指定了泛型，告诉编译器我们存的是 [u8;32]，用 MergeBlake3 策略。
        // `&self.store`: MMR 结构体本身很轻，不拥有数据，它只是借用(borrow)下面的 store 来读写。
        let mut mmr = MMR::<[u8; 32], MergeBlake3, _>::new(self.mmr_size, &self.store);
        
        // Push 操作
        // 这一步在内部进行了大量位运算，寻找插入点和合并不是 O(1) 而是 O(log n)。
        let pos = mmr.push(leaf_hash).map_err(|e| anyhow::anyhow!("MMR append error: {}", e))?;
        
        // 更新状态
        self.mmr_size = mmr.mmr_size();

        // 获取最新的 Root
        // 这个 Root 就是未来要写到区块链 Block Header 里的那个 32 字节。
        let root = mmr.get_root().map_err(|e| anyhow::anyhow!("MMR get_root error: {}", e))?;
        
        Ok((root, pos))
    }

    /// 核心功能：开具证明 (Generate Merkle Proof)
    /// 
    /// **场景**: 第三方审计员问：“第 1005 号证据真的在这个 Root 里吗？我不信，除非你给我证据。”
    /// **输出**: Merkle Proof
    /// 
    /// **图解 Proof 结构**:
    /// 假设要证明 $L_3$ 存在 (Target)，审计员需要以下数据回放计算过程：
    /// 1. $L_4$ (兄弟, 用于计算 $P_{34} = Hash(L_3, L_4)$)
    /// 2. $H_{1..2}$ (叔叔, 用于计算 $H_{1..4} = Hash(H_{1..2}, P_{34})$)
    /// 3. 其他山峰的 Peaks (用于计算最终 Root)
    /// 
    /// ```text
    ///            Peak (H_1..8)
    ///           /            \
    ///      H_1..4            H_5..8
    ///     /      \
    ///  H_1..2    P_34 (Calculated)
    ///            /    \
    ///         L_3      L_4 (Sibling)
    ///       (Target)
    /// ```
    pub fn get_proof(&self, pos_list: Vec<u64>) -> anyhow::Result<ckb_merkle_mountain_range::MerkleProof<[u8; 32], MergeBlake3>> {
        let mmr = MMR::<[u8; 32], MergeBlake3, _>::new(self.mmr_size, &self.store);
        mmr.gen_proof(pos_list).map_err(|e| anyhow::anyhow!("MMR gen_proof error: {}", e))
    }
}
