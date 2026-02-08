mod evidence;
mod fingerprint;
mod signer;
mod mmr_store;
mod api;
mod config;

use config::Config;
use mmr_store::EvidenceStore;
use signer::EvidenceSigner;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ----------------------------------------------------------------
    // 0. 加载配置
    // ----------------------------------------------------------------
    let config = Config::from_env();
    println!("⚙️  配置加载完成: Host={}:{}, DB={}, Key={}", 
        config.host, config.port, config.db_path, config.key_path);

    // ----------------------------------------------------------------
    // 1. 系统初始化 & 身份加载
    // ----------------------------------------------------------------
    println!("🛡️ [原镜 Yuanjing] 司法级可信确证服务启动中...");
    
    // 加载或生成密钥对 (Task C)
    let signer = EvidenceSigner::load_or_generate(&config.key_path)?;
    let pub_key_bytes = signer.public_key().to_bytes();
    println!("🆔 服务身份ID (Public Key): {}", hex::encode(pub_key_bytes));

    // 初始化 MMR 存储 (Task B)
    let store = EvidenceStore::new(&config.db_path);
    println!("📚 证据库 (MMR) 初始化完成 (Headless Mode)");

    // ----------------------------------------------------------------
    // 2. 状态共享容器
    // ----------------------------------------------------------------
    let shared_state = Arc::new(api::AppState {
        store: Arc::new(Mutex::new(store)),
        signer: Arc::new(signer),
    });

    // ----------------------------------------------------------------
    // 3. 启动 HTTP 服务 (Task D)
    // ----------------------------------------------------------------
    let app = api::app(shared_state);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await?;
    
    println!("🚀 API 服务已运行在: http://{}", addr);
    println!("   - POST /prove   : 提交图片指纹进行确证");
    println!("   - GET  /audit/:pos : 获取特定位置的 Merkle Proof");

    axum::serve(listener, app).await?;

    Ok(())
}
