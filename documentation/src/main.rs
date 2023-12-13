#![deny(missing_docs)]
#![warn()]
#![allow(dead_code)]

// use cargo doc

//! documenting module
//! with more lines
//! and more lines

/// documentation for main - the first line is most important - some summary
///
/// and the rest
///
/// with examples
/// (lines with hash are not included in the documentation; needed only for running the example)
/// ```
/// # fn main() -> Result<(), std::num::ParseIntError> {
/// let fortytwo = "42".parse::<u32>()?;
/// println!("{} + 10 = {}", fortytwo, fortytwo+10);
/// # Ok(())
/// # }
/// ````
fn main() {
    println!("Hello, world!");
}

#[doc(include = "details.md")]
#[doc(html_playground_url = "https://play.rust-lang.org/")]
mod my_module {
    // module impl here
}

/// This struct is not [Bar]
pub struct Foo1;

/// This struct is also not [bar](Bar)
pub struct Foo2;

/// This struct is also not [bar][b]
///
/// [b]: Bar
pub struct Foo3;

/// This struct is also not [`Bar`]
pub struct Foo4;

/// This struct *is* [`Bar`]!
/// See also: [`Foo`](struct@Foo)
pub struct Bar;

/// This is different from [`Foo`](fn@Foo)
struct Foo {}
fn Foo() {}

//--------------------------------------------
/// Defines a color.
///
/// # Example
///
/// ```
/// let red = Color::Red;
/// ```
pub enum Color {
    /// Color red
    Red,
    /// Color green
    Green,
    /// Color blue
    Blue,
}
