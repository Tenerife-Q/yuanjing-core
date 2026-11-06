use ckb_merkle_mountain_range::{MMR, Merge, MMRStore, Result as MMRResult, Error as MMRError};
use crate::evidence::Evidence;
use sled::Db;
use std::convert::TryInto;

/// 合并策略 (Merge Strategy)
pub struct MergeBlake3;

impl Merge for MergeBlake3 {
    type Item = [u8; 32];

    fn merge(lhs: &Self::Item, rhs: &Self::Item) -> MMRResult<Self::Item> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(lhs);
        hasher.update(rhs);
        Ok(*hasher.finalize().as_bytes())
    }
}

/// 基于 Sled 的持久化存储
#[derive(Clone)]
pub struct SledStore {
    db: Db,
}

impl SledStore {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    pub fn get_meta_size(&self) -> u64 {
        let meta = self.db.open_tree("meta").expect("open meta tree");
        match meta.get(b"size") {
            Ok(Some(v)) => {
                 let arr: [u8; 8] = v.as_ref().try_into().unwrap_or([0; 8]);
                 u64::from_be_bytes(arr)
            },
            _ => 0
        }
    }

    pub fn set_meta_size(&self, size: u64) -> anyhow::Result<()> {
        let meta = self.db.open_tree("meta")?;
        meta.insert(b"size", &size.to_be_bytes())?;
        meta.flush()?;
        Ok(()) 
    }

    pub fn flush(&self) -> anyhow::Result<()> {
        self.db.flush()?;
        Ok(())
    }

    pub fn register_model(&self, hash: &str, description: &str) -> anyhow::Result<()> {
        let tree = self.db.open_tree("models_allowlist")?;
        tree.insert(hash, description)?;
        tree.flush()?;
        Ok(())
    }

    pub fn is_model_authorized(&self, hash: &str) -> bool {
        if let Ok(tree) = self.db.open_tree("models_allowlist") {
            return tree.contains_key(hash).unwrap_or(false);
        }
        false
    }
}

// 为引用类型实现 MMRStore
// 这样每次 MMR 操作时，我们可以传入 &store
impl MMRStore<[u8; 32]> for &SledStore {
    fn get_elem(&self, pos: u64) -> MMRResult<Option<[u8; 32]>> {
        let key = pos.to_be_bytes();
        match self.db.get(key) {
            Ok(Some(v)) => {
                if v.len() != 32 {
                    return Err(MMRError::StoreError("Invalid data length in DB".to_string()));
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&v);
                Ok(Some(arr))
            },
            Ok(None) => Ok(None),
            Err(e) => Err(MMRError::StoreError(e.to_string())),
        }
    }

    fn append(&mut self, pos: u64, elems: Vec<[u8; 32]>) -> MMRResult<()> {
        let mut current_pos = pos;
        let mut batch = sled::Batch::default();
        for elem in elems {
            batch.insert(&current_pos.to_be_bytes(), &elem);
            current_pos += 1;
        }
        self.db.apply_batch(batch).map_err(|e| MMRError::StoreError(e.to_string()))
    }
}

/// 证据仓库 (Evidence Store)
pub struct EvidenceStore {
    store: SledStore,
    mmr_size: u64,
}

impl EvidenceStore {
    /// 初始化仓库 (加载 DB)
    pub fn new(db_path: &str) -> Self {
        let store = SledStore::new(db_path).expect("Failed to open Sled DB");
        let mmr_size = store.get_meta_size();
        
        println!("📚 MMR Store Loaded. Size: {}", mmr_size);

        Self {
            store,
            mmr_size,
        }
    }

    /// 核心功能：证据上链入库
    pub fn append(&mut self, evidence: &Evidence) -> anyhow::Result<([u8; 32], u64)> {
        // Step 0: 白名单校验 (Model Governance)
        // 防止未授权的模型版本写入区块链
        if !self.store.is_model_authorized(&evidence.prompt_pool_hash) {
             return Err(anyhow::anyhow!("Unauthorized Model Version: '{}'. Please register first.", evidence.prompt_pool_hash));
        }

        let payload = bcs::to_bytes(evidence)?;
        let leaf_hash = *blake3::hash(&payload).as_bytes();

        let mut mmr = MMR::<[u8; 32], MergeBlake3, _>::new(self.mmr_size, &self.store);
        
        let pos = mmr.push(leaf_hash).map_err(|e| anyhow::anyhow!("MMR append error: {}", e))?;
        
        let new_size = mmr.mmr_size();
        let root = mmr.get_root().map_err(|e| anyhow::anyhow!("MMR get_root error: {}", e))?;

        mmr.commit().map_err(|e| anyhow::anyhow!("MMR commit error: {}", e))?;

        // 持久化新的 Size
        self.store.set_meta_size(new_size)?;
        
        // 显式 flush 确保数据落盘
        self.store.flush()?;

        self.mmr_size = new_size;
        
        Ok((root, pos))
    }

    /// 注册新模型
    pub fn register_model(&self, hash: &str, description: &str) -> anyhow::Result<()> {
        self.store.register_model(hash, description)
    }

    /// 核心功能：开具证明
    pub fn get_proof(&self, pos_list: Vec<u64>) -> anyhow::Result<ckb_merkle_mountain_range::MerkleProof<[u8; 32], MergeBlake3>> {
        let mmr = MMR::<[u8; 32], MergeBlake3, _>::new(self.mmr_size, &self.store);
        mmr.gen_proof(pos_list).map_err(|e| anyhow::anyhow!("MMR gen_proof error: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn get_temp_db_path(name: &str) -> String {
        let path = std::env::temp_dir().join(format!("yuanjing_test_db_{}", name));
        if path.exists() {
            let _ = fs::remove_dir_all(&path);
        }
        path.to_str().unwrap().to_string()
    }

    fn mock_evidence(model_hash: &str) -> Evidence {
        Evidence {
            image_phash: "12345678".to_string(),
            image_sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            verdict: true,
            confidence: "0.99".to_string(),
            activated_prompts: vec![10, 20],
            prompt_pool_hash: model_hash.to_string(),
            external_knowledge_hash: "wiki_ref".to_string(),
            timestamp: 1710000000,
        }
    }

    #[test]
    fn test_model_auth_and_mmr_append() {
        let db_path = get_temp_db_path("auth_append");
        let mut store = EvidenceStore::new(&db_path);

        let model_hash = "0000000000000000000000000000000000000000000000000000000000000000";
        
        // Before register, append should fail
        let ev = mock_evidence(model_hash);
        assert!(store.append(&ev).is_err(), "Append should fail for unregistered model");

        // Register model
        store.register_model(model_hash, "test model").expect("Failed to register model");

        // Now append should succeed
        let (root, pos) = store.append(&ev).expect("Append should succeed after registration");
        assert_ne!(root, [0u8; 32]);
        assert_eq!(pos, 0);

        // Generate proof
        let proof = store.get_proof(vec![pos]).expect("Failed to generate proof");
        assert_eq!(proof.proof_items().len(), 0); // Single leaf MMR has no sibling proof items

        let _ = fs::remove_dir_all(&db_path);
    }
}
