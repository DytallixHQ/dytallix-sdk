use pqcrypto_dilithium::dilithium3;
use pqcrypto_sphincsplus::sphincsshake192ssimple;
use pqcrypto_traits::sign::{
    DetachedSignature as DetachedSignatureTrait, PublicKey as PublicKeyTrait,
    SecretKey as SecretKeyTrait,
};

use crate::error::DytallixError;

const MLDSA65_PUBLIC_KEY_BYTES: usize = 1_952;
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
    /// ML-DSA-65 (FIPS 204 / Dilithium3), the canonical Dytallix signing scheme.
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
        let (public_key, private_key) = dilithium3::keypair();

        Self {
            public_key: public_key.as_bytes().to_vec(),
            private_key: private_key.as_bytes().to_vec(),
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
                let private_key = <dilithium3::SecretKey as SecretKeyTrait>::from_bytes(bytes)
                    .map_err(|err| DytallixError::InvalidKeypair(err.to_string()))?;
                let public_key = recover_mldsa65_public_key(private_key.as_bytes())?;

                Ok(Self {
                    public_key,
                    private_key: private_key.as_bytes().to_vec(),
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
                let private_key =
                    <dilithium3::SecretKey as SecretKeyTrait>::from_bytes(&self.private_key)
                        .map_err(|err| DytallixError::InvalidKeypair(err.to_string()))?;
                let signature = dilithium3::detached_sign(message, &private_key);
                let bytes = signature.as_bytes().to_vec();

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

#[repr(C)]
#[derive(Clone, Copy)]
struct Poly {
    coeffs: [i32; 256],
}

impl Default for Poly {
    fn default() -> Self {
        Self { coeffs: [0; 256] }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Polyvecl {
    vec: [Poly; 5],
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Polyveck {
    vec: [Poly; 6],
}

mod ffi {
    use super::{Polyveck, Polyvecl};

    unsafe extern "C" {
        pub fn PQCLEAN_DILITHIUM3_CLEAN_unpack_sk(
            rho: *mut u8,
            tr: *mut u8,
            key: *mut u8,
            t0: *mut Polyveck,
            s1: *mut Polyvecl,
            s2: *mut Polyveck,
            sk: *const u8,
        );

        pub fn PQCLEAN_DILITHIUM3_CLEAN_pack_pk(pk: *mut u8, rho: *const u8, t1: *const Polyveck);

        pub fn PQCLEAN_DILITHIUM3_CLEAN_polyvec_matrix_expand(mat: *mut Polyvecl, rho: *const u8);

        pub fn PQCLEAN_DILITHIUM3_CLEAN_polyvecl_ntt(v: *mut Polyvecl);
        pub fn PQCLEAN_DILITHIUM3_CLEAN_polyvec_matrix_pointwise_montgomery(
            t: *mut Polyveck,
            mat: *const Polyvecl,
            v: *const Polyvecl,
        );
        pub fn PQCLEAN_DILITHIUM3_CLEAN_polyveck_reduce(v: *mut Polyveck);
        pub fn PQCLEAN_DILITHIUM3_CLEAN_polyveck_invntt_tomont(v: *mut Polyveck);
        pub fn PQCLEAN_DILITHIUM3_CLEAN_polyveck_add(
            w: *mut Polyveck,
            u: *const Polyveck,
            v: *const Polyveck,
        );
        pub fn PQCLEAN_DILITHIUM3_CLEAN_polyveck_caddq(v: *mut Polyveck);
        pub fn PQCLEAN_DILITHIUM3_CLEAN_polyveck_power2round(
            v1: *mut Polyveck,
            v0: *mut Polyveck,
            v: *const Polyveck,
        );
    }
}

fn recover_mldsa65_public_key(private_key: &[u8]) -> Result<Vec<u8>, DytallixError> {
    if private_key.len() != MLDSA65_PRIVATE_KEY_BYTES {
        return Err(DytallixError::InvalidKeySize {
            expected: MLDSA65_PRIVATE_KEY_BYTES,
            got: private_key.len(),
        });
    }

    let mut rho = [0u8; 32];
    let mut tr = [0u8; 64];
    let mut key = [0u8; 32];
    let mut t0 = Polyveck::default();
    let mut s1 = Polyvecl::default();
    let mut s2 = Polyveck::default();
    let mut mat = [Polyvecl::default(); 6];
    let mut t1 = Polyveck::default();
    let mut t0_round = Polyveck::default();
    let mut public_key = vec![0u8; MLDSA65_PUBLIC_KEY_BYTES];

    unsafe {
        ffi::PQCLEAN_DILITHIUM3_CLEAN_unpack_sk(
            rho.as_mut_ptr(),
            tr.as_mut_ptr(),
            key.as_mut_ptr(),
            &mut t0,
            &mut s1,
            &mut s2,
            private_key.as_ptr(),
        );

        let mut s1hat = s1;
        ffi::PQCLEAN_DILITHIUM3_CLEAN_polyvec_matrix_expand(mat.as_mut_ptr(), rho.as_ptr());
        ffi::PQCLEAN_DILITHIUM3_CLEAN_polyvecl_ntt(&mut s1hat);
        ffi::PQCLEAN_DILITHIUM3_CLEAN_polyvec_matrix_pointwise_montgomery(
            &mut t1,
            mat.as_ptr(),
            &s1hat,
        );
        ffi::PQCLEAN_DILITHIUM3_CLEAN_polyveck_reduce(&mut t1);
        ffi::PQCLEAN_DILITHIUM3_CLEAN_polyveck_invntt_tomont(&mut t1);

        let t1_ptr: *mut Polyveck = &mut t1;
        ffi::PQCLEAN_DILITHIUM3_CLEAN_polyveck_add(t1_ptr, t1_ptr as *const Polyveck, &s2);
        ffi::PQCLEAN_DILITHIUM3_CLEAN_polyveck_caddq(t1_ptr);
        ffi::PQCLEAN_DILITHIUM3_CLEAN_polyveck_power2round(
            t1_ptr,
            &mut t0_round,
            t1_ptr as *const Polyveck,
        );
        ffi::PQCLEAN_DILITHIUM3_CLEAN_pack_pk(public_key.as_mut_ptr(), rho.as_ptr(), &t1);
    }

    let public_key = <dilithium3::PublicKey as PublicKeyTrait>::from_bytes(&public_key)
        .map_err(|err| DytallixError::InvalidKeypair(err.to_string()))?;

    Ok(public_key.as_bytes().to_vec())
}
