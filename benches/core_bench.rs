use criterion::{criterion_group, criterion_main, Criterion};
use std::path::Path;
use yuanjing_core::{fingerprint, mmr_store::EvidenceStore, evidence::Evidence};
use std::sync::Once;

static INIT: Once = Once::new();

fn setup_env() {
    INIT.call_once(|| {
        // Ensure data directory exists
        let _ = std::fs::create_dir_all("data/temp_bench");
        // Ensure a sample file exists for fingerprinting
        if !Path::new("data/samples/original.jpg").exists() {
             // Mock one if missing logic could be added here, but assuming it exists from previous steps
        }
    });
}

fn bench_fingerprint(c: &mut Criterion) {
    setup_env();
    let img_path = Path::new("data/samples/original.jpg");
    
    // Only run if file exists
    if img_path.exists() {
        c.bench_function("fingerprint_generation", |b| {
            b.iter(|| {
                fingerprint::generate_fingerprints(img_path).unwrap();
            })
        });
    } else {
        println!("⚠️  Skipping fingerprint bench: Sample image not found.");
    }
}

fn bench_mmr_append(c: &mut Criterion) {
    setup_env();
    // Use a separate temp db for benchmarking
    let mut store = EvidenceStore::new("data/temp_bench/bench_mmr_db");

    // Register a mock model so appends succeed
    let mock_pool_hash = "mock_pool_hash_abc123";
    let _ = store.register_model(mock_pool_hash, "Bench Model");

    let evidence = Evidence {
        image_phash: "mock_phash".to_string(),
        image_sha256: "mock_sha256".to_string(),
        verdict: true,
        confidence: "0.99".to_string(),
        activated_prompts: vec![1, 2, 3],
        prompt_pool_hash: mock_pool_hash.to_string(),
        external_knowledge_hash: "mock_ext".to_string(),
        timestamp: 1234567890,
    };

    c.bench_function("mmr_append_entry", |b| {
        b.iter(|| {
            // We append the same evidence repeatedly. Sled handles this fine.
            store.append(&evidence).unwrap();
        })
    });
    
    // Cleanup? Sled typically keeps locks. We might just let it be.
}

criterion_group!(benches, bench_fingerprint, bench_mmr_append);
criterion_main!(benches);
