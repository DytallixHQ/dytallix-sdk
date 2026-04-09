use blake3::Hasher as Blake3Hasher;
use fips204::ml_dsa_65;
use fips204::traits::{KeyGen, SerDes, Signer};
use pqcrypto_sphincsplus::sphincsshake192ssimple;
use pqcrypto_traits::sign::{
    DetachedSignature as DetachedSignatureTrait, PublicKey as PublicKeyTrait,
    SecretKey as SecretKeyTrait,
};

use crate::error::DytallixError;

const MLDSA65_PRIVATE_KEY_BYTES: usize = 4_032;
const MLDSA65_SIGNATURE_BYTES: usize = 3_309;
const SLHDSA_PUBLIC_KEY_BYTES: usize = 48;
const SLHDSA_PRIVATE_KEY_BYTES: usize = 96;
const SLHDSA_SIGNATURE_BYTES: usize = 16_224;

/// Supported signature schemes for Dytallix keypairs.
///
/// `MlDsa65` is the canonical default. `SlhDsa` is provided for cold-storage
/// workflows only.
///
/// # Examples
///
/// ```rust
/// use dytallix_core::keypair::{DytallixKeypair, KeyScheme};
///
/// let keypair = DytallixKeypair::generate();
/// assert_eq!(keypair.scheme(), KeyScheme::MlDsa65);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum KeyScheme {
    /// ML-DSA-65 (FIPS 204), the canonical Dytallix signing scheme.
    MlDsa65,
    /// SLH-DSA-SHAKE-192s, supported for cold storage and explicit opt-in flows.
    SlhDsa,
}

/// In-memory Dytallix keypair.
///
/// The keypair stores raw public and private key bytes plus the active scheme.
///
/// # Examples
///
/// ```rust
/// use dytallix_core::keypair::{DytallixKeypair, KeyScheme};
///
/// let keypair = DytallixKeypair::generate();
/// assert_eq!(keypair.scheme(), KeyScheme::MlDsa65);
/// assert_eq!(keypair.public_key().len(), 1952);
/// ```
pub struct DytallixKeypair {
    public_key: Vec<u8>,
    private_key: Vec<u8>,
    scheme: KeyScheme,
}

impl DytallixKeypair {
    /// Generates a new ML-DSA-65 keypair.
    ///
    /// The returned public key is exactly 1,952 bytes and the private key is
    /// exactly 4,032 bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dytallix_core::keypair::DytallixKeypair;
    ///
    /// let keypair = DytallixKeypair::generate();
    /// assert_eq!(keypair.public_key().len(), 1952);
    /// assert_eq!(keypair.private_key().len(), 4032);
    /// ```
    pub fn generate() -> Self {
        let (public_key, private_key) = ml_dsa_65::KG::try_keygen()
            .expect("ML-DSA-65 key generation should succeed with the OS RNG");

        Self {
            public_key: public_key.into_bytes().to_vec(),
            private_key: private_key.into_bytes().to_vec(),
            scheme: KeyScheme::MlDsa65,
        }
    }

    /// Generates a new SLH-DSA-SHAKE-192s keypair.
    ///
    /// The returned public key is exactly 48 bytes and the private key is
    /// exactly 96 bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dytallix_core::keypair::{DytallixKeypair, KeyScheme};
    ///
    /// let keypair = DytallixKeypair::generate_slh_dsa();
    /// assert_eq!(keypair.scheme(), KeyScheme::SlhDsa);
    /// assert_eq!(keypair.public_key().len(), 48);
    /// assert_eq!(keypair.private_key().len(), 96);
    /// ```
    pub fn generate_slh_dsa() -> Self {
        let (public_key, private_key) = sphincsshake192ssimple::keypair();

        Self {
            public_key: public_key.as_bytes().to_vec(),
            private_key: private_key.as_bytes().to_vec(),
            scheme: KeyScheme::SlhDsa,
        }
    }

