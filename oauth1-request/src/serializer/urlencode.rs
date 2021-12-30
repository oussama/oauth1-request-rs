//! A URI query/`x-www-form-urlencoded` string serializer.

use core::fmt::Write;

use crate::util::PercentEncode;

use super::Serializer;

/// A `Serializer` that produces a URI query or an `x-www-form-urlencoded` string from a request.
pub struct Urlencoder<
    #[cfg(feature = "alloc")] W = alloc::string::String,
    #[cfg(not(feature = "alloc"))] W,
> {
    data: W,
    next_append: Append,
}

enum Append {
    None,
    Question,
    Ampersand,
}

impl<W> Urlencoder<W>
where
    W: Write,
{
    /// Creates a `Urlencoder` that produces an `x-www-form-urlencoded` string.
    pub fn form() -> Self
    where
        W: Default,
    {
        Urlencoder {
            data: W::default(),
            next_append: Append::None,
        }
    }

    /// Same with `form` but writes the resulting form string into `buf`.
    pub fn form_with_buf(buf: W) -> Self {
        Urlencoder {
            data: buf,
            next_append: Append::None,
        }
    }

    /// Creates a `Urlencoder` that appends a query part to the given URI.
    pub fn query(uri: W) -> Self {
        Urlencoder {
            data: uri,
            next_append: Append::Question,
        }
    }

    fn append_delim(&mut self) {
        match self.next_append {
            Append::None => self.next_append = Append::Ampersand,
            Append::Question => {
                self.data.write_char('?').unwrap();
                self.next_append = Append::Ampersand;
            }
            Append::Ampersand => self.data.write_char('&').unwrap(),
        }
    }
}

impl<W: Write> Serializer for Urlencoder<W> {
    type Output = W;

    fn serialize_parameter<V>(&mut self, k: &str, v: V)
    where
        V: core::fmt::Display,
    {
        self.append_delim();
        write!(self.data, "{}={}", k, PercentEncode(&v)).unwrap();
    }

    fn serialize_parameter_encoded<V>(&mut self, k: &str, v: V)
    where
        V: core::fmt::Display,
    {
        self.append_delim();
        write!(self.data, "{}={}", k, v).unwrap();
    }

    super::skip_serialize_oauth_parameters!();

    fn end(self) -> Self::Output {
        self.data
    }
}
