#![doc = concat!("[![crates.io](https://img.shields.io/crates/v/", env!("CARGO_PKG_NAME"), "?style=flat-square&logo=rust)](https://crates.io/crates/", env!("CARGO_PKG_NAME"), ")")]
#![doc = concat!("[![docs.rs](https://img.shields.io/docsrs/", env!("CARGO_PKG_NAME"), "?style=flat-square&logo=docs.rs)](https://docs.rs/", env!("CARGO_PKG_NAME"), ")")]
#![doc = "![license](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue?style=flat-square)"]
#![doc = concat!("![msrv](https://img.shields.io/badge/msrv-", env!("CARGO_PKG_RUST_VERSION"), "-blue?style=flat-square&logo=rust)")]
//! [![github](https://img.shields.io/github/stars/nik-rev/docstr)](https://github.com/nik-rev/docstr)
//!
//! This crate provides a macro [`docstr!`] for ergonomically creating multi-line string literals.
//!
//! ```toml
#![doc = concat!(env!("CARGO_PKG_NAME"), " = ", "\"", env!("CARGO_PKG_VERSION_MAJOR"), ".", env!("CARGO_PKG_VERSION_MINOR"), "\"")]
//! ```
//!
//! Note: `docstr` does not have any dependencies such as `syn` or `quote`, so compile-speeds are very fast.
//!
//! # Usage
//!
//! [`docstr!`](crate::docstr) takes documentation comments as arguments and converts them into a string
//!
//! ```rust
//! use docstr::docstr;
//!
//! let hello_world_in_c: &'static str = docstr!(
//!     /// #include <stdio.h>
//!     ///
//!     /// int main(int argc, char **argv) {
//!     ///     printf("hello world\n");
//!     ///     return 0;
//!     /// }
//! );
//!
//! assert_eq!(hello_world_in_c, r#"#include <stdio.h>
//!
//! int main(int argc, char **argv) {
//!     printf("hello world\n");
//!     return 0;
//! }"#)
//! ```
//!
//! # Composition
//!
//! [`docstr!`](crate::docstr) can pass the generated string to any macro. This example shows the string being forwarded to the [`format!`] macro:
//!
//! ```
//! # use docstr::docstr;
//! let name = "Bob";
//! let age = 21;
//!
//! let greeting: String = docstr!(format!
//!     /// Hello, my name is {name}.
//!     /// I am {} years old!
//!     age
//! );
//!
//! assert_eq!(greeting, "\
//! Hello, my name is Bob.
//! I am 21 years old!");
//! ```
//!
//! This is great because there's just a single macro, `docstr!`, that can do anything. No need for `docstr_format!`, `docstr_println!`, `docstr_write!`, etc.
//!
//! ## How composition works
//!
//! If the first argument to `docstr!` is a path to a macro, that macro will be called. This invocation:
//!
//! ```
//! # use docstr::docstr;
//! # let name = "";
//! # let age = "";
//! let greeting: String = docstr!(format!
//!     /// Hello, my name is {name}.
//!     /// I am {} years old!
//!     age
//! );
//! ```
//!
//! Is equivalent to this:
//!
//! ```
//! # let name = "";
//! # let age = "";
//! let greeting: String = format!("\
//! Hello, my name is {name}.
//! I am {} years old!",
//!     age
//! );
//! ```
//!
//! You can inject arguments before the format string:
//!
//! ```rust
//! # let mut w = String::new();
//! # use std::fmt::Write as _;
//! # use docstr::docstr;
//! docstr!(write! w
//!    /// Hello, world!
//! );
//! ```
//!
//! Expands to:
//!
//! ```rust
//! # let mut w = String::new();
//! # use std::fmt::Write as _;
//! write!(w, "Hello, world!");
//! ```
//!
//! # Global Import
//!
//! This will make `docstr!` globally accessible in your entire crate, without needing to import it:
//!
//! ```
//! #[macro_use(docstr)]
//! extern crate docstr;
//! ```

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

