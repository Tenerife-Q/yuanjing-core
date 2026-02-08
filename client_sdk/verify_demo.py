import requests
import json
import nacl.signing
import nacl.encoding
from evidence_schema import Evidence
import sapt_simulator

# 配置
API_URL = "http://localhost:3000"
SERVER_PUBKEY_HEX = "818ac195d7fb669fdb05e695527193e3377a6b7db04af60cb8ff3c7978b182fb"

def verify_receipt(receipt: dict, server_pubkey_hex: str):
    print("\n🔍 开始验证存证回执...")
    
    # 1. 重建 Evidence 对象
    ev_data = receipt['evidence_dump']
    evidence = Evidence(
        image_phash=ev_data['image_phash'],
        image_sha256=ev_data['image_sha256'],
        verdict=ev_data['verdict'],
        confidence=ev_data['confidence'],
        activated_prompts=ev_data['activated_prompts'],
        prompt_pool_hash=ev_data['prompt_pool_hash'],
        external_knowledge_hash=ev_data['external_knowledge_hash'],
        timestamp=ev_data['timestamp']
    )

    # 2. 本地执行 BCS 序列化
    message_bytes = evidence.to_bcs()
    
    # 3. 验证签名 (Ed25519)
    try:
        verify_key = nacl.signing.VerifyKey(server_pubkey_hex, encoder=nacl.encoding.HexEncoder)
        signature_bytes = nacl.encoding.HexEncoder.decode(receipt['signature'])
        verify_key.verify(message_bytes, signature_bytes)
        print("✅ 签名验证通过: 数据完整且来源可信！")
        return True
    except nacl.exceptions.BadSignatureError:
        print("❌ 签名验证失败: 数据可能被篡改或私钥不匹配！")
        return False

def main():
    # 1. 准备模型 (模拟器)
    print("--- 步骤 1: 初始化 SAPT 模拟器 ---")
    pool_hash = sapt_simulator.generate_mock_pool()
    
    # 2. 注册模型 (如果不注册，后续 prove 应该失败)
    print("\n--- 步骤 2: 注册模型到 Rust 后端 ---")
    
    print("⚠️  [注意] 由于 Rust 端 api.rs 目前仍硬编码了 PromptHash 为 'mock_pool_hash_abc123'，")
    print("   为了跑通本次演示，我们将显式注册这个 Hardcoded Hash，模拟 '合法模型' 的注册过程。")
    print("   (在真实生产环境中，Rust 端将从 Python 请求中动态接收 Hash，并校验是否在白名单中)")
    
    mock_hardcoded_hash = "mock_pool_hash_abc123"
    if not sapt_simulator.register_model(mock_hardcoded_hash):
        print("停止演示：模型注册失败")
        return

    # 3. 模拟一次真实的推理
    print("\n--- 步骤 3: 模拟 AI 推理与存证 ---")
    image_path = "data/samples/original.jpg"
    
    # 获取确定性的路由路径
    activated_indices = sapt_simulator.get_activated_prompts(image_path)
    
    payload = {
        "image_path": image_path,
        "verdict": True, 
        "confidence": 0.88,
        "source": "sapt_simulator_v1"
    }
    
    print(f"🚀 发送存证请求: {payload}")
    
    try:
        resp = requests.post(f"{API_URL}/prove", json=payload)
        
        if resp.status_code != 200:
            print(f"❌ 请求失败: {resp.text}")
            return
            
        receipt = resp.json()
        print(f"📄 收到回执. Root: {receipt['root_hash'][:10]}...")
        
        # 验证
        verify_receipt(receipt, SERVER_PUBKEY_HEX)

    except requests.exceptions.ConnectionError:
        print("❌ 无法连接到服务器，请确保 'cargo run' 正在运行。")

if __name__ == "__main__":
    main()
