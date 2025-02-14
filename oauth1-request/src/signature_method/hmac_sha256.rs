//! The `HMAC-SHA1` signature method ([RFC 5849 section 3.4.2.][rfc]).
//!
//! [rfc]: https://tools.ietf.org/html/rfc5849#section-3.4.2
//!
//! This module is only available when `hmac-sha1` feature is activated.

use core::fmt::{self, Debug, Display, Formatter, Write};

use digest::core_api::BlockSizeUser;
use digest::generic_array::sequence::GenericSequence;
use digest::generic_array::GenericArray;
use digest::{OutputSizeUser, Update};
use hmac_sha256::Hash;

use super::digest_common::{Base64PercentEncodeDisplay, UpdateSign};
use super::{write_signing_key, Sign, SignatureMethod};

/// The `HMAC-SHA1` signature method.
#[derive(Clone, Copy, Default)]
pub struct HmacSha256 {
    _priv: (),
}

#[derive(Clone)]
struct Hasher256(Hash);

impl Update for Hasher256 {
    fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }
}

impl Debug for Hasher256 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Hasher256").finish()
    }
}

/// A type that signs a signature base string with the HMAC-SHA1 signature algorithm.
#[derive(Clone)]
pub struct HmacSha256Sign {
    inner: UpdateSign<Hasher256>,
}

impl Debug for HmacSha256Sign {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("HmacSha256Sign").finish()
    }
}

type Hash256 = [u8; 32];

/// A signature produced by an `HmacSha1Sign`.
pub struct HmacSha256Signature {
    inner: Base64PercentEncodeDisplay<[u8; 32]>,
}

/// The `HMAC-SHA1` signature method with a default configuration.
pub const HMAC_SHA1: HmacSha256 = HmacSha256::new();

#[derive(Clone)]
struct SigningKey(Hasher256);

impl HmacSha256 {
    /// Creates a new `HmacSha1`.
    pub const fn new() -> Self {
        HmacSha256 { _priv: () }
    }
}

impl Debug for HmacSha256 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        #[derive(Debug)]
        struct HmacSha256;
        HmacSha256.fmt(f)
    }
}

impl SignatureMethod for HmacSha256 {
    type Sign = HmacSha256Sign;

    fn sign_with(self, client_secret: &str, token_secret: Option<&str>) -> HmacSha256Sign {
        let mut key = SigningKey::new();
        write_signing_key(&mut key, client_secret, token_secret).unwrap();
        HmacSha256Sign {
            inner: UpdateSign(key.into_hmac()),
        }
    }
}

impl Sign for HmacSha256Sign {
    type Signature = HmacSha256Signature;

    fn get_signature_method_name(&self) -> &'static str {
        "HMAC-SHA256"
    }

    fn request_method(&mut self, method: &str) {
        self.inner.request_method(method);
    }

    fn uri<T: Display>(&mut self, uri: T) {
        self.inner.uri(uri);
    }

    fn parameter<V: Display>(&mut self, key: &str, value: V) {
        self.inner.parameter(key, value);
    }

    fn delimiter(&mut self) {
        self.inner.delimiter();
    }

    fn end(self) -> HmacSha256Signature {
        HmacSha256Signature {
            inner: Base64PercentEncodeDisplay(self.inner.0 .0.finalize()),
        }
    }
}

impl Display for HmacSha256Signature {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl SigningKey {
    fn new() -> Self {
        SigningKey(Hasher256(Hash::new()))
    }

    fn write(&mut self, input: &[u8]) {
        self.0.update(input);
    }

    fn into_hmac(self) -> Hasher256 {
        self.0
    }
}

impl Write for SigningKey {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::vec::Vec;

    use digest::generic_array::typenum::Unsigned;

    use super::*;

    #[test]
    fn signing_key() {
        let mut sk = SigningKey::new();
        //let mut k = Vec::new();
        /*
        for _ in 0..=<Sha1 as BlockSizeUser>::BlockSize::to_usize() + 1 {
            sk.write(&[1]);
            k.extend(&[1]);

            let mut skm = sk.clone().into_hmac();
            let mut m = Hmac::<Sha1>::new_from_slice(&k).unwrap();
            skm.update(b"test");
            m.update(b"test");

            assert_eq!(skm.finalize().into_bytes(), m.finalize().into_bytes());
        }*/
    }
}
