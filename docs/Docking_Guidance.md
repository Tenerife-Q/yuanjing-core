

---

### 🔎 代码层面细节复盘（确认没有遗漏）

在正式交接前，我们从代码逻辑上最后盘点一下现在的对接现状：

1. **信心值（Confidence）的安全边界**：
   - Rust 端严格要求 `0.0 <= confidence <= 1.0`，如果是 NaN 或者越界会报 500 错。
   - 我们的 Python 端 `blockchain_bridge.py` 中的 `normalize_confidence` 函数已经做了兜底：`max(0.0, min(float(confidence), 1.0))`，所以 AI 队友传什么诡异的浮点数过来都不会崩。**（细节满分 ✅）**
2. **标签映射（Label -> Verdict）**：
   - AI 模型预测出的是 `0` 或 `1`，但 Rust 链端需要的是布尔值 `true / false`。
   - `blockchain_bridge.py` 中的 `DATASET_LABEL_TO_VERDICT` 已经处理了这个逻辑。队友只需要无脑传入模型的原始 label，桥接层会自动转换（例如 weibo 的 0 转为 False，1 转为 True）。**（逻辑解耦 ✅）**
3. **⚠️ 唯一潜在隐患：图片路径（Image Path）**：
   - 刚才我们遇到了 500 错误，是因为 Rust 后端去读图片时找不到文件。
   - **注意：** 两个程序跑在不同的文件夹下，传给 `PredictionPayload` 的 `image_path` 必须是**绝对路径**（Absolute Path），或者队友运行 Rust 服务和运行 Python 脚本必须保证相对路径能够解析。我会在下面的指导书里特意强调这一点。

---

### 📄 移交队友的指导文书（可直接复制发送）

你可以直接把下面的内容复制发给你的 AI 队友，里面包含了他们需要知道的所有信息和接入代码：

```markdown
# MMFN 模型链端存证接入指南

嗨，模型侧的联调兄弟/姐妹：

区块链存证底座（yuanjing-core）和 Python 侧的通信桥接层（blockchain_bridge.py）已经全部调试打通了。现在只需要将这段代码嵌入到真实的 MMFN 模型推理循环中即可。

## ⚙️ 前置准备（启动测试环境）

在跑模型推理前，确保存证服务已经启动并完成了模型注册（必须执行）：

1. **启动区块链核心（单独开一个终端）**
   ```bash
   cd yuanjing-core
   cargo run
   ```
2. **注册当前测试的模型（新开终端，执行一次即可）**
   ```bash
   curl --noproxy "*" -X POST http://localhost:3000/model/register \
     -H "Content-Type: application/json" \
     -d '{"hash":"0000000000000000000000000000000000000000000000000000000000000000", "description":"mmfn real model test"}'
   ```

## 💻 Python 模型代码接入示例

在真实的 `main.py` 或 `test.py` 的**推理结果输出部分**，引入桥接模块并发送存证。

**示例代码：**
```python
import os
from pathlib import Path
from blockchain_bridge import PredictionPayload, submit_proof_with_retry

# 设置环境变量，指向链端地址
os.environ["YUANJING_BASE_URL"] = "http://localhost:3000"

def run_inference_and_prove():
    # 1. 你的模型做推理...
    dataset_name = "weibo"
    image_rel_path = "./data/weibo/images/12345.jpg" 
    predicted_label = 1    # 模型吐出的 0 或 1
    confidence_score = 0.982 # 模型的 softmax/sigmoid 概率
    
    # ⚠️ 关键细节：由于 Rust 进程和 Python 进程可能不在同一个目录
    # 必须将图片路径转换为「绝对路径」，否则 Rust 端会报 500 找不到文件
    abs_image_path = str(Path(image_rel_path).resolve())

    # 2. 组装存证 Payload
    payload = PredictionPayload(
        dataset=dataset_name,
        image_path=abs_image_path, 
        predicted_label=predicted_label,
        confidence=confidence_score,
        source="mmfn_real_inference",
        # 如果还没生成真实模型hash，先用64个0占位，必须和上面curl注册的一致
        prompt_pool_hash="0" * 64 
    )

    # 3. 提交到区块链
    try:
        print(f"正在为图片 {abs_image_path} 提交存证...")
        receipt = submit_proof_with_retry(payload, max_retries=3)
        print(f"✅ 上链成功！证据位置 (pos): {receipt.get('leaf_pos')}")
    except Exception as e:
        print(f"❌ 存证失败: {e}")

# 放在你的测试循环里
run_inference_and_prove()
```

## 🛠️ 常见问题排查 (Troubleshooting)
- **500 Server Error**: 99% 是因为传给 Payload 的 `image_path` 不是绝对路径，Rust 后端无法在磁盘上找到这张图以计算特征。
- **Connection timed out**: 检查终端是否开启了 VPN/代理，导致 `localhost` 被劫持。可在终端临时执行 `unset http_proxy https_proxy all_proxy` 解决。
```

### 收尾建议
你的对接任务非常漂亮。接下来，你只需要在你的 `multi-modal-fake-news-detection...` 仓库里，**把那个名为 "Fix yuanjing-core integration..." 的 PR #5 点击 Merge 合并到 main 分支**，你的任务就彻底完美收官了！