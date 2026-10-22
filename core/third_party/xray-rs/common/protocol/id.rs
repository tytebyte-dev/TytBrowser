use md5::{Digest, Md5};
use uuid::Uuid;

pub const ID_BYTES_LEN: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id {
    uuid: Uuid,
    cmd_key: [u8; ID_BYTES_LEN],
}

impl Id {
    pub fn new(uuid: Uuid) -> Self {
        let mut hasher = Md5::new();
        hasher.update(uuid.as_bytes());
        hasher.update(b"c48619fe-8f02-49e0-b9e9-edf763e17e21");
        let result = hasher.finalize();

        let mut cmd_key = [0u8; ID_BYTES_LEN];
        cmd_key.copy_from_slice(&result);

        Self { uuid, cmd_key }
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn bytes(&self) -> &[u8; 16] {
        self.uuid.as_bytes()
    }

    pub fn cmd_key(&self) -> &[u8; ID_BYTES_LEN] {
        &self.cmd_key
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.uuid)
    }
}
