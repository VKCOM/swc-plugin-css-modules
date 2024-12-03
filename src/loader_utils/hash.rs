use base32ct::Base32;
use base32ct::Encoding as Base32Encoding;
use base64ct::Base64;
use base64ct::Encoding as Base64Encoding;
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
        "hex" => base16ct::lower::encode_string(&input),
        _ => unimplemented!("unsupported hash digest: {}", digest_type),
    }
}

#[cfg(test)]
mod tests {
    use super::get_hash_digest;

    #[test]
    fn get_hash_digest_xxhash64() {
        assert_eq!(
            get_hash_digest("test string".as_bytes(), "xxhash64", "hex", 9999),
            "e9e2c351e3c6b198"
        );
        assert_eq!(
            get_hash_digest("test string".as_bytes(), "xxhash64", "base64", 9999),
            "6eLDUePGsZg="
        );
        assert_eq!(
            get_hash_digest("abc\\0♥".as_bytes(), "xxhash64", "hex", 9999),
            "4b9a34297dc03d20"
        );
        assert_eq!(
            get_hash_digest("abc\\0💩".as_bytes(), "xxhash64", "hex", 9999),
            "86733ec125b93904"
        );
        assert_eq!(
            get_hash_digest("abc\\0♥".as_bytes(), "xxhash64", "base64", 9999),
            "S5o0KX3APSA="
        );
        assert_eq!(
            get_hash_digest("abc\\0💩".as_bytes(), "xxhash64", "base64", 9999),
            "hnM+wSW5OQQ="
        );
    }

    #[test]
    fn get_hash_digest_md4() {
        assert_eq!(
            get_hash_digest("test string".as_bytes(), "md4", "hex", 4),
            "2e06"
        );
        assert_eq!(
            get_hash_digest("test string".as_bytes(), "md4", "base64", 9999),
            "Lgbt1PFiMmjFpRcw2KCyrw=="
        );
        assert_eq!(
            get_hash_digest("abc\\0♥".as_bytes(), "md4", "hex", 9999),
            "46b9627fecf49b80eaf01c01d86ae9fd"
        );
        assert_eq!(
            get_hash_digest("abc\\0💩".as_bytes(), "md4", "hex", 9999),
            "45aa5b332f8e562aaf0106ad6fc1d78f"
        );
        assert_eq!(
            get_hash_digest("abc\\0♥".as_bytes(), "md4", "base64", 9999),
            "Rrlif+z0m4Dq8BwB2Grp/Q=="
        );
        assert_eq!(
            get_hash_digest("abc\\0💩".as_bytes(), "md4", "base64", 9999),
            "RapbMy+OViqvAQatb8HXjw=="
        );
    }

    #[test]
    fn get_hash_digest_md5() {
        assert_eq!(
            get_hash_digest("test string".as_bytes(), "md5", "hex", 4),
            "6f8d"
        );
        assert_eq!(
            get_hash_digest("test string".as_bytes(), "md5", "hex", 9999),
            "6f8db599de986fab7a21625b7916589c"
        );
        assert_eq!(
            get_hash_digest("test string".as_bytes(), "md5", "base64", 9999),
            "b421md6Yb6t6IWJbeRZYnA=="
        );
        assert_eq!(
            get_hash_digest("abc\\0♥".as_bytes(), "md5", "hex", 9999),
            "2e897b64f8050e66aff98d38f7a012c5"
        );
        assert_eq!(
            get_hash_digest("abc\\0💩".as_bytes(), "md5", "hex", 9999),
            "63ad5b3d675c5890e0c01ed339ba0187"
        );
        assert_eq!(
            get_hash_digest("abc\\0♥".as_bytes(), "md5", "base64", 9999),
            "Lol7ZPgFDmav+Y0496ASxQ=="
        );
        assert_eq!(
            get_hash_digest("abc\\0💩".as_bytes(), "md5", "base64", 9999),
            "Y61bPWdcWJDgwB7TOboBhw=="
        );
    }

    #[test]
    fn get_hash_digest_sha512() {
        assert_eq!(
            get_hash_digest("test string".as_bytes(), "sha512", "hex", 9999),
            "10e6d647af44624442f388c2c14a787ff8b17e6165b83d767ec047768d8cbcb71a1a3226e7cc7816bc79c0427d94a9da688c41a3992c7bf5e4d7cc3e0be5dbac"
        );
        assert_eq!(
            get_hash_digest("test string".as_bytes(), "sha512", "base64", 9999),
            "EObWR69EYkRC84jCwUp4f/ixfmFluD12fsBHdo2MvLcaGjIm58x4Frx5wEJ9lKnaaIxBo5kse/Xk18w+C+XbrA=="
        );
    }
}
