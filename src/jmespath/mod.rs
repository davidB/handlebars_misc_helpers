#![cfg_attr(feature = "specialized", feature(specialization))]

pub use errors::{ErrorReason, JmespathError, RuntimeError};
pub use parser::{parse, ParseResult};
pub use runtime::Runtime;
pub use variable::{to_variable, Variable};

pub mod ast;
pub mod functions;

use serde::ser;
#[cfg(feature = "specialized")]
use serde_json::Value;
use std::fmt;

use ast::Ast;
use interpreter::{interpret, SearchResult};
use variable::Serializer;

mod errors;
mod interpreter;
mod lexer;
mod parser;
mod runtime;
mod variable;

lazy_static::lazy_static! {
    pub static ref DEFAULT_RUNTIME: Runtime = {
        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        runtime
    };
}

/// `Rc` reference counted JMESPath `Variable`.
#[cfg(not(feature = "sync"))]
pub type Rcvar = std::rc::Rc<Variable>;
/// `Arc` reference counted JMESPath `Variable`.
#[cfg(feature = "sync")]
pub type Rcvar = std::sync::Arc<Variable>;

/// Compiles a JMESPath expression using the default Runtime.
///
/// The default Runtime is created lazily the first time it is dereferenced
/// by using the `lazy_static` macro.
///
/// The provided expression is expected to adhere to the JMESPath
/// grammar: http://jmespath.org/specification.html
#[inline]
pub fn compile(expression: &str) -> Result<Expression<'static>, JmespathError> {
    DEFAULT_RUNTIME.compile(expression)
}

/// Converts a value into a reference-counted JMESPath Variable.
///
#[cfg_attr(
    feature = "specialized",
    doc = "\
There is a generic serde Serialize implementation, and since this
documentation was compiled with the `specialized` feature turned
**on**, there are also a number of specialized implementations for
`ToJmespath` built into the library that should work for most
cases."
)]
#[cfg_attr(
    not(feature = "specialized"),
    doc = "\
There is a generic serde Serialize implementation. Since this
documentation was compiled with the `specialized` feature turned
**off**, this is the only implementation available.

(If the `specialized` feature were turned on, there there would be
a number of additional specialized implementations for `ToJmespath`
built into the library that should work for most cases.)"
)]
pub trait ToJmespath {
    fn to_jmespath(self) -> Rcvar;
}

/// Create searchable values from Serde serializable values.
impl<'a, T: ser::Serialize> ToJmespath for T {
    #[cfg(not(feature = "specialized"))]
    fn to_jmespath(self) -> Rcvar {
        let variable = self.serialize(Serializer).unwrap();
        Rcvar::new(variable)
    }

    #[cfg(feature = "specialized")]
    default fn to_jmespath(self) -> Rcvar {
        let variable = self.serialize(Serializer).unwrap();
        Rcvar::new(variable)
    }
}

#[cfg(feature = "specialized")]
impl ToJmespath for Value {
    #[inline]
    fn to_jmespath(self) -> Rcvar {
        Rcvar::new(Variable::from(self))
    }
}

#[cfg(feature = "specialized")]
impl<'a> ToJmespath for &'a Value {
    #[inline]
    fn to_jmespath(self) -> Rcvar {
        Rcvar::new(Variable::from(self))
    }
}

#[cfg(feature = "specialized")]
/// Identity coercion.
impl ToJmespath for Rcvar {
    #[inline]
    fn to_jmespath(self) -> Rcvar {
        self
    }
}

#[cfg(feature = "specialized")]
impl<'a> ToJmespath for &'a Rcvar {
    #[inline]
    fn to_jmespath(self) -> Rcvar {
        self.clone()
    }
}

#[cfg(feature = "specialized")]
impl ToJmespath for Variable {
    #[inline]
    fn to_jmespath(self) -> Rcvar {
        Rcvar::new(self)
    }
}

#[cfg(feature = "specialized")]
impl<'a> ToJmespath for &'a Variable {
    #[inline]
    fn to_jmespath(self) -> Rcvar {
        Rcvar::new(self.clone())
    }
}

#[cfg(feature = "specialized")]
impl ToJmespath for String {
    fn to_jmespath(self) -> Rcvar {
        Rcvar::new(Variable::String(self))
    }
}

#[cfg(feature = "specialized")]
impl<'a> ToJmespath for &'a str {
    fn to_jmespath(self) -> Rcvar {
        Rcvar::new(Variable::String(self.to_owned()))
    }
}

#[cfg(feature = "specialized")]
impl ToJmespath for i8 {
    fn to_jmespath(self) -> Rcvar {
        Rcvar::new(Variable::Number(self as f64))
    }
}

#[cfg(feature = "specialized")]
impl ToJmespath for i16 {
    fn to_jmespath(self) -> Rcvar {
        Rcvar::new(Variable::Number(self as f64))
    }
}

#[cfg(feature = "specialized")]
impl ToJmespath for i32 {
    fn to_jmespath(self) -> Rcvar {
        Rcvar::new(Variable::Number(self as f64))
    }
}

#[cfg(feature = "specialized")]
impl ToJmespath for i64 {
    fn to_jmespath(self) -> Rcvar {
        Rcvar::new(Variable::Number(self as f64))
    }
}

#[cfg(feature = "specialized")]
impl ToJmespath for u8 {
    fn to_jmespath(self) -> Rcvar {
        Rcvar::new(Variable::Number(self as f64))
    }
}

#[cfg(feature = "specialized")]
impl ToJmespath for u16 {
    fn to_jmespath(self) -> Rcvar {
        Rcvar::new(Variable::Number(self as f64))
    }
}

