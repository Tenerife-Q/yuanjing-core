use img_hash::{HasherConfig, HashAlg}; 
use sha2::{Sha256, Digest};
use std::fs;
use std::path::Path;

pub fn generate_fingerprints(path: &Path) -> anyhow::Result<(String, String)> {
    // 1. SHA256 (保持不变)
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let sha_hash = format!("{:x}", hasher.finalize());

    // 2. pHash (逻辑微调)
    let img = img_hash::image::open(path)?;
    let hasher = HasherConfig::new()
        .hash_alg(HashAlg::Gradient) // 梯度算法
        .hash_size(8, 8)
        .to_hasher();
    
    // img_hash 库的 hash_image 返回的是 Hash 对象，我们需要转为 Base64 或 Hex
    let phash = hasher.hash_image(&img);

    Ok((sha_hash, phash.to_base64()))
}