use std::collections::HashMap;
use std::path::PathBuf;

use path_absolutize::*;
use swc_core::atoms::Wtf8Atom;
use swc_core::ecma::ast::{Expr, ImportDecl, ImportSpecifier, Lit, MemberProp};
use swc_core::ecma::atoms::Atom;
use swc_core::ecma::visit::{VisitMut, VisitMutWith};
use swc_core::plugin::errors::HANDLER;

use crate::generic_names::{Generator, Options};
use crate::Config;

/// Returns the full path to the file's directory.
///
/// - swc/loader and swc/jest pass full `filepath`
/// - swc/cli pass relative `filepath`
fn get_dir(mut context: PathBuf, filepath: PathBuf) -> PathBuf {
    if filepath.has_root() {
        return filepath.parent().unwrap().to_path_buf();
    }

    context.push(filepath);

    context.parent().unwrap().to_path_buf()
}

pub struct Injector {
    dir: PathBuf,
    config: Config,

    generator: Generator,
    imports: HashMap<Atom, PathBuf>,
    named_imports: HashMap<Atom, (Atom, PathBuf)>,
}

impl Injector {
    pub fn new(cwd: &str, filepath: &str, config: Config) -> Self {
        let context = PathBuf::from(if config.root.is_empty() {
            cwd.to_string()
        } else {
            config.root.clone()
        });

        let dir = get_dir(context.clone(), PathBuf::from(filepath));

        Self {
            dir,
            config: config.clone(),
            imports: HashMap::new(),
            named_imports: HashMap::new(),
            generator: Generator::new_with_options(
                config.generate_scoped_name.as_str(),
                Options {
                    context,
                    hash_prefix: config.hash_prefix,
                },
            ),
        }
    }

    fn filepath_from_src(&self, src: &Wtf8Atom) -> PathBuf {
        let path_buf = PathBuf::from(src.as_atom().expect("non-utf8 string").to_string());

        let filepath = path_buf
            .absolutize_from(self.dir.clone())
            .unwrap()
            .to_path_buf();

        if !filepath.has_root() {
            panic!(
                "dir: {}; p: {}; filepath: {}",
                self.dir.to_str().unwrap(),
                path_buf.to_str().unwrap(),
                filepath.to_str().unwrap()
            )
        }

        filepath
    }

    fn new_import(&mut self, local: &Atom, src: &Wtf8Atom) {
        let filepath = self.filepath_from_src(src);

        self.imports.insert(local.clone(), filepath);
    }

    fn new_named_import(&mut self, imported: &Atom, local: &Atom, src: &Wtf8Atom) {
        let filepath = self.filepath_from_src(src);

        self.named_imports
            .insert(local.clone(), (imported.clone(), filepath));
    }

    /// Returns class name from list.
    fn generated_name(&self, module: &Atom, name: &Atom) -> String {
        let filepath = self.imports.get(module).unwrap().to_path_buf();

        self.generator.generate(name.to_string().as_str(), filepath)
    }

    fn generated_name_for_named_import(&self, name: &Atom) -> String {
        let (imported, filepath) = self.named_imports.get(name).unwrap();

        self.generator
            .generate(imported.to_string().as_str(), filepath.to_path_buf())
    }
}

impl VisitMut for Injector {
    fn visit_mut_expr(&mut self, expression: &mut Expr) {
        expression.visit_mut_children_with(self);

        if self.imports.is_empty() && self.named_imports.is_empty() {
            return;
        }

        match expression {
            Expr::Member(member) => {
                if let Expr::Ident(obj) = &*member.obj {
                    // Check variable usage for css modules
                    if !self.imports.contains_key(&obj.sym) {
                        return;
                    }

                    match &member.prop {
                        // styles.title
                        MemberProp::Ident(i) => {
                            let generated_name = self.generated_name(&obj.sym, &i.sym);

                            let exp = Expr::from(generated_name);

                            expression.clone_from(&exp)
                        }

                        MemberProp::Computed(computed) => match &*computed.expr {
                            // styles['Component--disabled']
                            Expr::Lit(Lit::Str(str_lit)) => {
                                let generated_name = self.generated_name(
                                    &obj.sym,
                                    str_lit.value.as_atom().expect("non-utf8 key"),
                                );

                                let exp = Expr::from(generated_name);

                                expression.clone_from(&exp)
                            }

                            // styles[prefix + "title"]
                            _ => HANDLER.with(|handler| {
                                handler
                                    .struct_span_err(
                                        computed.span,
                                        "Computed hit cannot be injected",
                                    )
                                    .emit()
                            }),
                        },

                        _ => {}
                    }
                }
            }

            // import { foo } from "./Component.module.css"
            //
            // className(foo)
            Expr::Ident(ident) => {
                if !self.named_imports.contains_key(&ident.sym) {
                    return;
                }

                let generated_name = self.generated_name_for_named_import(&ident.sym);

                let exp = Expr::from(generated_name);

                expression.clone_from(&exp)
            }
            _ => {}
        }
    }

    fn visit_mut_import_decl(&mut self, n: &mut ImportDecl) {
        n.visit_mut_children_with(self);

        // Check if it's a css modules import

        if !n
            .src
            .value
            .as_str()
            .expect("non-utf8 string")
            .ends_with(self.config.css_modules_suffix.as_str())
            || n.specifiers.is_empty()
        {
            return;
        }

        let src = &n.src.value;

        // Extract variable name and process css modules
        n.specifiers.iter().for_each(|specifier| match specifier {
            // import { foo as bar } from "./Component.module.css"
            ImportSpecifier::Named(named) => self.new_named_import(
                &named
                    .imported
                    .clone()
                    .map_or(named.local.sym.clone(), |s| s.atom().as_ref().to_owned()),
                &named.local.sym,
                src,
            ),

            // import styles from "./Component.module.css"
            ImportSpecifier::Default(default) => self.new_import(&default.local.sym, src),

            // import * as styles from "./Component.module.css"
            ImportSpecifier::Namespace(namespace) => self.new_import(&namespace.local.sym, src),
        });

        // import styles from "./Component.module.css";
        // ↓ ↓ ↓ ↓ ↓ ↓
        // import "./Component.module.css";
        n.specifiers.clear();
    }
}