#[cfg(feature = "specialized")]
impl ToJmespath for u32 {
    fn to_jmespath(self) -> Rcvar {
        Rcvar::new(Variable::Number(self as f64))
    }
}

#[cfg(feature = "specialized")]
impl ToJmespath for u64 {
    fn to_jmespath(self) -> Rcvar {
        Rcvar::new(Variable::Number(self as f64))
    }
}

#[cfg(feature = "specialized")]
impl ToJmespath for isize {
    fn to_jmespath(self) -> Rcvar {
        Rcvar::new(Variable::Number(self as f64))
    }
}

#[cfg(feature = "specialized")]
impl ToJmespath for usize {
    fn to_jmespath(self) -> Rcvar {
        Rcvar::new(Variable::Number(self as f64))
    }
}

#[cfg(feature = "specialized")]
impl ToJmespath for f32 {
    fn to_jmespath(self) -> Rcvar {
        Rcvar::new(Variable::Number(self as f64))
    }
}

#[cfg(feature = "specialized")]
impl ToJmespath for f64 {
    fn to_jmespath(self) -> Rcvar {
        Rcvar::new(Variable::Number(self as f64))
    }
}

#[cfg(feature = "specialized")]
impl ToJmespath for () {
    fn to_jmespath(self) -> Rcvar {
        Rcvar::new(Variable::Null)
    }
}

#[cfg(feature = "specialized")]
impl ToJmespath for bool {
    fn to_jmespath(self) -> Rcvar {
        Rcvar::new(Variable::Bool(self))
    }
}

/// A compiled JMESPath expression.
///
/// The compiled expression can be used multiple times without incurring
/// the cost of re-parsing the expression each time. The expression may
/// be shared between threads if JMESPath is compiled with the `sync`
/// feature, which forces the use of an `Arc` instead of an `Rc` for
/// runtime variables.
#[derive(Clone)]
pub struct Expression<'a> {
    ast: Ast,
    expression: String,
    runtime: &'a Runtime,
}

impl<'a> Expression<'a> {
    /// Creates a new JMESPath expression.
    ///
    /// Normally you will create expressions using either `jmespath::compile()`
    /// or using a jmespath::Runtime.
    #[inline]
    pub fn new<S>(expression: S, ast: Ast, runtime: &'a Runtime) -> Expression<'a>
    where
        S: Into<String>,
    {
        Expression {
            expression: expression.into(),
            ast,
            runtime,
        }
    }

    /// Returns the result of searching data with the compiled expression.
    ///
    /// The SearchResult contains a JMESPath Rcvar, or a reference counted
    /// Variable. This value can be used directly like a JSON object.
    /// Alternatively, Variable does implement Serde serialzation and
    /// deserialization, so it can easily be marshalled to another type.
    pub fn search<T: ToJmespath>(&self, data: T) -> SearchResult {
        let mut ctx = Context::new(&self.expression, self.runtime);
        interpret(&data.to_jmespath(), &self.ast, &mut ctx)
    }

    /// Returns the JMESPath expression from which the Expression was compiled.
    ///
    /// Note that this is the same value that is returned by calling
    /// `to_string`.
    pub fn as_str(&self) -> &str {
        &self.expression
    }

    /// Returns the AST of the parsed JMESPath expression.
    ///
    /// This can be useful for debugging purposes, caching, etc.
    #[allow(dead_code)]
    pub fn as_ast(&self) -> &Ast {
        &self.ast
    }
}

impl<'a> fmt::Display for Expression<'a> {
    /// Shows the jmespath expression as a string.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<'a> fmt::Debug for Expression<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<'a> PartialEq for Expression<'a> {
    fn eq(&self, other: &Expression) -> bool {
        self.as_str() == other.as_str()
    }
}

/// Context object used for error reporting.
///
/// The Context struct is mostly used when interacting between the
/// interpreter and function implemenations. Unless you're writing custom
/// JMESPath functions, this struct is an implementation detail.
pub struct Context<'a> {
    /// Expression string that is being interpreted.
    pub expression: &'a str,
    /// JMESPath runtime used to compile the expression and call functions.
    pub runtime: &'a Runtime,
    /// Ast offset that is currently being evaluated.
    pub offset: usize,
}

impl<'a> Context<'a> {
    /// Create a new context struct.
    #[inline]
    pub fn new(expression: &'a str, runtime: &'a Runtime) -> Context<'a> {
        Context {
            expression,
            runtime,
            offset: 0,
        }
    }
}

#[cfg(test)]
mod test {
    use super::ast::Ast;
    use super::*;

    #[test]
    fn formats_expression_as_string_or_debug() {
        let expr = compile("foo | baz").unwrap();
        assert_eq!("foo | baz/foo | baz", format!("{}/{:?}", expr, expr));
    }

    #[test]
    fn implements_partial_eq() {
        let a = compile("@").unwrap();
        let b = compile("@").unwrap();
        assert!(a == b);
    }

    #[test]
    fn can_evaluate_jmespath_expression() {
        let expr = compile("foo.bar").unwrap();
        let var = Variable::from_json("{\"foo\":{\"bar\":true}}").unwrap();
        assert_eq!(Rcvar::new(Variable::Bool(true)), expr.search(var).unwrap());
    }

    #[test]
    fn can_get_expression_ast() {
        let expr = compile("foo").unwrap();
        assert_eq!(
            &Ast::Field {
                offset: 0,
                name: "foo".to_string(),
            },
            expr.as_ast()
        );
    }

    #[test]
    fn test_creates_rcvar_from_tuple_serialization() {
        use super::ToJmespath;
        let t = (true, false);
        assert_eq!("[true,false]", t.to_jmespath().to_string());
    }

    #[test]
    fn expression_clone() {
        let expr = compile("foo").unwrap();
        let _ = expr.clone();
    }
}