/// Turns documentation comments into string at compile-time.
///
/// ```rust
/// use docstr::docstr;
///
/// let hello_world: String = docstr!(format!
///     /// fn say_hi() {{
///     ///     println!("Hello, my name is {}");
///     /// }}
///     "Bob"
/// );
///
/// assert_eq!(hello_world, r#"fn say_hi() {
///     println!("Hello, my name is Bob");
/// }"#);
/// ```
///
/// Expands to this:
///
/// ```rust
/// format!(r#"fn say_hi() {{
///     println!("Hello, my name is {}");
/// }}"#, "Bob");
/// ```
///
/// See the [crate-level](crate) documentation for more info
#[proc_macro]
pub fn docstr(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter().peekable();

    // If we encounter any errors, we collect them into here
    // and report them all at once
    //
    // compile_error!("you have done horrible things!")
    let mut compile_errors = TokenStream::new();
    let mut compile_error = |span: Span, message: &str| {
        compile_errors.extend(CompileError::new(span, message));
    };

    // Path to the macro that we send tokens to.
    //
    // If this is `None`, we don't forward the path to any macro,
    // and docstr! produces a string literal of type &'static str
    //
    // docstr!(
    //     /// hello world
    // )
    // => "hello world"
    //
    // docstr!(format!
    //     /// hello {world}
    // )
    // => format!("hello {world}")
    let macro_path = match input.peek() {
        // No macro path, this will directly produce a string literal
        //
        // docstr!(
        //     /// hello world
        // )
        Some(TokenTree::Punct(punct)) if *punct == '#' => None,
        // Macro input is completely empty
        //
        // docstr!()
        None => {
            return CompileError::new(
                Span::call_site(),
                "requires at least a documentation comment argument: `/// ...`",
            )
            .into()
        }
        // Path to a macro.
        //
        // docstr!(format!
        //     /// hello {world}
        // )
        Some(_) => {
            // Contains tokens of the macro, e.g. `std::format!`
            match extract_macro_path(&mut input) {
                Ok(macro_path) => macro_path,
                Err(compile_error) => return compile_error.into(),
            }
        }
    };

    // Tokens BEFORE the doc comments, which are appended
    // directly to the `macro_path` we just got - before the `doc_comments`
    let mut tokens_before_doc_comments = TokenStream::new();

    // Contents of the doc comments which we collect
    //
    // /// foo
    // /// bar
    //
    // Expands to:
    //
    // #[doc = "foo"]
    // #[doc = "bar"]
    //
    // Which we collect to:
    //
    // ["foo", "bar"]
    let mut doc_comments = Vec::new();

    // Tokens AFTER the doc comments, which are appended
    // directly to the `macro_path` we just got - after the `doc_comments`
    let mut tokens_after_doc_comments = TokenStream::new();

    /// In the middle of `docstr!(...)` macro's invocation, we will always have doc comments.
    ///
    /// ```ignore
    /// docstr!(
    ///     // DocComments::NotReached
    ///     but we can have tokens here
    ///     // DocComments::Inside
    ///     /// foo
    ///     /// bar
    ///     // DocComments::Finished
    ///     and here too
    /// )
    /// ```
    #[derive(Eq, PartialEq, PartialOrd, Ord)]
    enum DocCommentProgress {
        /// doc comments `///` not reached yet
        NotReached,
        /// currently we are INSIDE the doc comments
        Inside,
        /// We have parsed all the doc comments
        Finished,
    }

    // State machine corresponding to our current progress in the macro
    let mut doc_comment_progress = DocCommentProgress::NotReached;

    // Let's collect all of the doc comments into a Vec<String> where each
    // String corresponds to the doc comment
    while let Some(tt) = input.next() {
        // #[doc = "..."]
        // ^
        let doc_comment_start_span = match tt {
            // this token is passed verbatim to the macro at the end,
            // after the doc comments
            tt if doc_comment_progress == DocCommentProgress::Finished => {
                tokens_after_doc_comments.extend([tt]);
                continue;
            }
            // start of doc comment
            TokenTree::Punct(punct) if punct == '#' => {
                match doc_comment_progress {
                    DocCommentProgress::NotReached => {
                        doc_comment_progress = DocCommentProgress::Inside;
                    }
                    DocCommentProgress::Inside => {
                        // ok
                    }
                    DocCommentProgress::Finished => {
                        unreachable!("if it's finished we would `continue` in an earlier arm")
                    }
                }
                match input.peek() {
                    Some(TokenTree::Punct(punct)) if *punct == '!' => {
                        compile_error(
                            punct.span(),
                            "Inner doc comments `//! ...` are not supported. Please use `/// ...`",
                        );
                        // eat '!'
                        input.next();
                    }
                    _ => (),
                }
                punct.span()
            }
            // this token is passed verbatim to the macro at the beginning,
            // before the doc comments
            tt if doc_comment_progress == DocCommentProgress::NotReached => {
                // Comma before '#' is optional
                //
                // docstr!(writeln! w,
                //                   ^ this comma can be omitted
                //     #[doc = "..."]
                //     ^ next token
                // )
                let insert_comma = match input.peek() {
                    Some(TokenTree::Punct(next)) => match &tt {
                        TokenTree::Punct(current) if *current == ',' && *next == '#' => false,
                        _ if *next == '#' => true,
                        _ => false,
                    },
                    _ => false,
                };

                tokens_before_doc_comments.extend([tt]);

                if insert_comma {
                    tokens_before_doc_comments
                        .extend([TokenTree::Punct(Punct::new(',', Spacing::Joint))]);
                }

                continue;
            }
            _ => {
                unreachable!("when the next token is not `#` progress is `Finished`")
            }
        };

        // #[doc = "..."]
        //  ^^^^^^^^^^^^^
        let doc_comment_square_brackets = match input.next() {
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket => group,
            Some(tt) => {
                compile_error(tt.span(), "expected `[...]`");
                continue;
            }
            None => {
                compile_error(
                    doc_comment_start_span,
                    "expected `#` to be followed by `[...]`",
                );
                continue;
            }
        };

        // Check if there is a doc comment after this one
        //
        // #[doc = "..."]            #[doc = "..."]
        // ^^^^^^^^^^^^^^ current    ^ next?
        match input.peek() {
            Some(TokenTree::Punct(punct)) if *punct == '#' => {
                // Yes, there is. Continue doc comment
            }
            _ => {
                // The next token is not `#` so there are no more doc comments
                doc_comment_progress = DocCommentProgress::Finished;
            }
        }

        // #[doc = "..."]
        //  ^^^^^^^^^^^^^
        let mut doc_comment_attribute_inner = doc_comment_square_brackets.stream().into_iter();

        // #[doc = "..."]
        //   ^^^
        let kw_doc_span = match doc_comment_attribute_inner.next() {
            Some(TokenTree::Ident(kw_doc)) if kw_doc.to_string() == "doc" => kw_doc.span(),
            Some(tt) => {
                compile_error(tt.span(), "expected `doc`");
                continue;
            }
            None => {
                compile_error(
                    doc_comment_square_brackets.span_open(),
                    "expected `doc` after `[`",
                );
                continue;
            }
        };

        // #[doc = "..."]
        //       ^
        let punct_eq_span = match doc_comment_attribute_inner.next() {
            Some(TokenTree::Punct(eq)) if eq == '=' => eq.span(),
            Some(tt) => {
                compile_error(tt.span(), "expected `=`");
                continue;
            }
            None => {
                compile_error(kw_doc_span, "expected `=` after `doc`");
                continue;
            }
        };

        // #[doc = "..."]
        //         ^^^^^
        let next = doc_comment_attribute_inner.next();
        let Some(tt) = next else {
            compile_error(punct_eq_span, "expected string literal after `=`");
            continue;
        };
        let span = tt.span();

        // #[doc = "..."]
        //          ^^^
        let Ok(litrs::Literal::String(literal)) = litrs::Literal::try_from(tt) else {
            compile_error(
                span,
                "only string \"...\" or r\"...\" literals are supported",
            );
            continue;
        };

        let literal = literal.value();

        // Reached contents of the doc comment
        //
        // let's remove leading space
        //
        // /// foo bar
        //
        // this expands to:
        //
        // #[doc = " foo bar"]
        //          ^ remove this space from the actual output
        //
        // We usually always have a space after the comment token,
        // since it looks good. And e.g. Rustdoc ignores it as well.
        let literal = literal.strip_prefix(' ').unwrap_or(literal);

        doc_comments.push(literal.to_string());
    }

    if doc_comments.is_empty() {
        compile_error(
            Span::call_site(),
            "requires at least a documentation comment argument: `/// ...`",
        );
    }

    // The fully constructed string literal that we output
    //
    // docstr!(
    //     /// foo
    //     /// bar
    // )
    //
    // becomes this:
    //
    // "foo\nbar"
    let string = doc_comments
        .into_iter()
        .reduce(|mut acc, s| {
            acc.push('\n');
            acc.push_str(&s);
            acc
        })
        .unwrap_or_default();

    let Some(macro_) = macro_path else {
        if !tokens_before_doc_comments.is_empty() || !tokens_after_doc_comments.is_empty() {
            compile_error(
                Span::call_site(),
                concat!(
                    "expected macro input to only contain doc comments: `/// ...`, ",
                    "because you haven't supplied a macro path as the 1st argument"
                ),
            );
        }

        if !compile_errors.is_empty() {
            return compile_errors;
        }

        // Just a plain string literal
        return TokenTree::Literal(Literal::string(&string)).into();
    };

    if !compile_errors.is_empty() {
        return compile_errors;
    }

    // The following:
    //
    // let a = docstr!(format!
    //     hello
    //     /// foo
    //     /// bar
    //     a,
    //     b
    // );
    //
    // Expands into this:
    //
    // let a = format!(hello, "foo\nbar", a, b);
    TokenStream::from_iter(
        // format!(hello, "foo\nbar", a, b)
        // ^^^^^^^
        macro_.into_iter().chain([TokenTree::Group(Group::new(
            // format!(hello, "foo\nbar", a, b)
            //        ^                      ^
            Delimiter::Parenthesis,
            // format!(hello, "foo\nbar", a, b)
            //         ^^^^^^^^^^^^^^^^^^^^^^^
            TokenStream::from_iter(
                // format!(hello, "foo\nbar", a, b)
                //         ^^^^^^
                tokens_before_doc_comments
                    .into_iter()
                    .chain([
                        // format!(hello, "foo\nbar", a, b)
                        //                ^^^^^^^^^^
                        TokenTree::Literal(Literal::string(&string)),
                        // format!(hello, "foo\nbar", a, b)
                        //                          ^
                        TokenTree::Punct(Punct::new(',', Spacing::Joint)),
                    ])
                    // format!(hello, "foo\nbar", a, b)
                    //                            ^^^^
                    .chain(tokens_after_doc_comments),
            ),
        ))]),
    )
}

