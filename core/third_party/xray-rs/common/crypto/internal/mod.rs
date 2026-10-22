pub mod chacha;
pub mod chacha_core_gen;
#[path = "chacha_core.generated.rs"]
pub mod chacha_core_generated;

pub use chacha::ChaChaCore;
pub use chacha_core_generated::CHACHA_ROUNDS;
