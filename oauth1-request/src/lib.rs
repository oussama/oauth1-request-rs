//! Yet yet yet another OAuth 1 client library.
//!
//! # Usage
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! oauth = { version = "0.3", package = "oauth1-request" }
//! ```
//!
//! For brevity we refer to the crate name as `oauth` throughout the documentation.
//
//! ## Create a request
//!
//! ```
//! # extern crate oauth1_request as oauth;
//! #
//! let uri = "https://example.com/api/v1/comments/create.json";
//!
//! // Define a type to represent your request.
//! #[derive(oauth::Request)]
//! struct CreateComment<'a> {
//!     article_id: u64,
//!     text: &'a str,
//! }
//!
//! let request = CreateComment {
//!     article_id: 123456789,
//!     text: "Rust lang is great 🦀",
//! };
//!
//! // Create a `Builder` and populate it with your credentials.
//! let consumer = oauth::Credentials::new("consumer_key", "consumer_secret");
//! let token = oauth::Credentials::new("token", "token_secret");
//! let mut builder = oauth::Builder::new(consumer, oauth::HmacSha1);
//! builder.token(token).nonce("nonce").timestamp(9999999999);
//!
//! // Authorize the request.
//! let authorization_header = builder.post(uri, &request);
//! assert_eq!(
//!     authorization_header,
//!     "OAuth \
//!          oauth_consumer_key=\"consumer_key\",\
//!          oauth_nonce=\"nonce\",\
//!          oauth_signature_method=\"HMAC-SHA1\",\
//!          oauth_timestamp=\"9999999999\",\
//!          oauth_token=\"token\",\
//!          oauth_signature=\"bbhEIrjfisdDBrZkKnEXKa4ykE4%3D\"",
//! );
//!
//! // You can create a x-www-form-urlencoded or a URI with query pairs from the request.
//!
//! let form = oauth::to_form_urlencoded(&request);
//! assert_eq!(
//!     form,
//!     "article_id=123456789&text=Rust%20lang%20is%20great%20%F0%9F%A6%80",
//! );
//!
//! let uri = oauth::to_uri_query(uri.to_owned(), &request);
//! assert_eq!(
//!     uri,
//!     "https://example.com/api/v1/comments/create.json?article_id=123456789&text=Rust%20lang%20is%20great%20%F0%9F%A6%80",
//! );
//! ```
//!
//! See [`Request`] for more details on the custom derive macro.

#![doc(html_root_url = "https://docs.rs/oauth1-request/0.3.2")]
#![deny(intra_doc_link_resolution_failure)]
#![warn(missing_docs, rust_2018_idioms)]

#[macro_use]
mod util;

pub mod request;
pub mod serializer;
pub mod signature_method;

#[cfg(feature = "derive")]
pub use oauth1_request_derive::Request;

pub use request::Request;
pub use serializer::Serializer;
#[cfg(feature = "hmac-sha1")]
pub use signature_method::HmacSha1;
pub use signature_method::Plaintext;

use std::borrow::Borrow;
use std::fmt::{self, Debug, Display, Formatter};
use std::str;

use serializer::auth::{self, Authorizer};
use serializer::Urlencoder;
use signature_method::SignatureMethod;

/// A builder for OAuth `Authorization` header string.
#[derive(Clone, Debug)]
pub struct Builder<'a, SM, T = String> {
    signature_method: SM,
    client: Credentials<T>,
    token: Option<Credentials<T>>,
    options: auth::Options<'a>,
}

/// The "credentials" pair defined in [RFC 5849 section 1.1][rfc].
///
/// [rfc]: https://tools.ietf.org/html/rfc5849#section-1.1
///
/// This type represents:
///
/// - Client credentials (consumer key and secrets)
/// - Temporary credentials (request token and secret)
/// - Token credentials (access token and secret)
#[derive(Clone, Copy)]
pub struct Credentials<T = String> {
    /// The unique identifier part of the credentials pair.
    pub identifier: T,
    /// The shared secret part of the credentials pair.
    pub secret: T,
}

impl<'a, SM: SignatureMethod, T: Borrow<str>> Builder<'a, SM, T> {
    /// Creates a `Builder` that signs requests using the specified client credentials
    /// and signature method.
    pub fn new(client: Credentials<T>, signature_method: SM) -> Self {
        Builder {
            signature_method,
            client,
            token: None,
            options: auth::Options::new(),
        }
    }

    /// Sets/unsets the token credentials pair to sign requests with.
    pub fn token(&mut self, token: impl Into<Option<Credentials<T>>>) -> &mut Self {
        self.token = token.into();
        self
    }

    /// Sets/unsets the `oauth_callback` URI.
    pub fn callback(&mut self, callback: impl Into<Option<&'a str>>) -> &mut Self {
        self.options.callback(callback);
        self
    }

