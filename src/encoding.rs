pub trait Encoding {
    const ALPHABET: &[u8];

    const MIN_LEN: usize;
}

pub struct Base64;
impl Encoding for Base64 {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    const MIN_LEN: usize = 22;
}

pub struct Base64Safe;
impl Encoding for Base64Safe {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    const MIN_LEN: usize = 22;
}

pub struct Base32;
impl Encoding for Base32 {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

    const MIN_LEN: usize = 26;
}

pub struct Base32Extended;
impl Encoding for Base32Extended {
    const ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUV";

    const MIN_LEN: usize = 26;
}

pub struct Base16;
impl Encoding for Base16 {
    const ALPHABET: &[u8] = b"0123456789ABCDEF";

    const MIN_LEN: usize = 32;
}
