#!/bin/bash

# Ensure the script is run from the repo root
if [ ! -d .git ]; then
  echo "This script must be run from the repository root."
  exit 1
fi

# Check if BACKEND_URL is set, else default to http://127.0.0.1:3000
BACKEND_URL=${BACKEND_URL:-http://127.0.0.1:3000}

# Register PROMPT_POOL_HASH if not provided, default to mmfn_v1
PROMPT_POOL_HASH=${PROMPT_POOL_HASH:-mmfn_v1}

# Ensure data/samples/news.jpg exists
if [ ! -f data/samples/news.jpg ]; then
  echo "Downloading test image..."
  mkdir -p data/samples
  curl -o data/samples/news.jpg https://picsum.photos/400.jpg
fi

# Call the POST /model/register
response=$(curl -s -X POST -H "Content-Type: application/json" -d '{"prompt_pool_hash": "$PROMPT_POOL_HASH"}' "$BACKEND_URL/model/register")

# Parse leaf_pos from response
leaf_pos=$(echo "$response" | jq -r '.leaf_pos')
if [ "$leaf_pos" == "null" ]; then
  echo "Error: leaf_pos not returned from registration."
  exit 1
fi

# Call POST /prove
image_path=$(realpath data/samples/news.jpg)
curl -X POST -H "Content-Type: application/json" -d '{"image_path": "$image_path", "verdict": false, "confidence": 0.99, "source": "manual-test", "prompt_pool_hash": "$PROMPT_POOL_HASH"}' "$BACKEND_URL/prove"

# Call GET /audit/{leaf_pos}
curl -X GET "$BACKEND_URL/audit/$leaf_pos"