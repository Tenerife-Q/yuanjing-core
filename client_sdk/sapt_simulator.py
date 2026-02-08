import os
import blake3
import random
import requests

PROMPT_POOL_SIZE = 100_000
POOL_FILE = "mock_prompt_pool.bin"
API_URL = "http://localhost:3000"

def generate_mock_pool(size_mb=5):
    """
    生成一个模拟的 Prompt Pool 文件，并返回其 Blake3 哈希。
    """
    if os.path.exists(POOL_FILE):
        print(f"📦 检测到现有 Pool 文件: {POOL_FILE}")
        with open(POOL_FILE, "rb") as f:
            data = f.read()
    else:
        print(f"🌊 生成 Mock Prompt Pool ({size_mb}MB)...")
        data = os.urandom(size_mb * 1024 * 1024)
        with open(POOL_FILE, "wb") as f:
            f.write(data)
    
    # 计算整体哈希
    hasher = blake3.blake3()
    hasher.update(data)
    pool_hash = hasher.hexdigest()
    print(f"🔑 Pool Hash: {pool_hash}")
    return pool_hash

def register_model(pool_hash: str):
    """
    向 Rust 服务器注册模型哈希
    """
    print(f"\n📝 正在注册模型: {pool_hash}...")
    try:
        resp = requests.post(f"{API_URL}/model/register", json={
            "hash": pool_hash,
            "description": "SAPT-v2.0-Mock (Copilot Generated)"
        })
        if resp.status_code == 200:
            print("✅ 模型注册成功！")
            return True
        else:
            print(f"❌ 注册失败: {resp.text}")
            return False
    except Exception as e:
        print(f"❌ 连接错误: {e}")
        return False

def get_activated_prompts(image_path: str, top_k=5) -> list[int]:
    """
    SAPT 核心稀疏路由模拟:
    基于图片内容的哈希，确定性地选择 Top-K 个专家索引。
    这保证了如果不改变图片，推理路径永远一致。
    """
    # 1. 读取图片 (如果不存在则用路径字符串模拟内容)
    if not os.path.exists(image_path):
        content = f"mock_content_for_{image_path}".encode()
    else:
        with open(image_path, "rb") as f:
            content = f.read()
    
    # 2. 计算图片哈希 (作为路由种子)
    # 在真实 SAPT 中，这里会是 Vision Encoder 输出的 Embedding
    img_hash = blake3.blake3(content).digest()
    
    # 3. 确定性随机数生成 (Deterministic RNG)
    # 使用图片哈希的前8个字节作为 Seed
    seed_int = int.from_bytes(img_hash[:8], 'little')
    rng = random.Random(seed_int)
    
    # 4. 稀疏采样
    indices = rng.sample(range(PROMPT_POOL_SIZE), top_k)
    indices.sort()
    
    print(f"🧠 [SAPT] 图片 {os.path.basename(image_path)} 激活了专家: {indices}")
    return indices
