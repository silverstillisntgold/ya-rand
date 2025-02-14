/// Specifies parameters for encoding random data into
/// a valid utf-8 `String`.
///
/// # Safety
///
/// The `CHARSET` field must only contain valid ascii characters.
/// Failure to uphold this condition will result in `text` methods
/// returning invalid `Strings`'s.
///
/// The `MIN_LEN` field must be at least ceil(log<sub>`base`</sub>(2<sup>128</sup>)),
/// where `base` is the length of the `CHARSET` field. Failure to
/// uphold this condition will result in lower overall security of
/// generated `String`'s. If you'd like you target a different security level,
/// change 128 to the level of security desired when computing your `MIN_LEN`.
/// Going lower than 128 is **extremely** discouraged, but higher values may be
/// desireable for some applications.
pub unsafe trait Encoding {
    /// The character set of the encoding implementation.
    ///
    /// It is recommended that all characters are unique, although
    /// that's not strictly required. If you want certain characters
    /// to appear more frequently in the generated `String`'s, then
    /// having more of them in `CHARSET` is a reasonable way to introduce bias.
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
unsafe impl Encoding for Base64 {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    const MIN_LEN: usize = 22;
}

/// Base64 encoding for URLs and filenames, as specified by RFC 4648.
pub struct Base64URL;
unsafe impl Encoding for Base64URL {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    const MIN_LEN: usize = 22;
}

/// Base62 (alphanumeric) encoding.
pub struct Base62;
unsafe impl Encoding for Base62 {
    const CHARSET: &[u8] = crate::rng::ALPHANUMERIC;

    const MIN_LEN: usize = 22;
}

/// Base32 encoding, as specified by RFC 4648.
pub struct Base32;
unsafe impl Encoding for Base32 {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

    const MIN_LEN: usize = 26;
}

/// Base32 encoding using extended hex, as specified by RFC 4648.
pub struct Base32Hex;
unsafe impl Encoding for Base32Hex {
    const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUV";

    const MIN_LEN: usize = 26;
}

/// Base16 (hexidecimal) encoding, as specified by RFC 4648.
pub struct Base16;
unsafe impl Encoding for Base16 {
    const CHARSET: &[u8] = b"0123456789ABCDEF";

    const MIN_LEN: usize = 32;
}
