//! [![crates.io](https://img.shields.io/crates/v/docstr?style=flat-square&logo=rust)](https://crates.io/crates/docstr)
//! [![docs.rs](https://img.shields.io/badge/docs.rs-docstr-blue?style=flat-square&logo=docs.rs)](https://docs.rs/docstr)
//! ![license](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue?style=flat-square)
//! ![msrv](https://img.shields.io/badge/msrv-1.56-blue?style=flat-square&logo=rust)
//! [![github](https://img.shields.io/github/stars/nik-rev/docstr)](https://github.com/nik-rev/docstr)
//!
//! This crate provides a procedural macro for ergonomically creating multi-line string literals.
//! It is an alternative to [`indoc`](https://docs.rs/indoc/latest/indoc/).
//!
//! ```toml
//! [dependencies]
//! docstr = "0.4"
//! ```
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
//! [`docstr!`](crate::docstr) can pass the generated string to any macro:
//!
//! ```rust
//! use docstr::docstr;
//!
//! let age = 21;
//! let name = "Bob";
//! let colors = ["red", "green", "blue"];
//!
//! let greeting: String = docstr!(format!
//!                              //^^^^^^^ the generated string is passed to `format!`
//!                              //        as the 1st argument
//!     /// Hello, my name is {name}.
//!     /// I am {age} years old!
//!     ///
//!     /// My favorite color is: {}
//!
//!     // anything after the doc comments is passed directly at the end
//!     colors[1]
//! );
//! //^ above expands to: format!("...", colors[1])
//!
//! assert_eq!(greeting, "Hello, my name is Bob.\nI am 21 years old!\n\nMy favorite color is: green");
//! ```
//!
//! Injecting arguments before the generated string is also possible.
//!
//! ```rust
//! # let mut w = String::new();
//! # use std::fmt::Write as _;
//! # use docstr::docstr;
//! docstr!(write! w,
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
    // If this is `None`, this macro produces a string literal
    let macro_ = match input.peek() {
        Some(TokenTree::Punct(punct)) if *punct == '#' => {
            // No macro, this will directly produce a string literal
            None
        }
        // Ok, this is a path to a macro.
        Some(_) => {
            let mut macro_ = TokenStream::new();
            // for better error messages
            let mut last_is_ident = false;

            // on the first compile error we stop trying to process the path because it won't
            // make any sense after that
            loop {
                let tt = input.next();
                match tt {
                    // std::format!
                    //            ^
                    Some(TokenTree::Punct(exclamation)) if exclamation == '!' => {
                        macro_.extend([TokenTree::Punct(exclamation)]);
                        // end of the macro
                        break;
                    }
                    // std::format!
                    //    ^
                    //     ^
                    Some(TokenTree::Punct(colon)) if colon == ':' => {
                        last_is_ident = false;
                        macro_.extend([TokenTree::Punct(colon)]);
                    }
                    // std::format!
                    // ^^^
                    //      ^^^^^^
                    Some(TokenTree::Ident(ident)) => {
                        if last_is_ident {
                            compile_error(ident.span(), &format!("2 identifiers in a row is not a valid macro path\n\ndid you mean one of:\n- `{macro_}::{ident}`\n- `{macro_}! {ident}`"));
                            macro_ = TokenStream::new();
                            break;
                        }

                        last_is_ident = true;
                        macro_.extend([TokenTree::Ident(ident)]);
                    }
                    Some(TokenTree::Punct(comma)) if comma == ',' => {
                        compile_error(
                            comma.span(),
                            &format!("replace with `!` to pass the macro: `{macro_}!`",),
                        );
                        macro_ = TokenStream::new();
                        break;
                    }
                    _ => {
                        let span = tt.map(|tt| tt.span()).unwrap_or_else(|| {
                            macro_
                                .clone()
                                .into_iter()
                                .last()
                                .map(|last| last.span())
                                .unwrap_or_else(Span::call_site)
                        });
                        compile_error(
                            span,
                            concat!(
                                "expected path ",
                                "to macro like: `std::format!`\n\nnote: ",
                                "macro path is optional and can be omitted ",
                                "to produce a `&'static str`"
                            ),
                        );
                        macro_ = TokenStream::new();
                        break;
                    }
                }
            }

            Some(macro_)
        }
        // Macro input is totally empty - just expand to an empty string
        None => {
            return CompileError::new(
                Span::call_site(),
                "expected at least 1 documentation comment `/// ...`",
            )
            .into_iter()
            .collect()
        }
    };

    // Tokens BEFORE the doc comments, which are appended
    // directly to the `macro_` we just got
    let mut before = TokenStream::new();

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
    // directly to the `macr` we just got
    let mut after = TokenStream::new();

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
                after.extend([tt]);
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
                let is_current_comma =
                    matches!(tt, TokenTree::Punct(ref punct_1) if *punct_1 == ',');
                let current_span = tt.span();
                before.extend([tt]);

                // Not `,` followed by `#` is a syntax error
                //
                // docstr!(write! foo
                //                // ^ missing `,` here
                //     /// hello world
                // )
                match input.peek() {
                    Some(TokenTree::Punct(next)) if !is_current_comma && *next == '#' => {
                        compile_error(current_span, "expected `,` after this");

                        // Recover from the error so we can collect more errors
                        before.extend([TokenTree::Punct(Punct::new(',', Spacing::Joint))]);
                    }
                    _ => (),
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
            "expected at least 1 documentation comment `/// ...`",
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

    let Some(macro_) = macro_ else {
        if !before.is_empty() || !after.is_empty() {
            compile_error(
                Span::call_site(),
                concat!(
                    "expected macro input to only contain doc comments `///`, ",
                    "because you haven't supplied a path to a macro as the 1st argument"
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
    // let a = docstr!(
    //     format,
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
                before
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
                    .chain(after),
            ),
        ))]),
    )
}

/// `.into_iter()` generates `compile_error!($message)` at `$span`
struct CompileError {
    /// Where the compile error is generates
    pub span: Span,
    /// Message of the compile error
    pub message: String,
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
