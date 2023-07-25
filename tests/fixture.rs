use std::{env, fs, path::PathBuf};

use swc_core::ecma::{
    parser::{EsConfig, Syntax},
    transforms::testing::test_fixture,
    visit::as_folder,
};
use swc_plugin_css_modules::{Config, Injector};

fn syntax() -> Syntax {
    Syntax::Es(EsConfig {
        jsx: true,
        ..Default::default()
    })
}

#[testing::fixture("tests/fixture/**/input.js")]
fn fixture(input: PathBuf) {
    let output = input.parent().unwrap().join("output.js");
    let config_path = input.parent().unwrap().join("config.json");

    let config_file = fs::File::open(config_path).expect("failed to open config");

    let config: Config = serde_json::from_reader(config_file).expect("invalid config");

    let cwd = env::current_dir().unwrap();

    test_fixture(
        syntax(),
        &|_| {
            as_folder(Injector::new(
                cwd.to_str().unwrap(),
                input.to_str().unwrap(),
                config.clone(),
            ))
        },
        &input,
        &output,
        Default::default(),
    );
}
