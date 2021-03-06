pub use self::expr::*;
pub use self::stmt::*;

pub mod expr;
pub mod stmt;

use std::iter::FromIterator;

/// A Ruby program.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program
{
    pub items: Vec<Item>,
}

/// An item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Item
{
    Module(Module),
    Class(Class),
    Function(Function),
    Stmt(Stmt),
}

/// A module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Module
{
    /// The name of the module.
    pub name: String,
    /// The items contained in the module.
    pub items: Vec<Item>,
}

/// A class.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Class
{
    /// The name of the class.
    pub name: String,
    /// The items contained in the class.
    pub items: Vec<Item>,
    /// The parent class.
    pub superclass: Option<Path>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function
{
    /// The name of the function.
    pub name: String,
    /// The statements in the function.
    pub statements: Vec<Stmt>,
}

/// An identifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identifier(pub String);

/// A constant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Constant(pub String);

/// A list of identifiers separated by periods.
///
/// `my.object.do_thing`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Path
{
    /// The parts that make up the path.
    pub parts: Vec<PathSegment>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathSegment
{
    pub kind: PathSegmentKind,
    pub separator: PathSeparator,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathSegmentKind
{
    Constant(Constant),
    Identifier(Identifier),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PathSeparator
{
    /// The root part of a path.
    /// Should only be used on the very first segment.
    Root,
    Dot,
    DoubleColon,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Argument
{
    /// A standard positional argument.
    Positional(Expr),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parameter
{
    pub name: String,
    pub default: Option<Box<Expr>>,
}

impl Program
{
    pub fn new() -> Self {
        Program { items: Vec::new() }
    }
}

impl Module
{
    pub fn new<S>(name: S) -> Self where S: Into<String> {
        Module { name: name.into(), items: Vec::new() }
    }
}

impl Class
{
    pub fn new<S>(name: S) -> Self where S: Into<String> {
        Class { name: name.into(), items: Vec::new(), superclass: None }
    }
}

impl PathSegmentKind
{
    pub fn new(text: String) -> Self {
        if text.chars().next().unwrap().is_uppercase() {
            PathSegmentKind::Constant(Constant(text))
        } else {
            PathSegmentKind::Identifier(Identifier(text))
        }
    }
}

impl Into<Item> for Class { fn into(self) -> Item { Item::Class(self) } }
impl Into<Item> for Module { fn into(self) -> Item { Item::Module(self) } }
impl Into<Item> for Function { fn into(self) -> Item { Item::Function(self) } }
impl Into<Item> for Stmt { fn into(self) -> Item { Item::Stmt(self) } }

impl FromIterator<PathSegment> for Path
{
    fn from_iter<T>(it: T) -> Self where T: IntoIterator<Item=PathSegment> {
        Path { parts: it.into_iter().collect() }
    }
}
