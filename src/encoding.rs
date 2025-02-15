/// Specifies parameters for encoding random data into
/// a valid utf-8 `String`.
///
/// # Safety
///
/// The `CHARSET` field must only contain valid ascii characters.
/// Meaning that all u8 values must be in the interval [0, 128).
/// Failure to uphold this condition will result in `text` methods
/// returning invalid `Strings`'s.
///
/// The `MIN_LEN` field must be at least ceil(log<sub>`base`</sub>(2<sup>128</sup>)),
/// where `base` is the length of the `CHARSET` field. Failure to uphold
/// this condition will result in poor security of generated `String`'s.
pub unsafe trait YARandEncoder {
    /// The character set of the encoder implementation.
    ///
    /// See trait-level docs for security comments.
    const CHARSET: &[u8];

    /// Shortest length `String` that will contain at least 128 bits
    /// of randomness.
    ///
    /// See trait-level docs for security comments.
    const MIN_LEN: usize;
}

/// Base64 encoding, as specified by RFC 4648.
pub struct Base64;
unsafe impl YARandEncoder for Base64 {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    const MIN_LEN: usize = 22;
}

/// Base64 encoding for URLs and filenames, as specified by RFC 4648.
pub struct Base64URL;
unsafe impl YARandEncoder for Base64URL {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    const MIN_LEN: usize = 22;
}

/// Base62 (alphanumeric) encoding.
pub struct Base62;
unsafe impl YARandEncoder for Base62 {
    const CHARSET: &[u8] = crate::rng::ALPHANUMERIC;

    const MIN_LEN: usize = 22;
}

/// Base32 encoding, as specified by RFC 4648.
pub struct Base32;
unsafe impl YARandEncoder for Base32 {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

    const MIN_LEN: usize = 26;
}

/// Base32 encoding using extended hex, as specified by RFC 4648.
pub struct Base32Hex;
unsafe impl YARandEncoder for Base32Hex {
    const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUV";

    const MIN_LEN: usize = 26;
}

/// Base16 (hexidecimal) encoding, as specified by RFC 4648.
pub struct Base16;
unsafe impl YARandEncoder for Base16 {
    const CHARSET: &[u8] = b"0123456789ABCDEF";

    const MIN_LEN: usize = 32;
}
