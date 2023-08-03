use serde::Deserialize;
use serde_inline_default::serde_inline_default;
use ts_rs::TS;

// TODO: locals_convention

#[serde_inline_default]
#[derive(Debug, Deserialize, Clone, TS)]
#[ts(export, export_to = "types.d.ts")]
pub struct Config {
    #[serde_inline_default("[hash:base64]".to_string())]
    pub generate_scoped_name: String,

    #[serde_inline_default("".to_string())]
    pub hash_prefix: String,

    #[serde_inline_default(".css".to_string())]
    pub css_modules_suffix: String,

    #[serde_inline_default("".to_string())]
    pub root: String,
}
