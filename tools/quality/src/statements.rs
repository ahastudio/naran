//! Counts the statements of a function body.

use syn::visit::{self, Visit};

use crate::{Finding, Place};

/// The most statements a function body may have.
pub const LIMIT: usize = 5;

/// A finding when the body has more statements than the limit.
pub fn check(place: &Place, body: &syn::Block) -> Option<Finding> {
    let count = statements(body);
    (count > LIMIT).then(|| place.message(&format!("has {count} statements")))
}

/// The statements of a body, nested blocks included.
#[must_use]
pub fn statements(body: &syn::Block) -> usize {
    let mut counter = Counter(0);
    counter.visit_block(body);
    counter.0
}

struct Counter(usize);

impl<'ast> Visit<'ast> for Counter {
    fn visit_block(&mut self, block: &'ast syn::Block) {
        self.0 += block.stmts.len();
        visit::visit_block(self, block);
    }

    /// A nested function is counted on its own.
    fn visit_item_fn(&mut self, _: &'ast syn::ItemFn) {}

    /// A nested `impl` is counted on its own.
    fn visit_item_impl(&mut self, _: &'ast syn::ItemImpl) {}

    /// A closure belongs to the statement it is written in.
    fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(body: &str) -> usize {
        statements(&syn::parse_str(body).unwrap())
    }

    #[test]
    fn statements_in_nested_blocks_count_and_closures_do_not() {
        assert_eq!(count("{ a(); b(); c }"), 3);
        assert_eq!(count("{ if x { a(); b(); } else { c(); } }"), 4);
        assert_eq!(count("{ match x { 1 => { a(); b() } _ => c() } }"), 3);
        assert_eq!(count("{ for x in y { a(); } loop { b(); } }"), 4);
        assert_eq!(count("{ let f = || { a(); b(); c(); }; f() }"), 2);
        assert_eq!(count("{ fn inner() { a(); b(); } impl X { } inner() }"), 3);
    }
}
