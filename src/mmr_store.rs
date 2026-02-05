use ckb_merkle_mountain_range::{MMR, Merge, util::MemStore};
use crate::evidence::Evidence;

/// 定义合并策略 (Merge Strategy)
/// MMR 是二叉树结构，父节点哈希 = Merge(左子节点, 右子节点)
pub struct MergeBlake3;

impl Merge for MergeBlake3 {
    type Item = [u8; 32]; // 这里我们存储 32 字节的哈希值

    fn merge(lhs: &Self::Item, rhs: &Self::Item) -> ckb_merkle_mountain_range::Result<Self::Item> {
        let mut hasher = blake3::Hasher::new();
        // 按照 ckb-mmr 惯例或比特币惯例，通常是先拼接再哈希
        hasher.update(lhs);
        hasher.update(rhs);
        Ok(*hasher.finalize().as_bytes())
    }
}

/// 证据存储仓 (Evidence Store)
/// 负责管理 MMR 的状态和数据追加
pub struct EvidenceStore {
    // 内存存储后端 (MemStore 是 ckb-mmr 提供的简单实现)
    store: MemStore<[u8; 32]>,
    // 当前 MMR 的大小 (节点总数，包含叶子和内部节点)
    mmr_size: u64,
}

impl EvidenceStore {
    pub fn new() -> Self {
        Self {
            store: MemStore::default(),
            mmr_size: 0,
        }
    }

    /// 追加新的证据到 MMR
    /// 返回: (新的 Root Hash, 插入位置 pos)
    pub fn append(&mut self, evidence: &Evidence) -> anyhow::Result<([u8; 32], u64)> {
        // 1. 计算证据本身的哈希 (Blake3)
        // 注意：这里是对 Evidence 原始数据哈希，作为 MMR 的叶子节点
        let payload = serde_json::to_vec(evidence)?;
        let leaf_hash = *blake3::hash(&payload).as_bytes();

        // 2. 将叶子哈希插入 MMR
        // MMR::new(current_size, store)
        let mut mmr = MMR::<[u8; 32], MergeBlake3, _>::new(self.mmr_size, &self.store);
        
        let pos = mmr.push(leaf_hash).map_err(|e| anyhow::anyhow!("MMR append error: {}", e))?;
        
        // 3. 更新 size
        self.mmr_size = mmr.mmr_size();

        // 4. 计算并返回新的 Root
        let root = mmr.get_root().map_err(|e| anyhow::anyhow!("MMR get_root error: {}", e))?;
        
        Ok((root, pos))
    }

    /// 获取特定位置的 Merkle Proof
    pub fn get_proof(&self, pos_list: Vec<u64>) -> anyhow::Result<ckb_merkle_mountain_range::MerkleProof<[u8; 32], MergeBlake3>> {
        let mmr = MMR::<[u8; 32], MergeBlake3, _>::new(self.mmr_size, &self.store);
        mmr.gen_proof(pos_list).map_err(|e| anyhow::anyhow!("MMR gen_proof error: {}", e))
    }
}
