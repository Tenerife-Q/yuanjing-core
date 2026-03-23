





# Yuanjing-Core (原镜) 🔍

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)
![Cryptography](https://img.shields.io/badge/crypto-ed25519%20%7C%20Blake3-blue.svg)
![Blockchain](https://img.shields.io/badge/blockchain-MMR%20%7C%20Fact%20Anchoring-green.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

**Yuanjing** (meaning "Original Mirror") is a high-performance, cryptographically-secured backend service for image contextual authenticity verification. It bridges the gap between **Multi-modal AI Analysis (VLM)** and **Blockchain-anchored Evidence Storage**.

Built entirely in **Rust**, this core engine is designed to combat the escalating challenges of AIGC (AI-Generated Content) manipulation, providing tamper-proof, mathematically verifiable authentication records for digital media.

## 🌟 Core Features

- **Perceptual Fingerprinting**: Utilizes `pHash` (Perceptual Hash) alongside strict cryptographic hashes (`SHA-256`) to tolerate benign image compression while detecting fundamental structural alterations.
- **Multi-Modal Fact-Checking**: Integrates with external VLMs to verify semantic logic, temporal accuracy, and physical lighting consistency.
- **Cryptographic Anchoring**: Generates `ed25519` digital signatures for non-repudiation.
- **Append-Only Evidence Ledger**: Implements an embedded **Merkle Mountain Range (MMR)** over a `sled` database to guarantee the temporal sequence and immutability of the verification history.
- **SAPT (Sparse Activation Path Tracking)**: Provides auditable, white-box verification paths for AI inference.

## 🏗️ System Architecture

Yuanjing-Core operates on a dual-engine architecture:

1. **Inference & Fingerprint Engine**: Extracts `pHash`/`SHA256` directly from image buffers. Evaluates semantic consistency and generates an initial trust score.
2. **Blockchain Authentication Engine**: Packages the evaluation metadata, signs it via an `ed25519` keypair, and appends the root hash to the MMR state tree.

### Tech Stack
- **Core HTTP Server**: `axum`, `tokio` (Async multi-threading)
- **Cryptography**: `ed25519-dalek`, `blake3`, `sha2`
- **Image Processing**: `image`, `img_hash`
- **Embedded Database**: `sled`
- **Data Structures**: `ckb-merkle-mountain-range`, `bcs`, `serde`

## 🚀 Getting Started

### Prerequisites
- Rust `1.70+` and Cargo installed.

### Installation & Run

```bash
# Clone the repository
git clone https://github.com/Tenerife-Q/yuanjing-core.git
cd yuanjing-core

# Build the project in release mode
cargo build --release

# Run the server
cargo run --release
```

*Note: On the first run, the system will automatically generate a new `ed25519` keypair (`yuanjing.key`) and initialize the MMR database in the `./data` directory.*

## 🔌 Core API Endpoints

### 1. Submit Evidence (`POST /prove`)
Submit an image's metadata and VLM verdict to generate a cryptographic proof.

**Request:**
```json
{
  "image_path": "data/test_image.jpg",
  "verdict": false,
  "confidence": 0.95,
  "source": "user_submission",
  "prompt_pool_hash": "abc123..."
}
```

**Response (200 OK):**
```json
{
  "root_hash": "a1b2c3d4...",
  "leaf_pos": 42,
  "signature": "e4f5...",
  "evidence_dump": {
    "image_phash": "1a2b3c4d5e6f...",
    "image_sha256": "...",
    "verdict": false,
    "confidence": "0.95",
    "timestamp": 1710000000
  }
}
```

### 2. Verify Audit Proof (`GET /audit/{position}`)
Retrieve the Merkle proof for a specific evidence entry, enabling trustless third-party verification.

## 🛡️ Security Considerations

- **Key Management**: In the current development phase, the private key is stored locally in the file system. For production, integration with a Hardware Security Module (HSM) or a secure enclave is strongly recommended.
- **Cryptographic Primitives**: We rely on industry-standard ECC (Elliptic Curve Cryptography) and Blake3 for collision-resistant, fast hashing.

## 🤝 Contributing

This project is actively maintained as part of an academic research initiative focusing on Web3 media provenance and AI safety. Pull Requests and structural suggestions are highly welcome. 

## 📄 License

This project is licensed under the [MIT License](LICENSE).
```

