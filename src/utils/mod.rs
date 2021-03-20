mod byte_buffer_reader;

use {ahash::AHasher, std::hash::Hasher};

pub use byte_buffer_reader::PanickingByteBufferReader;

pub fn hash_string(string: &str) -> u64 {
    let mut hasher = AHasher::default();
    hasher.write(string.as_bytes());
    hasher.finish()
}