/// Extracts path to macro, if one exists
///
/// ```ignore
/// docstr!(::std::format!
///         ^^^^^^^^^^^^^^
///     /// ...
/// )
/// ```
fn extract_macro_path(
    input: &mut std::iter::Peekable<proc_macro::token_stream::IntoIter>,
) -> Result<Option<TokenStream>, CompileError> {
    let mut macro_path = TokenStream::new();

    enum PreviousMacroPathToken {
        PathSeparator,
        Ident,
    }

    // Tracked for better error messages
    let mut previous_macro_path_token = None;

    macro_rules! invalid_macro_path {
        () => {
            CompileError::new(
                macro_path
                    .into_iter()
                    .next()
                    .map(|tt| tt.span())
                    .unwrap_or_else(Span::call_site),
                "invalid macro path",
            )
        };
    }

    // on the first compile error we stop trying to process the path because it won't
    // make any sense after that
    loop {
        let tt = input.next();
        match tt {
            // Reached end of macro
            //
            // std::format!
            //            ^
            Some(TokenTree::Punct(exclamation)) if exclamation == '!' => {
                macro_path.extend([TokenTree::Punct(exclamation)]);
                break;
            }
            // std::format!
            //    ^^
            Some(TokenTree::Punct(colon)) if colon == ':' => {
                match previous_macro_path_token {
                    Some(PreviousMacroPathToken::Ident) | None => {
                        previous_macro_path_token = Some(PreviousMacroPathToken::PathSeparator);
                    }
                    Some(PreviousMacroPathToken::PathSeparator) => {
                        return Err(invalid_macro_path!());
                    }
                }

                macro_path.extend([TokenTree::Punct(colon)]);

                match input.next() {
                    // std::format!
                    //     ^
                    Some(TokenTree::Punct(colon)) if colon == ':' => {
                        macro_path.extend([TokenTree::Punct(colon)]);
                    }
                    _ => {
                        return Err(invalid_macro_path!());
                    }
                }
            }
            // std::format!
            // ^^^
            //      ^^^^^^
            Some(TokenTree::Ident(ident)) => match previous_macro_path_token {
                Some(PreviousMacroPathToken::PathSeparator) | None => {
                    macro_path.extend([TokenTree::Ident(ident)]);
                    previous_macro_path_token = Some(PreviousMacroPathToken::Ident);
                }
                Some(PreviousMacroPathToken::Ident) => {
                    return Err(invalid_macro_path!());
                }
            },
            _ if !macro_path.is_empty() => {
                let macro_path_display = macro_path.to_string();
                let last_token = macro_path.into_iter().last().expect("!.is_empty()");
                return Err(CompileError::new(
                    last_token.span(),
                    format!("macro path must be followed by `!`, try: `{macro_path_display}!`"),
                ));
            }
            _ => {
                return Err(CompileError::new(
                    tt.map(|tt| tt.span()).unwrap_or_else(Span::call_site),
                    "unexpected token",
                ));
            }
        }
    }

    Ok(Some(macro_path))
}

