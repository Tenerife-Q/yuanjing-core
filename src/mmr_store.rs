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
    pub fn new() -> Self {
        let db_path = "data/db/mmr_db";
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

    /// 核心功能：开具证明
    pub fn get_proof(&self, pos_list: Vec<u64>) -> anyhow::Result<ckb_merkle_mountain_range::MerkleProof<[u8; 32], MergeBlake3>> {
        let mmr = MMR::<[u8; 32], MergeBlake3, _>::new(self.mmr_size, &self.store);
        mmr.gen_proof(pos_list).map_err(|e| anyhow::anyhow!("MMR gen_proof error: {}", e))
    }
}
