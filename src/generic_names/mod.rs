// mod interpolate;
use lazy_static::lazy_static;
use regex::Regex;
use std::{env, path::PathBuf};

use crate::loader_utils::interpolate::{
    interpolate_name, LoaderContext, Options as LoaderUtilsOptions,
};

pub struct Options {
    pub context: PathBuf,
    pub hash_prefix: String,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            context: env::current_dir().unwrap(),
            hash_prefix: "".to_string(),
        }
    }
}

pub struct Generator {
    pattern: String,
    options: Options,
}

impl Generator {
    pub fn new(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            options: Options::default(),
        }
    }

    pub fn new_with_options(pattern: &str, options: Options) -> Self {
        Self {
            pattern: pattern.to_string(),
            options,
        }
    }
}

impl Generator {
    /// A Rust versions of [generic-names](https://github.com/css-modules/generic-names/)
    ///
    /// # Examples
    ///
    /// ```
    /// use swc_plugin_css_modules::generic_names::Generator;
    ///
    /// let generator = Generator::new("[name]__[local]___[hash:base64:5]");
    ///
    /// assert_eq!(
    ///     generator.generate("foo", "/case/source.css".into()),
    ///     "source__foo___pdq35".to_string(),
    /// );
    /// ```
    pub fn generate(&self, local_name: &str, filepath: PathBuf) -> String {
        let name = self.pattern.replace("[local]", local_name);

        let absolute_path = pathdiff::diff_paths(filepath.clone(), &self.options.context)
            .unwrap()
            .to_str()
            .unwrap()
            .replace('\\', r"/");

        let content = format!(
            "{}{}\x00{}",
            self.options.hash_prefix, absolute_path, local_name
        );

        let generic_name = interpolate_name(
            LoaderContext {
                resource_path: Some(filepath),
            },
            name.as_str(),
            LoaderUtilsOptions {
                context: Some(self.options.context.clone()),
                content: Some(content.as_bytes()),
            },
        );

        lazy_static! {
            static ref INVALID_SYMBOLS: Regex =
                Regex::new(r"[^a-zA-Z0-9\\-_\u00A0-\uFFFF]").unwrap();
            static ref INVALID_START: Regex = Regex::new(r"^((-?[0-9])|--)").unwrap();
        }

        let validate_symbols = INVALID_SYMBOLS.replace_all(&generic_name, "-");
        let result = INVALID_START.replace(&validate_symbols, "_$1");

        result.into_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::{env, path::PathBuf};

    use crate::generic_names::{Generator, Options};

    const PATTERN: &str = "[name]__[local]___[hash:base64:5]";

    fn filepath() -> PathBuf {
        env::current_dir()
            .unwrap()
            .join("test/test/case/source.css")
    }

    #[test]
    fn use_cwd_if_no_context_was_provided() {
        let generator = Generator::new(PATTERN);

        assert_eq!(generator.generate("foo", filepath()), "source__foo___VihAC");
    }

    #[test]
    fn generate_distinct_hash_for_the_provided_context() {
        let context = env::current_dir().unwrap().join("test/test");

        let generator = Generator::new_with_options(
            PATTERN,
            Options {
                context,
                hash_prefix: "".to_string(),
            },
        );

        assert_eq!(generator.generate("foo", filepath()), "source__foo___ZIJxV");
    }

    #[test]
    fn generate_distinct_hash_for_the_provided_hash_prefix() {
        let context = env::current_dir().unwrap().join("test/test");

        let generator = Generator::new_with_options(
            PATTERN,
            Options {
                context,
                hash_prefix: "--".to_string(),
            },
        );

        assert_eq!(generator.generate("foo", filepath()), "source__foo___QTVQp");
    }

    #[test]
    fn identity() {
        let generator = Generator::new("[local]");

        assert_eq!(generator.generate("foo", filepath()), "foo");
    }

    #[test]
    fn leading_digit() {
        let generator = Generator::new("0[local]");

        assert_eq!(generator.generate("foo", filepath()), "_0foo");
    }

    #[test]
    fn leading_digit_in_the_token() {
        let generator = Generator::new("[local]");

        assert_eq!(generator.generate("0foo", filepath()), "_0foo");
    }

    #[test]
    fn leading_two_hyphens() {
        let generator = Generator::new("--[local]");

        assert_eq!(generator.generate("foo", filepath()), "_--foo");
    }

    #[test]
    fn leading_hyphen_and_digit() {
        let generator = Generator::new("-0[local]");

        assert_eq!(generator.generate("foo", filepath()), "_-0foo");
    }
}