    /// Reconstructs a keypair from raw private key bytes.
    ///
    /// Byte lengths are matched against the canonical Dytallix schemes. For
    /// ML-DSA-65, the public key is reconstructed from the packed secret key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dytallix_core::keypair::DytallixKeypair;
    ///
    /// let original = DytallixKeypair::generate();
    /// let restored = DytallixKeypair::from_private_key(original.private_key()).unwrap();
    /// assert_eq!(restored.public_key(), original.public_key());
    /// ```
    pub fn from_private_key(bytes: &[u8]) -> Result<Self, DytallixError> {
        match bytes.len() {
            MLDSA65_PRIVATE_KEY_BYTES => {
                let private_key = ml_dsa_65_private_key_from_bytes(bytes)?;
                let public_key = private_key.get_public_key();

                Ok(Self {
                    public_key: public_key.into_bytes().to_vec(),
                    private_key: private_key.into_bytes().to_vec(),
                    scheme: KeyScheme::MlDsa65,
                })
            }
            SLHDSA_PRIVATE_KEY_BYTES => {
                let private_key =
                    <sphincsshake192ssimple::SecretKey as SecretKeyTrait>::from_bytes(bytes)
                        .map_err(|err| DytallixError::InvalidKeypair(err.to_string()))?;
                let public_key = private_key.as_bytes()
                    [SLHDSA_PRIVATE_KEY_BYTES - SLHDSA_PUBLIC_KEY_BYTES..]
                    .to_vec();
                <sphincsshake192ssimple::PublicKey as PublicKeyTrait>::from_bytes(&public_key)
                    .map_err(|err| DytallixError::InvalidKeypair(err.to_string()))?;

                Ok(Self {
                    public_key,
                    private_key: private_key.as_bytes().to_vec(),
                    scheme: KeyScheme::SlhDsa,
                })
            }
            got => Err(DytallixError::InvalidKeypair(format!(
                "unknown private key length: {got} bytes"
            ))),
        }
    }

    /// Signs a message with the active scheme.
    ///
    /// ML-DSA-65 signatures are deterministic and always 3,309 bytes. SLH-DSA
    /// signatures are always 16,224 bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dytallix_core::keypair::DytallixKeypair;
    ///
    /// let keypair = DytallixKeypair::generate();
    /// let sig = keypair.sign(b"hello dytallix").unwrap();
    /// assert_eq!(sig.len(), 3309);
    /// ```
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, DytallixError> {
        match self.scheme {
            KeyScheme::MlDsa65 => {
                let private_key = ml_dsa_65_private_key_from_bytes(&self.private_key)?;
                let seed = deterministic_mldsa65_seed(&self.private_key, message);
                let signature = private_key
                    .try_sign_with_seed(&seed, message, &[])
                    .map_err(|err| DytallixError::CryptoError(err.to_string()))?;
                let bytes = signature.to_vec();

                if bytes.len() != MLDSA65_SIGNATURE_BYTES {
                    return Err(DytallixError::InvalidSignatureSize {
                        expected: MLDSA65_SIGNATURE_BYTES,
                        got: bytes.len(),
                    });
                }

                Ok(bytes)
            }
            KeyScheme::SlhDsa => {
                let private_key =
                    <sphincsshake192ssimple::SecretKey as SecretKeyTrait>::from_bytes(
                        &self.private_key,
                    )
                    .map_err(|err| DytallixError::InvalidKeypair(err.to_string()))?;
                let signature = sphincsshake192ssimple::detached_sign(message, &private_key);
                let bytes = signature.as_bytes().to_vec();

                if bytes.len() != SLHDSA_SIGNATURE_BYTES {
                    return Err(DytallixError::InvalidSignatureSize {
                        expected: SLHDSA_SIGNATURE_BYTES,
                        got: bytes.len(),
                    });
                }

                Ok(bytes)
            }
        }
    }

    /// Returns the raw public key bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dytallix_core::keypair::DytallixKeypair;
    ///
    /// let keypair = DytallixKeypair::generate();
    /// assert_eq!(keypair.public_key().len(), 1952);
    /// ```
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    /// Returns the raw private key bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dytallix_core::keypair::DytallixKeypair;
    ///
    /// let keypair = DytallixKeypair::generate();
    /// assert_eq!(keypair.private_key().len(), 4032);
    /// ```
    pub fn private_key(&self) -> &[u8] {
        &self.private_key
    }

    /// Returns the active signing scheme.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dytallix_core::keypair::{DytallixKeypair, KeyScheme};
    ///
    /// let keypair = DytallixKeypair::generate();
    /// assert_eq!(keypair.scheme(), KeyScheme::MlDsa65);
    /// ```
    pub fn scheme(&self) -> KeyScheme {
        self.scheme
    }
}

fn ml_dsa_65_private_key_from_bytes(bytes: &[u8]) -> Result<ml_dsa_65::PrivateKey, DytallixError> {
    let private_key = bytes
        .try_into()
        .map_err(|_| DytallixError::InvalidKeySize {
            expected: MLDSA65_PRIVATE_KEY_BYTES,
            got: bytes.len(),
        })?;
    ml_dsa_65::PrivateKey::try_from_bytes(private_key)
        .map_err(|err| DytallixError::InvalidKeypair(err.to_string()))
}

fn deterministic_mldsa65_seed(private_key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut hasher = Blake3Hasher::new();
    hasher.update(b"dytallix-ml-dsa-65-seed");
    hasher.update(private_key);
    hasher.update(message);
    *hasher.finalize().as_bytes()
}
