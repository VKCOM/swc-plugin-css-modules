use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::path::PathBuf;

use crate::loader_utils::hash::get_hash_digest;

pub struct LoaderContext {
    pub resource_path: Option<PathBuf>,
}

pub struct Options<'a> {
    pub context: Option<PathBuf>,
    pub content: Option<&'a [u8]>,
}

/// A Rust versions of [interpolateName](https://github.com/webpack/loader-utils#interpolatename).
///
/// The following tokens are replaced in the name parameter:
/// - `[ext]` the extension of the resource
/// - `[name]` the basename of the resource
/// - `[folder]` the folder the resource is in
/// - `[contenthash]` the hash of options.content (Buffer) (by default it's the hex digest of the xxhash64 hash)
/// - `[<hashType>:contenthash:<digestType>:<length>]` optionally one can configure
///   other hashTypes, i. e. xxhash64, sha1, md4, md5, sha256, sha512
///   other digestTypes, i. e. hex, base32, base64
///   and length the length in chars
/// - `[hash]` the hash of options.content (Buffer) (by default it's the hex digest of the xxhash64 hash)
/// In loader context `[hash]` and `[contenthash]` are the same, but we recommend using `[contenthash]` for avoid misleading.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use swc_plugin_css_modules::loader_utils::interpolate::{interpolate_name, LoaderContext, Options};
///
/// let loader_context = LoaderContext {
///     resource_path: Some("/absolute/path/to/app/js/javascript.js".into()),
/// };
///
/// let options = Options {
///     context: Some(PathBuf::from("/absolute/path/to/app")),
///     content: Some(b"content"),
/// };
///
/// assert_eq!(
///     interpolate_name(
///         loader_context,
///         "js/[hash].script.[ext]",
///         options,
///     ),
///     "js/6c5b191a31c5a9fc.script.js".to_string(),
/// );
/// ```
pub fn interpolate_name(loader_context: LoaderContext, pattern: &str, options: Options) -> String {
    let mut url = pattern.to_string();

    let mut ext = "bin";
    let mut name = "file";
    let mut folder = "";
    // TODO: support path
    // TODO: support query

    if let Some(resource_path) = &loader_context.resource_path {
        if let Some(extension) = resource_path.extension() {
            ext = extension.to_str().unwrap();
        }

        if let Some(file_name) = resource_path.file_stem() {
            name = file_name.to_str().unwrap();
        }

        if let Some(dir) = resource_path.parent().unwrap().file_name() {
            folder = dir.to_str().unwrap();
        }
    }

    if let Some(data) = options.content {
        lazy_static! {
            static ref HASH_REGEX: Regex = Regex::new(
                r"\[(?:([^\[:\]]+):)?(?:hash|contenthash)(?::([a-z]+\d*))?(?::(\d+))?\]"
            )
            .unwrap();
        }

        url = HASH_REGEX
            .replace_all(&url, |caps: &Captures| -> String {
                let algorithm = caps.get(1).map_or("xxhash64", |m| m.as_str());
                let digest_type = caps.get(2).map_or("hex", |m| m.as_str());
                let max_length = caps
                    .get(3)
                    .map_or(9999, |m: regex::Match<'_>| m.as_str().parse().unwrap());

                get_hash_digest(data, algorithm, digest_type, max_length)
            })
            .to_string();
    }

    url = url
        .replace("[ext]", ext)
        .replace("[name]", name)
        .replace("[folder]", folder);

    url
}
