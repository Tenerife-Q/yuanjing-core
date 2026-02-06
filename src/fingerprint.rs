use img_hash::{HasherConfig, HashAlg}; // 引入 pHash 相关的配置器和算法枚举
use sha2::{Sha256, Digest};            // 引入 SHA2 算法和 Digest 特性(方法集)
use std::fs;                           // 文件系统操作
use std::path::Path;                   // 路径处理

// -> anyhow::Result<(String, String)>
// 这是一个返回 Result 的函数。
// 成功时：返回一个元组 (String, String)，分别对应 (SHA256, pHash)。
// 失败时：利用 anyhow 库抛出错误（比如文件找不到）。
pub fn generate_fingerprints(path: &Path) -> anyhow::Result<(String, String)> {
    // 1. fs::read(path)?
    // 作用：把整个文件读入内存，变成 Vec<u8> (字节数组)。
    // 语法细节 `?`: 如果读文件失败（文件不存在/无权限），直接在这里 return Err，不再往下走。
    let bytes = fs::read(path)?;

    // 2. Sha256::new()
    // 作用：创建一个哈希计算器的“状态机”实例。
    let mut hasher = Sha256::new();
    
    // 3. hasher.update(&bytes)
    // 作用：像喂碎纸机一样，把数据喂给哈希器。
    // 语法细节 `&`: 传入数据的引用，不发生所有权转移（虽然这里bytes之后也没用了）。
    hasher.update(&bytes);
    
    // 4. hasher.finalize()
    // 作用：按下“结束”按钮，计算出最终的 32 字节哈希值 (GenericArray)。
    // 5. format!("{:x}", ...)
    // 作用：`{:x}` 是格式化占位符，表示将二进制数据转为 "小写十六进制字符串" (Lower Hex)。
    let sha_hash = format!("{:x}", hasher.finalize());

    // 1. img_hash::image::open(path)?
    // 作用：这不是读字节，而是“解码图片”。
    // 它会解析 JPG/PNG 头部，把像素数据解压出来放到内存里的 ImageBuffer 中。
    // 如果文件是不是图片格式，这里会报错。
    let img = img_hash::image::open(path)?;

    // 2. HasherConfig::new()...to_hasher()
    // 作用：配置我们要用什么样的算法算 pHash。
    let hasher = HasherConfig::new()
        .hash_alg(HashAlg::Gradient) // 选择“梯度算法”。比起均值算法(Mean)，梯度对明暗变化更鲁棒。
        .hash_size(8, 8)             // 输出 8x8 = 64位 的指纹。
        .to_hasher();                // 完成配置，构建 Hasher 对象。
    
    // 3. hasher.hash_image(&img)
    // 作用：执行核心算法。
    // 过程：缩小图片 -> 灰度化 -> 计算梯度 -> 生成哈希对象。
    let phash = hasher.hash_image(&img);

    // 4. phash.to_base64()
    // 作用：pHash 结果本质是一串二进制位 (010101...)。
    // 为了存得短一点，常用 Base64 编码转成字符串。
    Ok((sha_hash, phash.to_base64()))
}
