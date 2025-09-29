//! 编解码抽象
//!
//! 目标：
//! - 为二进制编解码提供 trait，支持按需替换（如 bincode、prost、Cap’n Proto）。
//! - 明确解码失败语义：`Option<T>` 用于信号化不可恢复的格式错误。
//!
//! 兼容性与演进（草图）：
//! - 引入版本前缀或模式以实现向后兼容；避免破坏性变更。
//! - 对于日志/快照等持久化格式，应记录 schema 版本与校验和。
pub trait BinaryCodec<T> {
    fn encode(&self, value: &T) -> Vec<u8>;
    fn decode(&self, bytes: &[u8]) -> Option<T>;
}

/// 直接透传 `Vec<u8>` 的编解码器
#[derive(Debug, Default, Clone, Copy)]
pub struct BytesCodec;

impl BinaryCodec<Vec<u8>> for BytesCodec {
    fn encode(&self, value: &Vec<u8>) -> Vec<u8> {
        value.clone()
    }
    fn decode(&self, bytes: &[u8]) -> Option<Vec<u8>> {
        Some(bytes.to_vec())
    }
}

/// 使用 UTF-8 的 `String` 编解码器
#[derive(Debug, Default, Clone, Copy)]
pub struct StringUtf8Codec;

impl BinaryCodec<String> for StringUtf8Codec {
    fn encode(&self, value: &String) -> Vec<u8> {
        value.as_bytes().to_vec()
    }
    fn decode(&self, bytes: &[u8]) -> Option<String> {
        std::str::from_utf8(bytes).ok().map(|s| s.to_string())
    }
}
