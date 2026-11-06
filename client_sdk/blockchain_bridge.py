"""
Blockchain Bridge for Yuanjing-Core
Provides seamless integration between Vision-Language Models (e.g., ViRTA) and the Rust Blockchain Anchoring Backend.
"""

import os
import time
import requests
from pathlib import Path
from dataclasses import dataclass
from typing import Optional, Dict, Any

# Map dataset labels to binary authenticity verdicts
# (e.g. Weibo/GossipCop: 0 = Real, 1 = Fake)
DATASET_LABEL_TO_VERDICT = {
    "weibo": {0: True, 1: False},        # True = Real/Authentic, False = Fake
    "gossipcop": {0: True, 1: False},
    "default": {0: True, 1: False}
}

def normalize_confidence(confidence: float) -> float:
    """Ensure confidence is a finite float within [0.0, 1.0]."""
    try:
        val = float(confidence)
        if val != val:  # NaN check
            return 0.5
        return max(0.0, min(val, 1.0))
    except (TypeError, ValueError):
        return 0.5

@dataclass
class PredictionPayload:
    dataset: str
    image_path: str
    predicted_label: int
    confidence: float
    source: str = "virta_vlm_inference"
    prompt_pool_hash: str = "0" * 64
    external_knowledge_hash: str = "0" * 64

    def to_api_payload(self) -> Dict[str, Any]:
        mapping = DATASET_LABEL_TO_VERDICT.get(self.dataset.lower(), DATASET_LABEL_TO_VERDICT["default"])
        verdict = mapping.get(self.predicted_label, False)
        
        abs_image_path = str(Path(self.image_path).resolve())
        
        return {
            "image_path": abs_image_path,
            "verdict": verdict,
            "confidence": normalize_confidence(self.confidence),
            "source": self.source,
            "prompt_pool_hash": self.prompt_pool_hash
        }

def submit_proof_with_retry(payload: PredictionPayload, base_url: Optional[str] = None, max_retries: int = 3, retry_delay: float = 1.0) -> Dict[str, Any]:
    """Submit VLM prediction proof to yuanjing-core backend with automatic retry."""
    if base_url is None:
        base_url = os.environ.get("YUANJING_BASE_URL", "http://localhost:3000")
        
    endpoint = f"{base_url.rstrip('/')}/prove"
    json_data = payload.to_api_payload()

    for attempt in range(1, max_retries + 1):
        try:
            response = requests.post(endpoint, json=json_data, timeout=10)
            if response.status_code == 200:
                return response.json()
            elif response.status_code == 400 and "Unauthorized Model" in response.text:
                raise RuntimeError(f"Model hash not authorized: {response.text}")
            else:
                if attempt == max_retries:
                    response.raise_for_status()
        except (requests.RequestException, RuntimeError) as e:
            if attempt == max_retries:
                raise RuntimeError(f"Failed to submit proof after {max_retries} attempts: {e}")
            time.sleep(retry_delay)
    
    raise RuntimeError("Unexpected end of retry loop")
