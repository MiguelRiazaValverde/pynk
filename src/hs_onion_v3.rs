use data_encoding::BASE32_NOPAD;
use ed25519_dalek::SigningKey;
use napi::{bindgen_prelude::*, tokio};
use rand_core::OsRng;
use sha3::{Digest, Sha3_256};

const CHECKSUM_PREFIX: &[u8] = b".onion checksum";
const VERSION: u8 = 0x03;

#[napi(js_name = "OnionV3")]
#[derive(Default)]
pub struct NativeOnionV3 {
  secret: [u8; 32],
  public: [u8; 32],
  pub address: String,
  steps_to_gen: u32,
}

#[napi]
impl NativeOnionV3 {
  /**
   * Creates a new Onion v3 address with a randomly generated private key.
   */
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let public = signing_key.verifying_key().to_bytes();
    let address = Self::compute_onion_address(&public);
    Ok(Self {
      secret: signing_key.to_keypair_bytes()[..32].try_into().unwrap(),
      public,
      address,
      steps_to_gen: 1,
    })
  }

  /**
   * Generates a vanity Onion v3 address synchronously that starts with the given prefix.
   * Loops until a matching address is found, counting the number of attempts in `steps`.
   */
  #[napi]
  pub fn generate_vanity(prefix: String) -> Result<Self> {
    let mut csprng = OsRng;
    let mut steps = 0;
    loop {
      steps += 1;
      let signing_key = SigningKey::generate(&mut csprng);
      let public = signing_key.verifying_key().to_bytes();
      let addr = Self::compute_onion_address(&public);
      if addr.starts_with(&prefix) {
        return Ok(Self {
          secret: signing_key.to_keypair_bytes()[..32].try_into().unwrap(),
          public,
          address: addr,
          steps_to_gen: steps,
        });
      }
    }
  }

  /**
   * Creates a new Onion v3 address with a randomly generated secret key.
   */
  #[napi]
  pub fn create() -> Result<Self> {
    Self::new()
  }

  /**
   * Asynchronously generates a vanity Onion v3 address with the specified prefix.
   * Yields execution every `stopEach` attempts to avoid blocking the async runtime.
   */
  #[napi]
  pub async fn generate_vanity_async(prefix: String, stop_each: Option<u32>) -> Result<Self> {
    let mut csprng = OsRng;
    let mut steps = 0;
    let stop_each = stop_each.unwrap_or(1000).max(1);
    loop {
      steps += 1;
      let signing_key = SigningKey::generate(&mut csprng);
      let public = signing_key.verifying_key().to_bytes();
      let addr = Self::compute_onion_address(&public);
      if addr.starts_with(&prefix) {
        return Ok(Self {
          secret: signing_key.to_keypair_bytes()[..32].try_into().unwrap(),
          public,
          address: addr,
          steps_to_gen: steps,
        });
      } else if steps % stop_each == 0 {
        tokio::task::yield_now().await;
      }
    }
  }

  /**
   * Creates an Onion v3 instance from a 32-byte secret key buffer.
   * Returns an error if the buffer length is invalid.
   */
  #[napi]
  pub fn from_secret(private_key: Buffer) -> Result<Self> {
    if private_key.len() != 32 {
      return Err(Error::from_reason("Expected a 32-byte private key"));
    }

    let secret: [u8; 32] = private_key
      .as_ref()
      .try_into()
      .map_err(|_| Error::from_reason("Invalid private key length"))?;

    let signing_key = SigningKey::from_bytes(&secret);
    let public = signing_key.verifying_key().to_bytes();
    let address = Self::compute_onion_address(&public);

    Ok(Self {
      secret,
      public,
      address,
      steps_to_gen: 0,
    })
  }

  /**
   * Returns the secret key as a Buffer.
   */
  #[napi]
  pub fn get_secret(&self) -> Buffer {
    Buffer::from(self.secret.to_vec())
  }

  /**
   * Returns the public key as a Buffer.
   */
  #[napi]
  pub fn get_public(&self) -> Buffer {
    Buffer::from(self.public.to_vec())
  }

  /**
   * Number of steps taken during vanity address generation.
   */
  #[napi(getter)]
  pub fn steps(&self) -> u32 {
    self.steps_to_gen
  }

  fn compute_onion_address(public: &[u8; 32]) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(CHECKSUM_PREFIX);
    hasher.update(public);
    hasher.update([VERSION]);
    let full = hasher.finalize();
    let checksum = &full[..2];

    let mut payload = Vec::with_capacity(35);
    payload.extend_from_slice(public);
    payload.extend_from_slice(checksum);
    payload.push(VERSION);

    let b32 = BASE32_NOPAD.encode(&payload).to_lowercase();
    format!("{}.onion", b32)
  }
}
