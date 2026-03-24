use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex; 
use tower_http::cors::CorsLayer;

use crate::{evidence::Evidence, fingerprint, mmr_store::EvidenceStore, signer::EvidenceSigner};

// ==========================================
// 1. 定义应用状态 (Shared State)
// ==========================================
// 所有的 HTTP 请求都会共享这个状态。
// 使用 Arc 保证多线程安全，Mutex 保证写操作互斥（因为 MMR 是追加写的）。
pub struct AppState {
    pub signer: Arc<EvidenceSigner>,
    pub store: Arc<Mutex<EvidenceStore>>,
}

// ==========================================
// 2. 数据传输对象 (DTOs)
// ==========================================

// 请求：提交证据
#[derive(Deserialize)]
pub struct ProveRequest {
    // 实际场景中这里也是 Mock 的，前端发来图片路径
    pub image_path: String,
    
    // 模拟的 AI 参数（如果王嗣萱的模块调用，这里就是真实 AI 结果）
    pub verdict: bool,
    pub confidence: f64,
    pub source: String, // 来源说明
    /// AI model version hash; must be pre-registered via `/model/register`
    pub prompt_pool_hash: String,
}

// 响应：存证回执
#[derive(Serialize)]
pub struct ProveReceipt {
    pub root_hash: String,
    pub leaf_pos: u64,
    pub signature: String, // Hex encoded
    pub evidence_dump: Evidence, // 返回完整证据包供核对
}

// 响应：Merkle Proof
#[derive(Serialize)]
pub struct AuditResponse {
    pub proof_valid: bool, // 仅作为标记，实际验证在客户端
    pub leaf_pos: u64,
    pub proof_hex: Vec<String>, // 将 proof path 转为 Hex 数组方便前端展示
}

// 请求：注册模型
#[derive(Deserialize)]
pub struct ModelRegisterRequest {
    pub hash: String,
    pub description: String,
}

// 响应：注册成功
#[derive(Serialize)]
pub struct ModelRegisterResponse {
    pub status: String,
}

// ==========================================
// 3. API 路由构建
// ==========================================
pub fn app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/prove", post(submit_evidence))
        .route("/audit/{pos}", get(get_audit_proof))
        .route("/model/register", post(register_model))
        .layer(CorsLayer::permissive()) // ⚠️ 开发模式：允许所有跨域
        .with_state(state)
}

// ==========================================
// 4. 处理函数 (Handlers)
// ==========================================

/// 接口：注册新的 AI 模型
async fn register_model(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ModelRegisterRequest>,
) -> Result<Json<ModelRegisterResponse>, (StatusCode, String)> {
    println!("🆕 注册模型: {} ({})", req.hash, req.description);
    
    // 我们暂时需要在这里获取 lock，虽然 register_model 本身在 store 里是 &self (只读 self, 但内部有 db 操作)
    // 但 EvidenceStore 的定义目前是需要在 Mutex 里的。
    // 其实 register_model 只需要 &EvidenceStore，不需要 &mut EvidenceStore。
    // 但 AppState 里是 Mutex<EvidenceStore>，所以还是得 lock。
    let store = state.store.lock().await; // Lock handled automatically
    store.register_model(&req.hash, &req.description)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ModelRegisterResponse {
        status: "Registered".to_string(),
    }))
}

/// 接口：提交证据并上链
async fn submit_evidence(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ProveRequest>,
) -> Result<Json<ProveReceipt>, (StatusCode, String)> {
    
    println!("📥 收到存证请求: 图片={}, 判定={}", req.image_path, req.verdict);

    // 1. 校验 confidence 字段
    if req.confidence.is_nan() || req.confidence.is_infinite() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid confidence value: must be a finite number, got NaN or Inf".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&req.confidence) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Invalid confidence value: {} is out of range [0.0, 1.0]",
                req.confidence
            ),
        ));
    }

    // 2. 提取指纹 (CPU 密集型操作，已移至 spawn_blocking 优化)
    let img_path_str = req.image_path.clone(); // Clone for closure
    let (sha, phash) = tokio::task::spawn_blocking(move || {
        let path = std::path::Path::new(&img_path_str);
        if !path.exists() {
            return Err(anyhow::anyhow!("图片不存在: {}", img_path_str));
        }
        fingerprint::generate_fingerprints(path)
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join error: {}", e)))?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 3. 构造 Evidence (模拟 AI 结合 Rust 提取的特征)
    let evidence = Evidence {
        image_phash: phash,
        image_sha256: sha,
        verdict: req.verdict,
        confidence: req.confidence.to_string(),
        activated_prompts: vec![1, 2, 99], // Mock
        prompt_pool_hash: req.prompt_pool_hash,
        external_knowledge_hash: "mock_wiki_hash_xyz789".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    };

    // 4. 签名
    let signature = state.signer.sign(&evidence)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 5. 存入 MMR (需要获取锁)
    let (root, pos) = {
        let mut store = state.store.lock().await;
        store.append(&evidence)
            .map_err(|e| {
                if e.to_string().contains("Unauthorized Model") {
                     (StatusCode::BAD_REQUEST, e.to_string())
                } else {
                     (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                }
            })?
    };

    println!("✅ 存证成功: Root={}, Pos={}", hex::encode(root), pos);

    Ok(Json(ProveReceipt {
        root_hash: hex::encode(root),
        leaf_pos: pos,
        signature: hex::encode(signature.to_bytes()),
        evidence_dump: evidence,
    }))
}

/// 接口：获取审计证明
async fn get_audit_proof(
    State(state): State<Arc<AppState>>,
    Path(pos): Path<u64>,
) -> Result<Json<AuditResponse>, (StatusCode, String)> {
    println!("🔍 收到审计请求: Pos={}", pos);

    let store = state.store.lock().await;
    
    // 获取 Proof
    let proof = store.get_proof(vec![pos])
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("获取 Proof 失败: {}", e)))?;

    // 序列化 Proof 路径
    let proof_hex: Vec<String> = proof
        .proof_items()
        .iter()
        .map(|hash| hex::encode(hash))
        .collect();

    Ok(Json(AuditResponse {
        proof_valid: true,
        leaf_pos: pos,
        proof_hex,
    }))
}