    /// Sets/unsets the `oauth_nonce` value.
    ///
    /// By default, `Builder` generates a random nonce for each request.
    /// This method overrides that behavior and forces the `Builder` to use the specified nonce.
    ///
    /// This method is for debugging/testing purpose only and should not be used in production.
    pub fn nonce(&mut self, nonce: impl Into<Option<&'a str>>) -> &mut Self {
        self.options.nonce(nonce);
        self
    }

    /// Sets/unsets the `oauth_timestamp` value.
    ///
    /// By default, `Builder` uses the timestamp of the time when `build`(-like) method is called.
    /// This method overrides that behavior and forces the `Builder` to use the specified timestamp.
    ///
    /// This method is for debugging/testing purpose only and should not be used in production.
    pub fn timestamp(&mut self, timestamp: impl Into<Option<u64>>) -> &mut Self {
        self.options.timestamp(timestamp);
        self
    }

    /// Sets/unsets the `oauth_verifier` value.
    pub fn verifier(&mut self, verifier: impl Into<Option<&'a str>>) -> &mut Self {
        self.options.verifier(verifier);
        self
    }

    /// Sets whether to include the `oauth_version` value in requests.
    pub fn version(&mut self, version: bool) -> &mut Self {
        self.options.version(version);
        self
    }

    /// Authorizes a `GET` request to `uri`.
    pub fn get<U: Display, R: Request>(&self, uri: U, request: &R) -> String
    where
        SM: Clone,
    {
        self.build("GET", uri, request)
    }

    /// Authorizes an `PUT` request to `uri`.
    pub fn put<U: Display, R: Request>(&self, uri: U, request: &R) -> String
    where
        SM: Clone,
    {
        self.build("PUT", uri, request)
    }

    /// Authorizes an `POST` request to `uri`.
    pub fn post<U: Display, R: Request>(&self, uri: U, request: &R) -> String
    where
        SM: Clone,
    {
        self.build("POST", uri, request)
    }

    /// Authorizes a `DELETE` request to `uri`.
    pub fn delete<U: Display, R: Request>(&self, uri: U, request: &R) -> String
    where
        SM: Clone,
    {
        self.build("DELETE", uri, request)
    }

    /// Authorizes an `OPTIONS` request to `uri`.
    pub fn options<U: Display, R: Request>(&self, uri: U, request: &R) -> String
    where
        SM: Clone,
    {
        self.build("OPTIONS", uri, request)
    }

    /// Authorizes a `HEAD` request to `uri`.
    pub fn head<U: Display, R: Request>(&self, uri: U, request: &R) -> String
    where
        SM: Clone,
    {
        self.build("HEAD", uri, request)
    }

    /// Authorizes a `CONNECT` request to `uri`.
    pub fn connect<U: Display, R: Request>(&self, uri: U, request: &R) -> String
    where
        SM: Clone,
    {
        self.build("CONNECT", uri, request)
    }

    /// Authorizes an `PATCH` request to `uri`.
    pub fn patch<U: Display, R: Request>(&self, uri: U, request: &R) -> String
    where
        SM: Clone,
    {
        self.build("PATCH", uri, request)
    }

    /// Authorizes a `TRACE` request to `uri`.
    pub fn trace<U: Display, R: Request>(&self, uri: U, request: &R) -> String
    where
        SM: Clone,
    {
        self.build("TRACE", uri, request)
    }

    /// Authorizes a request to `uri` with a custom HTTP request method.
    pub fn build<U: Display, R: Request>(&self, method: &str, uri: U, request: &R) -> String
    where
        SM: Clone,
    {
        let serializer = Authorizer::with_signature_method(
            self.signature_method.clone(),
            method,
            uri,
            self.client.as_ref(),
            self.token.as_ref().map(Credentials::as_ref),
            &self.options,
        );

        request.serialize(serializer)
    }

    /// Authorizes a request and consumes `self`.
    ///
    /// This may be more efficient than `build` if the signature method holds a non-`Copy` data
    /// (e.g. RSA private key). However, the cost is the same as `build` for the signature methods
    /// bundled with this library (`HmacSha1` and `Plaintext`).
    pub fn consume<U: Display, R: Request>(self, method: &str, uri: U, request: &R) -> String {
        let serializer = Authorizer::with_signature_method(
            self.signature_method,
            method,
            uri,
            self.client.as_ref(),
            self.token.as_ref().map(Credentials::as_ref),
            &self.options,
        );

        request.serialize(serializer)
    }
}

impl<T: Borrow<str>> Credentials<T> {
    /// Creates a `Credentials` with the specified identifier and secret.
    pub fn new(identifier: T, secret: T) -> Self {
        Credentials { identifier, secret }
    }

    /// Returns the unique identifier part of the credentials pair.
    pub fn identifier(&self) -> &str {
        self.identifier.borrow()
    }

    /// Returns the shared secret part of the credentials pair.
    pub fn secret(&self) -> &str {
        self.secret.borrow()
    }

    /// Borrows the identifier and secret strings from `self`
    /// and creates a new `Credentials` with them.
    pub fn as_ref(&self) -> Credentials<&str> {
        Credentials {
            identifier: self.identifier.borrow(),
            secret: self.secret.borrow(),
        }
    }
}

