#!/usr/bin/env bash
set -euo pipefail

# Local smoke test for Yuanjing Core.
# Usage:
#   BACKEND_URL=http://127.0.0.1:3000 PROMPT_POOL_HASH=mmfn_v1 ./scripts/smoke_test_local.sh

if [[ ! -f Cargo.toml ]]; then
  echo "Error: run this script from the repo root (where Cargo.toml exists)."
  exit 1
fi

BACKEND_URL="${BACKEND_URL:-http://127.0.0.1:3000}"
PROMPT_POOL_HASH="${PROMPT_POOL_HASH:-mmfn_v1}"
IMAGE_REL="${IMAGE_REL:-data/samples/news.jpg}"

echo "[0] Repo: $(pwd)"
echo "[1] Backend: $BACKEND_URL"
echo "[2] prompt_pool_hash: $PROMPT_POOL_HASH"

echo "[3] Check backend reachable"
curl -fsS "$BACKEND_URL/audit/0" >/dev/null

echo "[4] Ensure JPEG test image exists"
mkdir -p "$(dirname "$IMAGE_REL")"
if [[ ! -s "$IMAGE_REL" ]]; then
  echo "  downloading JPEG to $IMAGE_REL"
  curl -fsSL -o "$IMAGE_REL" "https://picsum.photos/400.jpg"
fi

if command -v file >/dev/null 2>&1; then
  echo "  file: $(file "$IMAGE_REL")"
fi

IMAGE_ABS="$(python3 -c 'import os,sys; print(os.path.abspath(sys.argv[1]))' "$IMAGE_REL")"
echo "  image_path: $IMAGE_ABS"

echo "[5] POST /model/register"
curl -fsS -X POST "$BACKEND_URL/model/register" \
  -H "Content-Type: application/json" \
  -d "{\"hash\":\"$PROMPT_POOL_HASH\",\"description\":\"local smoke test\"}" >/dev/null

echo "[6] POST /prove"
PROVE_JSON="$(curl -fsS -X POST "$BACKEND_URL/prove" \
  -H "Content-Type: application/json" \
  -d "{
    \"image_path\":\"$IMAGE_ABS\",
    \"verdict\": false,
    \"confidence\": 0.99,
    \"source\": \"manual-test\",
    \"prompt_pool_hash\": \"$PROMPT_POOL_HASH\"
  }")"
echo "  /prove => $PROVE_JSON"

echo "[7] Extract leaf_pos"
LEAF_POS="$(python3 -c 'import json,sys; print(json.loads(sys.stdin.read())["leaf_pos"])' <<<"$PROVE_JSON")"
echo "  leaf_pos=$LEAF_POS"

echo "[8] GET /audit/{leaf_pos}"
AUDIT_JSON="$(curl -fsS "$BACKEND_URL/audit/$LEAF_POS")"
echo "  /audit => $AUDIT_JSON"

echo "OK: smoke test passed"
