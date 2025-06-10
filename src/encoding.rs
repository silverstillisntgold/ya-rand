use crate::rng::ALPHANUMERIC;

/// Specifies parameters for encoding random data into a valid UTF-8 `String`.
///
/// All provided implementations list their minimum secure length
/// in the documentation of their unit struct, and it is highly recommended
/// that custom implementations all do the same.
///
/// # Safety
///
/// The `CHARSET` field must only contain valid ascii characters,
/// meaning that all u8 values must be in the interval [0, 128).
/// Failure to uphold this condition will result in generation of
/// invalid `String values`.
///
/// The `MIN_LEN` field must be at least ceil(log<sub>`base`</sub>(2<sup>128</sup>)),
/// where `base` is the length of the `CHARSET` field. Failure to uphold
/// this condition will result in poor security of generated `String` values.
pub unsafe trait Encoder {
    /// The character set of the encoder implementation.
    ///
    /// See trait-level docs for safety comments.
    const CHARSET: &[u8];

    /// Shortest length `String` that will contain at least 128 bits
    /// of randomness.
    ///
    /// See trait-level docs for safety comments.
    const MIN_LEN: usize;
}

/// Base64 encoding, as specified by RFC 4648.
///
/// Minimum secure length is 22.
pub struct Base64;
unsafe impl Encoder for Base64 {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    const MIN_LEN: usize = 22;
}

/// Base64 (URLs and filenames) encoding, as specified by RFC 4648.
///
/// Minimum secure length is 22.
pub struct Base64URL;
unsafe impl Encoder for Base64URL {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    const MIN_LEN: usize = 22;
}

/// Base62 (alphanumeric) encoding.
///
/// Minimum secure length is 22.
pub struct Base62;
unsafe impl Encoder for Base62 {
    const CHARSET: &[u8] = ALPHANUMERIC;

    const MIN_LEN: usize = 22;
}

/// Base32 encoding, as specified by RFC 4648.
///
/// Minimum secure length is 26.
pub struct Base32;
unsafe impl Encoder for Base32 {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

    const MIN_LEN: usize = 26;
}

/// Base32 (extended hexidecimal) encoding, as specified by RFC 4648.
///
/// Minimum secure length is 26.
pub struct Base32Hex;
unsafe impl Encoder for Base32Hex {
    const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUV";

    const MIN_LEN: usize = 26;
}

/// Base16 (hexidecimal) encoding, as specified by RFC 4648.
///
/// Minimum secure length is 32.
pub struct Base16;
unsafe impl Encoder for Base16 {
    const CHARSET: &[u8] = b"0123456789ABCDEF";

    const MIN_LEN: usize = 32;
}

/// Base16 (lowercase hexidecimal) encoding.
///
/// Minimum secure length is 32.
pub struct Base16Lowercase;
unsafe impl Encoder for Base16Lowercase {
    const CHARSET: &[u8] = b"0123456789abcdef";

    const MIN_LEN: usize = 32;
}
