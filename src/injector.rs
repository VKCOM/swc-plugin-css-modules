use std::collections::HashMap;
use std::path::PathBuf;

use path_absolutize::*;
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
            generator: Generator::new_with_options(
                config.generate_scoped_name.as_str(),
                Options {
                    context,
                    hash_prefix: config.hash_prefix,
                },
            ),
        }
    }

    fn new_import(&mut self, local: &Atom, src: &Atom) {
        let p = PathBuf::from(src.to_string());

        let filepath = p.absolutize_from(self.dir.clone()).unwrap().to_path_buf();

        if !filepath.has_root() {
            panic!(
                "dir: {}; p: {}; filepath: {}",
                self.dir.to_str().unwrap(),
                p.to_str().unwrap(),
                filepath.to_str().unwrap()
            )
        }

        self.imports.insert(local.clone(), filepath);
    }

    /// Return class name from list.
    fn get_generated_name(&self, module: &Atom, name: &Atom) -> String {
        let filepath = self.imports.get(module).unwrap().to_path_buf();

        self.generator.generate(name.to_string().as_str(), filepath)
    }
}

impl VisitMut for Injector {
    fn visit_mut_expr(&mut self, n: &mut Expr) {
        n.visit_mut_children_with(self);

        if self.imports.is_empty() {
            return;
        }

        if let Expr::Member(member) = n {
            if let Expr::Ident(obj) = &*member.obj {
                // Проверяем что переменная используется для css модулей
                if !self.imports.contains_key(&obj.sym) {
                    return;
                }

                match &member.prop {
                    // styles.title
                    MemberProp::Ident(i) => {
                        let generated_name = self.get_generated_name(&obj.sym, &i.sym);

                        let exp = Expr::from(generated_name);

                        n.clone_from(&exp)
                    }

                    // this.#message
                    MemberProp::PrivateName(el) => HANDLER.with(|handler| {
                        handler
                            .struct_span_err(el.span, "PrivateName not used in css modules")
                            .emit()
                    }),

                    MemberProp::Computed(computed) => match &*computed.expr {
                        // styles['Component--disabled']
                        Expr::Lit(Lit::Str(str_lit)) => {
                            let generated_name = self.get_generated_name(&obj.sym, &str_lit.value);

                            let exp = Expr::from(generated_name);

                            n.clone_from(&exp)
                        }

                        // styles[prefix + "title"]
                        _ => HANDLER.with(|handler| {
                            handler
                                .struct_span_err(computed.span, "Computed hit cannot be injected")
                                .emit()
                        }),
                    },
                }
            }
        }
    }

    fn visit_mut_import_decl(&mut self, n: &mut ImportDecl) {
        n.visit_mut_children_with(self);

        // Проверяем что это импорт css modules

        if !n
            .src
            .value
            .ends_with(self.config.css_modules_suffix.as_str())
            || n.specifiers.is_empty()
        {
            return;
        }

        let src = &n.src.value;

        // Вытаскиваем название переменной и обрабатываем css modules
        n.specifiers.iter().for_each(|specifier| match specifier {
            // import { foo } from "./Component.module.css"
            ImportSpecifier::Named(_) => HANDLER.with(|handler| {
                handler
                    .struct_span_err(n.span, "Named import is not supported")
                    .emit()
            }),

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
