use base32ct::Base32;
use base32ct::Encoding as Base32Encoding;
use base64ct::Encoding as Base64Encoding;
use base64ct::{Base64, Base64Url};
use digest::DynDigest;
use std::hash::Hasher;

/// A Rust versions of [getHashDigest](https://github.com/webpack/loader-utils#gethashdigest).
///
/// # Arguments
///
/// * `data` - the content that should be hashed
/// * `algorithm` - one of `xxhash64`, `sha1`, `md4`, `md5`, `sha256`, `sha512`
/// * `digest_type` - one of `hex`, `base32`, `base64`
/// * `max_length` - the maximum length in chars
pub fn get_hash_digest(
    data: &[u8],
    algorithm: &str,
    digest_type: &str,
    max_length: usize,
) -> String {
    let hash = use_hasher(algorithm, data);
    let encoded = use_digest(digest_type, hash);
    let result = encoded.get(0..max_length).unwrap_or(encoded.as_str());

    result.to_string()
}

fn use_hasher(hash_type: &str, data: &[u8]) -> Box<[u8]> {
    // TODO: rewrite to DynDigest
    if hash_type == "xxhash64" {
        return xxhash64(data);
    }

    let mut hasher = select_hasher(hash_type);
    hasher.update(data);

    hasher.finalize_reset()
}

fn xxhash64(data: &[u8]) -> Box<[u8]> {
    let mut hasher = twox_hash::XxHash64::default();
    hasher.write(data);

    Box::new(hasher.finish().to_be_bytes())
}

#[allow(clippy::box_default)]
fn select_hasher(hash_type: &str) -> Box<dyn DynDigest> {
    match hash_type {
        "md4" => Box::new(md4::Md4::default()),
        "md5" => Box::new(md5::Md5::default()),
        "sha1" => Box::new(sha1::Sha1::default()),
        "sha224" => Box::new(sha2::Sha224::default()),
        "sha256" => Box::new(sha2::Sha256::default()),
        "sha384" => Box::new(sha2::Sha384::default()),
        "sha512" => Box::new(sha2::Sha512::default()),
        _ => unimplemented!("unsupported hash function: {}", hash_type),
    }
}

fn use_digest(digest_type: &str, input: Box<[u8]>) -> String {
    match digest_type {
        "base32" => Base32::encode_string(&input),
        "base64" => Base64::encode_string(&input),
        "base64url" => Base64Url::encode_string(&input),
        "hex" => base16ct::lower::encode_string(&input),
        _ => unimplemented!("unsupported hash digest: {}", digest_type),
    }
}
