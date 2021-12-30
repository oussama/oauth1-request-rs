//! The `PLAINTEXT` signature method ([RFC 5849 section 3.4.4.][rfc]).
//!
//! [rfc]: https://tools.ietf.org/html/rfc5849#section-3.4.4

use core::fmt::{self, Debug, Display, Formatter, Write};
use core::marker::PhantomData;

use super::{write_signing_key, Sign, SignatureMethod};

/// The `PLAINTEXT` signature method.
#[derive(Copy)]
pub struct Plaintext<
    #[cfg(feature = "alloc")] W = alloc::string::String,
    #[cfg(not(feature = "alloc"))] W,
> {
    marker: PhantomData<fn() -> W>,
}

/// A `Sign` implementation that just returns the signing key used to construct it.
#[derive(Clone, Debug)]
pub struct PlaintextSign<
    #[cfg(feature = "alloc")] W = alloc::string::String,
    #[cfg(not(feature = "alloc"))] W,
> {
    signing_key: W,
}

#[cfg(feature = "alloc")]
impl Plaintext {
    /// Creates a new `Plaintext`.
    pub fn new() -> Self {
        Plaintext {
            marker: PhantomData,
        }
    }
}

impl<W> Plaintext<W>
where
    W: Default + Display + Write,
{
    // We separate constructors for the case of `W = String` and the generic case because
    // `Plaintext::new_with_buf` would result in a type inference error due to current limitation of
    // defaulted type parameters. This would be fixed if default type parameter fallback landed.
    // <https://github.com/rust-lang/rust/issues/27336>

    /// Creates a new `Plaintext` that writes the resulting signatures into `W` values.
    pub fn with_buf() -> Self {
        Plaintext {
            marker: PhantomData,
        }
    }
}

impl<W> Clone for Plaintext<W> {
    fn clone(&self) -> Self {
        Plaintext {
            marker: PhantomData,
        }
    }
}

impl<W> Debug for Plaintext<W> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        #[derive(Debug)]
        struct Plaintext;
        Plaintext.fmt(f)
    }
}

impl<W> Default for Plaintext<W>
where
    W: Default + Display + Write,
{
    fn default() -> Self {
        Self::with_buf()
    }
}

impl<W> SignatureMethod for Plaintext<W>
where
    W: Default + Display + Write,
{
    type Sign = PlaintextSign<W>;

    fn sign_with(self, client_secret: &str, token_secret: Option<&str>) -> Self::Sign {
        let mut signing_key = W::default();
        write_signing_key(&mut signing_key, client_secret, token_secret).unwrap();
        PlaintextSign { signing_key }
    }
}

impl<W> Sign for PlaintextSign<W>
where
    W: Display + Write,
{
    type Signature = W;

    fn get_signature_method_name(&self) -> &'static str {
        "PLAINTEXT"
    }

    fn request_method(&mut self, _method: &str) {}

    fn uri<T>(&mut self, _uri: T) {}

    fn parameter<V>(&mut self, _key: &str, _value: V) {}

    fn delimiter(&mut self) {}

    fn end(self) -> W {
        self.signing_key
    }

    // The OAuth standard (section 3.1.) says that `oauth_timestamp` and `oauth_nonce` parameters
    // MAY be omitted when using the `PLAINTEXT` signature method. So, technically, we could
    // override `use_nonce` and `use_timestamp` so as not to use the parameters. However,
    // OAuth Core 1.0 Revision A (https://oauth.net/core/1.0a/) specification used to require these
    // parameters. So, we don't override the methods here for compatibility's sake.
}
