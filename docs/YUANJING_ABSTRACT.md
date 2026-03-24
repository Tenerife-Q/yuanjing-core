
### Project Title: **"Yuanjing" (Original Mirror) – A Multi-Dimensional Image Authenticity & Blockchain Provenance Framework**

#### 1. Project Abstract
"Yuanjing" is a comprehensive system designed to combat the erosion of digital trust caused by **AI-generated content (AIGC), sophisticated digital manipulation (Photoshop), and contextual misinformation (Contextual Spoofing).** The framework employs a dual-engine approach: a **Vision-Language Model (VLM)** for deep feature inference and a **Blockchain-based Factual Anchoring** mechanism. By integrating micro-level forgery detection with macro-level semantic verification, Yuanjing provides an end-to-end solution for identifying false information and securing immutable evidence.

#### 2. The Core Challenge: A Three-Tiered Threat
Current verification methods fail because they cannot address the full spectrum of image forgery:
* **Direct Forgery:** AI-generated images and manual pixel-level edits (PS) are becoming indistinguishable from reality.
* **Contextual Fraud:** "Old photos used in new stories"—where a genuine image is detached from its original context to propagate rumors.
* **Verification Fragility:** Standard cryptographic hashes (SHA-256) are too sensitive to benign edits (compression), leading to "false negatives" in decentralized storage.

#### 3. Technical Architecture & Methodology

**A. Multi-Granular Forgery Detection (VLM + Adversarial Training)**
To detect AI-generated and manipulated images:
* **Physical Consistency Analysis:** The VLM analyzes high-dimensional features like lighting vectors, shadow geometry, and texture anomalies that are often logically inconsistent in AI models.
* **Dynamic Adversarial Defense:** We co-train the detection model against an "Adversary Generator." This ensures the system remains robust even as AIGC forgery techniques evolve.

**B. Semantic Logic & Contextual Verification**
To solve the "Old Photo, New Story" problem:
* **Cross-Modal Alignment:** The system extracts semantic themes from the image and cross-references them with on-chain metadata (timestamps, geolocation, and news sources).
* **Logic Conflict Detection:** If the visual contents (e.g., a specific vehicle model) contradict the claimed era or location in the metadata, the system flags the contextual mismatch.

**C. Trusted Evidence Chain (pHash + Blockchain)**
* **Perceptual Resilience:** We utilize **Perceptual Hashing (pHash)** to create a structural fingerprint. This ensures the image can be identified and verified even after being compressed or resized during network transmission.
* **Decentralized Anchoring:** Detection results and structural fingerprints are signed and anchored on-chain, creating a transparent, tamper-proof audit trail for every verification.

#### 4. Project Goals & Innovation
1.  **Holistic Detection:** Integrating AI-artifact detection with semantic truth-seeking.
2.  **Mitigating VLM Hallucinations:** Using blockchain-stored "Ground Truth" as a logic anchor to improve the reliability of AI reasoning.
3.  **Bridge to DePIN:** Providing a scalable, compression-resilient verification layer for Web3 hardware and media infrastructure.

