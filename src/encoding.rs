pub trait Encoding {
    /// The character set of the encoding implementation.
    ///
    /// The length of this slice will always be equal to
    /// the base of the encoding.
    const CHARSET: &[u8];

    /// Shortest length `String` that will contain at least 128 bits
    /// of randomness for a given base.
    ///
    /// Calculated with ceil(log<sub>base</sub>(2<sup>128</sup>)).
    const MIN_LEN: usize;
}

/// Standard base64 encoding.
pub struct Base64;
impl Encoding for Base64 {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    const MIN_LEN: usize = 22;
}

/// URL and filename safe base64 encoding.
pub struct Base64URL;
impl Encoding for Base64URL {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    const MIN_LEN: usize = 22;
}

/// Standard base32 encoding.
pub struct Base32;
impl Encoding for Base32 {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

    const MIN_LEN: usize = 26;
}

/// Extended hexidecimal base32 encoding.
pub struct Base32Hex;
impl Encoding for Base32Hex {
    const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUV";

    const MIN_LEN: usize = 26;
}

/// Standard base16 (hexidecimal) encoding.
pub struct Base16;
impl Encoding for Base16 {
    const CHARSET: &[u8] = b"0123456789ABCDEF";

    const MIN_LEN: usize = 32;
}
