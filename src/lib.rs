mod config;
pub mod generic_names;
mod injector;
pub mod loader_utils;

pub use config::Config;
pub use injector::Injector;
use swc_core::ecma::{ast::Program, visit::visit_mut_pass};
use swc_core::plugin::metadata::TransformPluginMetadataContextKind;
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let config: Config = serde_json::from_str(
        &metadata
            .get_transform_plugin_config()
            .expect("failed to get plugin config"),
    )
    .expect("invalid config");

    let filepath = metadata
        .get_context(&TransformPluginMetadataContextKind::Filename)
        .expect("failed to get filepath");

    let cwd = metadata
        .get_context(&TransformPluginMetadataContextKind::Cwd)
        .expect("failed to get cwd");

    program.apply(visit_mut_pass(Injector::new(
        cwd.as_str(),
        filepath.as_str(),
        config,
    )))
}
