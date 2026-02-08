from dataclasses import dataclass
from typing import List
import struct

# BCS (Binary Canonical Serialization) 简易实现
# 对应 Rust 的 bcs crate logic

def serialize_u32(value: int) -> bytes:
    return struct.pack('<I', value)

def serialize_u64(value: int) -> bytes:
    return struct.pack('<Q', value)

def serialize_i64(value: int) -> bytes:
    return struct.pack('<q', value)

def serialize_bool(value: bool) -> bytes:
    return b'\x01' if value else b'\x00'

def serialize_uleb128(value: int) -> bytes:
    """BCS/Protobuf variable-length integer encoding for lengths"""
    out = bytearray()
    while value >= 0x80:
        out.append((value & 0x7f) | 0x80)
        value >>= 7
    out.append(value)
    return bytes(out)

def serialize_string(value: str) -> bytes:
    utf8_bytes = value.encode('utf-8')
    return serialize_uleb128(len(utf8_bytes)) + utf8_bytes

def serialize_vector_u32(values: List[int]) -> bytes:
    out = serialize_uleb128(len(values))
    for v in values:
        out += serialize_u32(v)
    return out

@dataclass
class Evidence:
    image_phash: str
    image_sha256: str
    verdict: bool
    confidence: str
    activated_prompts: List[int]
    prompt_pool_hash: str
    external_knowledge_hash: str
    timestamp: int

    def to_bcs(self) -> bytes:
        """
        按照 Rust 结构体的字段顺序进行 BCS 序列化
        """
        buffer = bytearray()
        buffer += serialize_string(self.image_phash)
        buffer += serialize_string(self.image_sha256)
        buffer += serialize_bool(self.verdict)
        buffer += serialize_string(self.confidence)
        buffer += serialize_vector_u32(self.activated_prompts)
        buffer += serialize_string(self.prompt_pool_hash)
        buffer += serialize_string(self.external_knowledge_hash)
        buffer += serialize_i64(self.timestamp)
        return bytes(buffer)
