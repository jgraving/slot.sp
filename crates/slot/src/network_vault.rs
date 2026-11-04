//! Credentials are encrypted under a user-held passphrase.
//! The key is deliberately never persisted or derived from a public device identifier.
use ring::{
    aead,
    rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use zeroize::{Zeroize, Zeroizing};

const MAGIC: &[u8; 8] = b"SLOTNET1";
const BUTTON_MAGIC: &[u8; 8] = b"SLOTBTN1";
const MAX_FILE: usize = 32 * 1024;
pub const MIN_CODE_LEN: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method { Buttons, Passphrase }
impl Method {
    fn magic(self) -> &'static [u8; 8] {
        match self { Self::Buttons => BUTTON_MAGIC, Self::Passphrase => MAGIC }
    }
    fn from_magic(bytes: &[u8]) -> Option<Self> {
        if bytes == BUTTON_MAGIC { Some(Self::Buttons) }
        else if bytes == MAGIC { Some(Self::Passphrase) } else { None }
    }
    fn validate(self, secret: &str) -> Result<(), &'static str> {
        match self {
            Self::Buttons if secret.len() >= MIN_CODE_LEN && secret.bytes().all(|b| (b'a'..=b'l').contains(&b)) => Ok(()),
            Self::Buttons => Err("Use at least 6 button presses"),
            Self::Passphrase if secret.chars().count() >= 12 => Ok(()),
            Self::Passphrase => Err("Use at least 12 characters"),
        }
    }
}
/// Only the input method is public; no code, hint, hash or recovery key is stored.
pub fn method(root: &Path) -> Option<Method> {
    use std::io::Read;
    let mut bytes = [0; 8];
    std::fs::File::open(vault_path(root)).ok()?.read_exact(&mut bytes).ok()?;
    Method::from_magic(&bytes)
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Secrets {
    #[serde(default)] pub sharing_password: String,
    #[serde(default)] pub sharing_network: String,
    pub ssid: String,
    pub wifi_password: String,
    pub username: String,
    pub password: String,
    pub token: String,
}
impl Drop for Secrets {
    fn drop(&mut self) {
        self.ssid.zeroize();
        self.wifi_password.zeroize();
        self.username.zeroize();
        self.password.zeroize();
        self.token.zeroize();
        self.sharing_password.zeroize();self.sharing_network.zeroize();
    }
}
struct Key {
    bytes: Zeroizing<[u8; 32]>,
    salt: [u8; 16],
    method: Method,
}
fn derive_for(pass: &str, salt: [u8; 16], method: Method) -> Result<Key, &'static str> {
    let mut bytes = Zeroizing::new([0; 32]);
    // 19 MiB, two passes; executed on a worker, never the render thread.
    let params =
        argon2::Params::new(19 * 1024, 2, 1, Some(32)).map_err(|_| "Cannot unlock credentials")?;
    // Argon2's allocating API does not clear its working blocks on release,
    // even with its zeroize feature. Own and wipe that key-derived memory too.
    let mut memory = Zeroizing::new(vec![argon2::Block::default(); params.block_count()]);
    argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
        .hash_password_into_with_memory(pass.as_bytes(), &salt, bytes.as_mut(), memory.as_mut_slice())
        .map_err(|_| "Cannot unlock credentials")?;
    Ok(Key { bytes, salt, method })
}
fn encrypt(path: &Path, key: &Key, secrets: &Secrets) -> Result<(), &'static str> {
    let mut payload =
        Zeroizing::new(serde_json::to_vec(secrets).map_err(|_| "Cannot save credentials")?);
    let mut nonce = [0; 12];
    SystemRandom::new()
        .fill(&mut nonce)
        .map_err(|_| "Cannot save credentials")?;
    let key_aead = aead::LessSafeKey::new(
        aead::UnboundKey::new(&aead::CHACHA20_POLY1305, key.bytes.as_ref())
            .map_err(|_| "Cannot save credentials")?,
    );
    let mut header = Vec::from(key.method.magic().as_slice());
    header.extend_from_slice(&key.salt);
    header.extend_from_slice(&nonce);
    key_aead
        .seal_in_place_append_tag(
            aead::Nonce::assume_unique_for_key(nonce),
            aead::Aad::from(&header),
            &mut *payload,
        )
        .map_err(|_| "Cannot save credentials")?;
    header.extend_from_slice(&payload);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| "Cannot save credentials")?;
    }
    slot_store::atomic_write(path, &header).map_err(|_| "Cannot save credentials")
}
fn decrypt(path: &Path, pass: &str) -> Result<(Key, Secrets), &'static str> {
    let file = std::fs::File::open(path).map_err(|_| "Cannot read encrypted credentials")?;
    use std::io::Read;
    let mut blob = Zeroizing::new(Vec::new());
    file.take((MAX_FILE + 1) as u64)
        .read_to_end(&mut blob)
        .map_err(|_| "Cannot read encrypted credentials")?;
    if blob.len() < 52 || blob.len() > MAX_FILE {
        return Err("Invalid encrypted credentials");
    }
    let method = Method::from_magic(&blob[..8]).ok_or("Invalid encrypted credentials")?;
    let salt = blob[8..24].try_into().unwrap();
    let nonce = blob[24..36].try_into().unwrap();
    let key = derive_for(pass, salt, method)?;
    let key_aead = aead::LessSafeKey::new(
        aead::UnboundKey::new(&aead::CHACHA20_POLY1305, key.bytes.as_ref())
            .map_err(|_| "Cannot unlock credentials")?,
    );
    let (header, payload) = blob.split_at_mut(36);
    let plain = key_aead
        .open_in_place(
            aead::Nonce::assume_unique_for_key(nonce),
            aead::Aad::from(header),
            payload,
        )
        .map_err(|_| "Wrong unlock code or damaged credentials")?;
    let secrets = serde_json::from_slice(plain).map_err(|_| "Invalid encrypted credentials")?;
    Ok((key, secrets))
}
pub enum Job {
    Create(Zeroizing<String>, Secrets, Method),
    Unlock(Zeroizing<String>),
    Save(Secrets),
    Forget,
}
pub enum Response { Unlocked(Secrets), Created(Method), Saved, Forgotten }
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Operation { Unlock, Create, Save, Forget }
pub struct Reply { pub operation: Operation, pub result: Result<Response, &'static str> }
pub struct Worker {
    pub jobs: Sender<Job>,
    pub results: Receiver<Reply>,
}
impl Worker {
    pub fn new(root: &Path) -> Self {
        let path = vault_path(root);
        let (jobs, requests) = mpsc::channel();
        let (responses, results) = mpsc::channel();
        std::thread::spawn(move || {
            let mut key = None;
            for job in requests {
                let operation = match &job {
                    Job::Unlock(_) => Operation::Unlock, Job::Create(..) => Operation::Create,
                    Job::Save(_) => Operation::Save, Job::Forget => Operation::Forget,
                };
                let result = match job {
                    Job::Unlock(pass) => decrypt(&path, &pass).map(|(next, secrets)| {
                        key = Some(next);
                        Response::Unlocked(secrets)
                    }),
                    Job::Create(pass, secrets, method) => (|| {
                        method.validate(&pass)?;
                        let mut salt = [0; 16];
                        SystemRandom::new()
                            .fill(&mut salt)
                            .map_err(|_| "Cannot save credentials")?;
                        let next = derive_for(&pass, salt, method)?;
                        encrypt(&path, &next, &secrets)?;
                        key = Some(next);
                        Ok(Response::Created(method))
                    })(),
                    Job::Save(secrets) => match &key {
                        Some(key) => encrypt(&path, key, &secrets).map(|_| Response::Saved),
                        None => Err("Unlock credentials first"),
                    },
                    Job::Forget => match std::fs::remove_file(&path) {
                        Ok(()) => {
                            key = None;
                            Ok(Response::Forgotten)
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                            key = None;
                            Ok(Response::Forgotten)
                        }
                        Err(_) => Err("Cannot remove saved credentials"),
                    },
                };
                if responses.send(Reply { operation, result }).is_err() {
                    break;
                }
            }
        });
        Self { jobs, results }
    }
}
pub fn vault_path(root: &Path) -> PathBuf {
    root.join("Config/network.vault")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cleared_working_memory_preserves_the_existing_key_derivation() {
        let salt=[7;16];let pass="existing vault passphrase";
        let mut legacy=Zeroizing::new([0;32]);
        argon2::Argon2::new(argon2::Algorithm::Argon2id,argon2::Version::V0x13,
            argon2::Params::new(19*1024,2,1,Some(32)).unwrap())
            .hash_password_into(pass.as_bytes(),&salt,legacy.as_mut()).unwrap();
        assert_eq!(derive_for(pass,salt,Method::Passphrase).unwrap().bytes.as_ref(),legacy.as_ref());
    }
    #[test]
    fn saves_use_fresh_nonces_and_authenticate_salt_and_nonce() {
        let dir=tempfile::tempdir().unwrap();let path=dir.path().join("vault");
        let pass="an independent passphrase";let key=derive_for(pass,[8;16],Method::Passphrase).unwrap();
        let mut secrets=Secrets::default();secrets.token="fixture-token".into();
        encrypt(&path,&key,&secrets).unwrap();let first=std::fs::read(&path).unwrap();
        encrypt(&path,&key,&secrets).unwrap();let second=std::fs::read(&path).unwrap();
        assert_ne!(&first[24..36],&second[24..36]);
        assert_eq!(decrypt(&path,pass).unwrap().1.token,"fixture-token");
        for index in [8,24] {
            let mut changed=second.clone();changed[index]^=1;std::fs::write(&path,changed).unwrap();
            assert!(decrypt(&path,pass).is_err());
        }
    }
    #[test]
    fn encrypted_credentials_require_the_passphrase_and_detect_tampering() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault");
        let mut secrets = Secrets::default();
        secrets.wifi_password = "not-in-plaintext".into();
        secrets.token = "account-secret".into();
        let key = derive_for("a long test passphrase", [1; 16], Method::Passphrase).unwrap();
        encrypt(&path, &key, &secrets).unwrap();
        let mut bytes = std::fs::read(&path).unwrap();
        assert!(!bytes.windows(16).any(|b| b == b"not-in-plaintext"));
        assert!(!bytes.windows(14).any(|b| b == b"account-secret"));
        assert!(decrypt(&path, "incorrect password").is_err());
        assert_eq!(
            decrypt(&path, "a long test passphrase").unwrap().1.token,
            "account-secret"
        );
        let n = bytes.len();
        bytes[n - 1] ^= 1;
        std::fs::write(&path, bytes).unwrap();
        assert!(decrypt(&path, "a long test passphrase").is_err());
    }
    #[test]
    fn method_header_is_authenticated_and_legacy_passphrase_files_still_unlock() {
        let root=tempfile::tempdir().unwrap();let path=vault_path(root.path());
        let code="acgjdi";
        Method::Buttons.validate(code).unwrap();assert!(Method::Buttons.validate("abc").is_err());
        assert!(Method::Buttons.validate("abcde").is_err());
        Method::Buttons.validate(&"abcdefghijkl".repeat(32)).unwrap();
        assert!(Method::Buttons.validate("abcz").is_err());
        let mut secrets=Secrets::default();secrets.wifi_password="network-password".into();
        let key=derive_for(code,[2;16],Method::Buttons).unwrap();encrypt(&path,&key,&secrets).unwrap();
        assert_eq!(method(root.path()),Some(Method::Buttons));
        assert_eq!(decrypt(&path,code).unwrap().1.wifi_password,"network-password");
        let mut bytes=std::fs::read(&path).unwrap();assert!(!bytes.windows(code.len()).any(|v|v==code.as_bytes()));
        bytes[..8].copy_from_slice(MAGIC);std::fs::write(&path,bytes).unwrap();assert!(decrypt(&path,code).is_err());
        let legacy=derive_for("a long test passphrase",[3;16],Method::Passphrase).unwrap();encrypt(&path,&legacy,&secrets).unwrap();
        assert_eq!(method(root.path()),Some(Method::Passphrase));assert!(decrypt(&path,"a long test passphrase").is_ok());
    }

}

#[cfg(test)]
pub(crate) fn test_legacy(root:&Path,code:&str,secrets:Secrets) {
    let key=derive_for(code,[2;16],Method::Buttons).unwrap();encrypt(&vault_path(root),&key,&secrets).unwrap();
}
#[cfg(test)]
pub(crate) fn test_decrypt(root:&Path,code:&str)->(Method,Secrets) {
    let (key,secrets)=decrypt(&vault_path(root),code).unwrap();(key.method,secrets)
}
