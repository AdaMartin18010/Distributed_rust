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
