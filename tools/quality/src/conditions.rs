//! The condition of an `if` or a `while` reads at a glance.

use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};

use crate::Finding;

/// The most `&&` and `||` one condition may have.
pub const LIMIT: usize = 2;

/// What is wrong with one condition.
pub fn check(path: &Path, condition: &syn::Expr) -> Vec<Finding> {
    let span = condition.span();
    let (start, end) = (span.start().line, span.end().line);
    let messages = problems(&Operators::in_condition(condition), end - start);
    let at = |message: &str| finding(path, start, message);
    messages.iter().map(|message| at(message)).collect()
}

fn problems(operators: &Operators, extra_lines: usize) -> Vec<String> {
    let count = operators.and + operators.or;
    let mixed = operators.and > 0 && operators.or > 0;
    let found = [
        (extra_lines > 0).then(|| "a condition spans more than one line".to_owned()),
        mixed.then(|| "a condition mixes && with ||".to_owned()),
        (count > LIMIT).then(|| format!("a condition has {count} logical operators")),
    ];
    found.into_iter().flatten().collect()
}

fn finding(path: &Path, line: usize, message: &str) -> Finding {
    Finding {
        path: path.to_path_buf(),
        line,
        message: message.to_owned(),
    }
}

#[derive(Default)]
struct Operators {
    and: usize,
    or: usize,
}

impl Operators {
    fn in_condition(condition: &syn::Expr) -> Self {
        let mut operators = Self::default();
        operators.visit_expr(condition);
        operators
    }
}

impl<'ast> Visit<'ast> for Operators {
    fn visit_bin_op(&mut self, operator: &'ast syn::BinOp) {
        self.and += usize::from(matches!(operator, syn::BinOp::And(_)));
        self.or += usize::from(matches!(operator, syn::BinOp::Or(_)));
    }

    /// A closure is read on its own.
    fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {}

    /// So is a block.
    fn visit_block(&mut self, _: &'ast syn::Block) {}
}

/// Every `if` and `while` condition in a function body.
pub struct Conditions<'a> {
    pub path: &'a Path,
    pub found: Vec<Finding>,
}

impl<'ast> Visit<'ast> for Conditions<'_> {
    /// A function or other item inside the body is checked on its own.
    fn visit_item(&mut self, _: &'ast syn::Item) {}

    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        self.found.extend(check(self.path, &expression.cond));
        visit::visit_expr_if(self, expression);
    }

    fn visit_expr_while(&mut self, expression: &'ast syn::ExprWhile) {
        self.found.extend(check(self.path, &expression.cond));
        visit::visit_expr_while(self, expression);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn messages(body: &str) -> Vec<String> {
        let body: syn::Block = syn::parse_str(body).unwrap();
        let mut conditions = Conditions {
            path: Path::new("a.rs"),
            found: Vec::new(),
        };
        conditions.visit_block(&body);
        let found = conditions.found.into_iter();
        found.map(|finding| finding.message).collect()
    }

    #[test]
    fn a_plain_condition_is_fine() {
        assert_eq!(
            messages("{ if a && b { while c || d {} } }"),
            Vec::<String>::new()
        );
        assert_eq!(
            messages("{ if f(|x| x && y || z) { } }"),
            Vec::<String>::new()
        );
        assert_eq!(messages("{ if { a && b || c } { } }"), Vec::<String>::new());
    }

    #[test]
    fn mixed_long_and_wrapped_conditions_are_found() {
        assert_eq!(
            messages("{ if a && b || c && d { } }"),
            [
                "a condition mixes && with ||",
                "a condition has 3 logical operators"
            ]
        );
        assert_eq!(
            messages("{ while a\n&& b { } }"),
            ["a condition spans more than one line"]
        );
    }
}