impl<T: Debug> Debug for Credentials<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        struct Dummy;
        impl Debug for Dummy {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.write_str("<hidden>")
            }
        }

        f.debug_struct("Credentials")
            .field("identifier", &self.identifier)
            .field("secret", &Dummy)
            .finish()
    }
}

/// Authorizes a `GET` request to `uri` using the given credentials.
pub fn get<SM, U, R>(
    signature_method: SM,
    uri: U,
    client: Credentials<&str>,
    token: Option<Credentials<&str>>,
    request: &R,
) -> String
where
    U: Display,
    SM: SignatureMethod,
    R: Request,
{
    let mut builder = Builder::new(client, signature_method);
    builder.token(token);
    builder.consume("GET", uri, request)
}

/// Authorizes a `PUT` request to `uri` using the given credentials.
pub fn put<SM, U, R>(
    signature_method: SM,
    uri: U,
    client: Credentials<&str>,
    token: Option<Credentials<&str>>,
    request: &R,
) -> String
where
    U: Display,
    SM: SignatureMethod,
    R: Request,
{
    let mut builder = Builder::new(client, signature_method);
    builder.token(token);
    builder.consume("PUT", uri, request)
}

/// Authorizes a `POST` request to `uri` using the given credentials.
pub fn post<SM, U, R>(
    signature_method: SM,
    uri: U,
    client: Credentials<&str>,
    token: Option<Credentials<&str>>,
    request: &R,
) -> String
where
    U: Display,
    SM: SignatureMethod,
    R: Request,
{
    let mut builder = Builder::new(client, signature_method);
    builder.token(token);
    builder.consume("POST", uri, request)
}

/// Authorizes a `DELETE` request to `uri` using the given credentials.
pub fn delete<SM, U, R>(
    signature_method: SM,
    uri: U,
    client: Credentials<&str>,
    token: Option<Credentials<&str>>,
    request: &R,
) -> String
where
    U: Display,
    SM: SignatureMethod,
    R: Request,
{
    let mut builder = Builder::new(client, signature_method);
    builder.token(token);
    builder.consume("DELETE", uri, request)
}

/// Authorizes a `OPTIONS` request to `uri` using the given credentials.
pub fn options<SM, U, R>(
    signature_method: SM,
    uri: U,
    client: Credentials<&str>,
    token: Option<Credentials<&str>>,
    request: &R,
) -> String
where
    U: Display,
    SM: SignatureMethod,
    R: Request,
{
    let mut builder = Builder::new(client, signature_method);
    builder.token(token);
    builder.consume("OPTIONS", uri, request)
}

/// Authorizes a `HEAD` request to `uri` using the given credentials.
pub fn head<SM, U, R>(
    signature_method: SM,
    uri: U,
    client: Credentials<&str>,
    token: Option<Credentials<&str>>,
    request: &R,
) -> String
where
    U: Display,
    SM: SignatureMethod,
    R: Request,
{
    let mut builder = Builder::new(client, signature_method);
    builder.token(token);
    builder.consume("HEAD", uri, request)
}

/// Authorizes a `CONNECT` request to `uri` using the given credentials.
pub fn connect<SM, U, R>(
    signature_method: SM,
    uri: U,
    client: Credentials<&str>,
    token: Option<Credentials<&str>>,
    request: &R,
) -> String
where
    U: Display,
    SM: SignatureMethod,
    R: Request,
{
    let mut builder = Builder::new(client, signature_method);
    builder.token(token);
    builder.consume("CONNECT", uri, request)
}

/// Authorizes a `PATCH` request to `uri` using the given credentials.
pub fn patch<SM, U, R>(
    signature_method: SM,
    uri: U,
    client: Credentials<&str>,
    token: Option<Credentials<&str>>,
    request: &R,
) -> String
where
    U: Display,
    SM: SignatureMethod,
    R: Request,
{
    let mut builder = Builder::new(client, signature_method);
    builder.token(token);
    builder.consume("PATCH", uri, request)
}

/// Authorizes a `TRACE` request to `uri` using the given credentials.
pub fn trace<SM, U, R>(
    signature_method: SM,
    uri: U,
    client: Credentials<&str>,
    token: Option<Credentials<&str>>,
    request: &R,
) -> String
where
    U: Display,
    SM: SignatureMethod,
    R: Request,
{
    let mut builder = Builder::new(client, signature_method);
    builder.token(token);
    builder.consume("TRACE", uri, request)
}

/// Turns a `Request` into an `x-www-form-urlencoded` string.
pub fn to_form_urlencoded<R>(request: &R) -> String
where
    R: Request,
{
    request.serialize(Urlencoder::form())
}

/// Turns a `Request` to a query string and appends it to the given URI.
pub fn to_uri_query<R>(uri: String, request: &R) -> String
where
    R: Request,
{
    request.serialize(Urlencoder::query(uri))
}