/// `.into_iter()` generates `compile_error!($message)` at `$span`
struct CompileError {
    /// Where the compile error is generates
    pub span: Span,
    /// Message of the compile error
    pub message: String,
}

impl From<CompileError> for TokenStream {
    fn from(value: CompileError) -> Self {
        value.into_iter().collect()
    }
}

impl CompileError {
    /// Create a new compile error
    pub fn new(span: Span, message: impl AsRef<str>) -> Self {
        Self {
            span,
            message: message.as_ref().to_string(),
        }
    }
}

impl IntoIterator for CompileError {
    type Item = TokenTree;
    type IntoIter = std::array::IntoIter<Self::Item, 3>;

    fn into_iter(self) -> Self::IntoIter {
        [
            TokenTree::Ident(Ident::new("compile_error", self.span)),
            TokenTree::Punct({
                let mut punct = Punct::new('!', Spacing::Alone);
                punct.set_span(self.span);
                punct
            }),
            TokenTree::Group({
                let mut group = Group::new(Delimiter::Brace, {
                    TokenStream::from_iter(vec![TokenTree::Literal({
                        let mut string = Literal::string(&self.message);
                        string.set_span(self.span);
                        string
                    })])
                });
                group.set_span(self.span);
                group
            }),
        ]
        .into_iter()
    }
}
