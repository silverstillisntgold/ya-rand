/// Specifies a charset for encoding and a minimum length
/// for encoded strings.
pub trait Encoding {
    /// The character set of the encoding implementation.
    ///
    /// The length of this slice should always be equal to
    /// the base of the encoding.
    const CHARSET: &[u8];

    /// Shortest length `String` that will contain at least 128 bits
    /// of randomness for a given base.
    ///
    /// Calculated with ceil(log<sub>base</sub>(2<sup>128</sup>)).
    const MIN_LEN: usize;
}

/// Base64 encoding, as specified by RFC 4648.
pub struct Base64;
impl Encoding for Base64 {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    const MIN_LEN: usize = 22;
}

/// Base64 encoding for URLs and filenames, as specified by RFC 4648.
pub struct Base64URL;
impl Encoding for Base64URL {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    const MIN_LEN: usize = 22;
}

/// Base62 encoding. Similar to Base64 but with only alphanumeric characters.
pub struct Base62;
impl Encoding for Base62 {
    const CHARSET: &[u8] = crate::rng::ASCII_CHARSET;

    const MIN_LEN: usize = 22;
}

/// Base32 encoding, as specified by RFC 4648.
pub struct Base32;
impl Encoding for Base32 {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

    const MIN_LEN: usize = 26;
}

/// Base32 encoding using extended hex, as specified by RFC 4648.
pub struct Base32Hex;
impl Encoding for Base32Hex {
    const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUV";

    const MIN_LEN: usize = 26;
}

/// Base16 (hexidecimal) encoding, as specified by RFC 4648.
pub struct Base16;
impl Encoding for Base16 {
    const CHARSET: &[u8] = b"0123456789ABCDEF";

    const MIN_LEN: usize = 32;
}
