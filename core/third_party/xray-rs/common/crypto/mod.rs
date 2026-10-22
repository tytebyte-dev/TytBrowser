pub mod aes;
pub mod auth;
pub mod chacha20;
pub mod chunk;
pub mod crypto;
pub mod internal;
pub mod io;

#[cfg(test)]
pub mod aes_test;
#[cfg(test)]
pub mod auth_test;
#[cfg(test)]
pub mod benchmark_test;
#[cfg(test)]
pub mod chacha20_test;
#[cfg(test)]
pub mod chunk_test;
#[cfg(test)]
pub mod io_test;

pub use aes::{AesGcmCipher, new_aes_gcm};
pub use auth::Authentication;
pub use chacha20::ChaCha20Cipher;
pub use chunk::{AeadChaCha20ChunkReader, AeadChaCha20ChunkWriter, IncreasingNonce, PlainChunk};
pub use crypto::{StreamCipher, rand_between, rand_bytes_between};
pub use internal::chacha::ChaChaCore;
pub use io::{CryptionReader, CryptionWriter, xor_buffers};
