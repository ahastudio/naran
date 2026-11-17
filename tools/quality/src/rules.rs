//! Walks one file and applies every rule to the code that is not a test.

use std::path::Path;

use syn::visit::{self, Visit};

use crate::conditions::Conditions;
use crate::duplicates::Body;
use crate::{Checked, Place, parameters, statements};

/// Everything the code of one file breaks that is not a test.
pub fn check(path: &Path, file: &syn::File) -> Checked {
    let mut rules = Rules::new(path);
    rules.visit_file(file);
    rules.checked
}

/// Whether the attributes keep an item to the tests: `#[cfg(test)]`,
/// `#[test]` or `#[gpui::test]`.
pub fn is_test_only(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        let test = |name: &syn::Ident| name == "test";
        let named = attribute
            .path()
            .segments
            .last()
            .map(|segment| &segment.ident);
        named.is_some_and(test) || is_cfg_test(attribute)
    })
}

fn is_cfg_test(attribute: &syn::Attribute) -> bool {
    let condition = attribute.parse_args::<syn::Ident>();
    attribute.path().is_ident("cfg") && condition.is_ok_and(|name| name == "test")
}

struct Rules<'a> {
    path: &'a Path,
    checked: Checked,
    in_trait_implementation: bool,
}

impl<'a> Rules<'a> {
    fn new(path: &'a Path) -> Self {
        Self {
            path,
            checked: Checked::default(),
            in_trait_implementation: false,
        }
    }

    /// Checks a function that is not a test, and says whether it was one.
    fn own(
        &mut self,
        attributes: &[syn::Attribute],
        signature: &syn::Signature,
        body: &syn::Block,
    ) -> bool {
        let own = !is_test_only(attributes);
        if own {
            self.function(signature, body);
        }
        own
    }

    fn function(&mut self, signature: &syn::Signature, body: &syn::Block) {
        let place = Place::of(self.path, &signature.ident);
        let parameters = self.parameters(&place, signature);
        let long = statements::check(&place, body);
        self.checked
            .findings
            .extend(long.into_iter().chain(parameters));
        self.body(&place, body);
    }

    /// A trait decides the parameters of its implementations.
    fn parameters(&self, place: &Place, signature: &syn::Signature) -> Vec<crate::Finding> {
        if self.in_trait_implementation {
            return Vec::new();
        }
        parameters::check(place, signature)
    }

    /// The conditions in the body, and the body to compare with the others.
    fn body(&mut self, place: &Place, body: &syn::Block) {
        let mut conditions = Conditions {
            path: self.path,
            found: Vec::new(),
        };
        conditions.visit_block(body);
        self.checked.findings.extend(conditions.found);
        self.checked.bodies.extend(Body::of(place, body));
    }
}

impl<'ast> Visit<'ast> for Rules<'_> {
    fn visit_item_mod(&mut self, module: &'ast syn::ItemMod) {
        if !is_test_only(&module.attrs) {
            visit::visit_item_mod(self, module);
        }
    }

    fn visit_item_impl(&mut self, implementation: &'ast syn::ItemImpl) {
        let of_trait = implementation.trait_.is_some();
        let outer = std::mem::replace(&mut self.in_trait_implementation, of_trait);
        visit::visit_item_impl(self, implementation);
        self.in_trait_implementation = outer;
    }

    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        if self.own(&function.attrs, &function.sig, &function.block) {
            visit::visit_item_fn(self, function);
        }
    }

    fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
        if self.own(&function.attrs, &function.sig, &function.block) {
            visit::visit_impl_item_fn(self, function);
        }
    }

    fn visit_trait_item_fn(&mut self, function: &'ast syn::TraitItemFn) {
        if let Some(body) = &function.default {
            self.function(&function.sig, body);
        }
        visit::visit_trait_item_fn(self, function);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn messages(source: &str) -> Vec<String> {
        let file = syn::parse_file(source).unwrap();
        let checked = check(Path::new("a.rs"), &file);
        let found = checked.findings().iter();
        found.map(|finding| finding.message.clone()).collect()
    }

    const LONG: &str = "{ a(); b(); c(); d(); e(); f(); }";

    #[test]
    fn every_kind_of_function_is_checked() {
        let source = format!(
            "fn free() {LONG}\nimpl S {{ fn method(&self) {LONG} }}\n\
             trait T {{ fn required(&self); fn provided(&self) {LONG} }}\n\
             fn outer() {{ fn inner() {LONG} inner() }}"
        );
        let expected =
            ["free", "method", "provided", "inner"].map(|name| format!("{name} has 6 statements"));
        assert_eq!(messages(&source), expected);
    }

    #[test]
    fn tests_are_left_out() {
        let source = format!(
            "#[test]\nfn a() {LONG}\n#[gpui::test]\nfn b() {LONG}\n\
             impl S {{ #[test]\nfn c() {LONG} }}\n\
             #[cfg(test)]\nmod tests {{ fn d() {LONG} }}\n\
             #[cfg(feature = \"x\")]\nmod other {{ fn e(flag: bool) {{}} }}"
        );
        assert_eq!(messages(&source), ["e takes the flag `flag`"]);
    }

    #[test]
    fn a_trait_decides_the_parameters_of_its_implementations() {
        let source = "impl T for S { fn f(a: A, b: B, c: C, d: D) { if a && b || c && d {} } }\n\
                      impl S { fn g(flag: bool) {} }";
        assert_eq!(
            messages(source),
            [
                "a condition mixes && with ||",
                "a condition has 3 logical operators",
                "g takes the flag `flag`"
            ]
        );
    }
}
