#![feature(iter_intersperse)]

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use std::borrow::Cow;

/// Short for "multi-line" string literals
#[proc_macro_attribute]
pub fn mln(path: TokenStream, input: TokenStream) -> TokenStream {
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
    let mut each_doc_comment = Vec::new();

    // At the end of the macro's input we have a Group
    //
    // #[multi]
    // /// foo bar
    // /// baz
    // { ... }
    // ^^^^^^ the group
    //
    // We MUST have it because all the attributes need to apply to some kind of expression
    // The group is exactly that expression
    let mut arguments: Option<TokenStream> = None;

    // If we encounter any errors, we collect them into here
    // and report them all at once
    //
    // compile_error!("you have done horrible things!")
    let mut compile_errors = TokenStream::new();
    let mut compile_error = |span: Span, message: &str| {
        compile_errors.extend(CompileError::new(span, message));
    };

    let mut tts = input.into_iter().peekable();

    // Let's collect all of the doc comments into a Vec<String> where each
    // String corresponds to the doc comment
    while let Some(tt) = tts.next() {
        // We get this when we get to the final expression.
        //
        // let foo = #[mln]
        // /// foo
        // /// bar
        // /// baz
        // (a, b);
        // ^^^^^^ final expression, this is the `group`
        if let TokenTree::Group(group) = tt {
            if arguments.is_some() {
                compile_error(group.span(), "expected only a single group");
                continue;
            }
            arguments = Some(group.stream());
            continue;
        };

        // #[doc = "..."]
        // ^
        let span = match tts.next() {
            Some(TokenTree::Punct(punct)) if punct == '#' => punct.span(),
            Some(unexpected) => {
                compile_error(
                    unexpected.span(),
                    "expected doc attribute: `#[doc = \"...\"]`",
                );
                continue;
            }
            _ => {
                compile_error(
                    Span::call_site(),
                    "expected doc attribute: `#[doc = \"...\"]`",
                );
                continue;
            }
        };

        // #[doc = "..."]
        //  ^^^^^^^^^^^^^
        let group_tt = match tts.next() {
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket => group,
            _ => {
                compile_error(span, "expected `#` to be followed by `[...]`");
                continue;
            }
        };

        // #[doc = "..."]
        //  ^^^^^^^^^^^^^
        let mut group = group_tt.stream().into_iter();

        // #[doc = "..."]
        //   ^^^
        let span = match group.next() {
            Some(TokenTree::Ident(kw_doc)) if kw_doc.to_string() == "doc" => kw_doc.span(),
            _ => {
                compile_error(group_tt.span_open(), "expected keyword `doc` after `[`");
                continue;
            }
        };

        // #[doc = "..."]
        //       ^
        let span = match group.next() {
            Some(TokenTree::Punct(eq)) if eq == '=' => eq.span(),
            _ => {
                compile_error(span, "expected keyword `doc` after `[`");
                continue;
            }
        };

        // #[doc = "..."]
        //         ^^^^^
        let Some(TokenTree::Literal(lit)) = tts.next() else {
            compile_error(span, "expected string literal after `=`");
            continue;
        };

        let literal = lit.to_string();
        let Some(literal) = literal
            .strip_prefix('"')
            .and_then(|lit| lit.strip_suffix('"'))
        else {
            compile_error(lit.span(), "invalid string literal");
            continue;
        };

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
        let literal = literal.strip_prefix(' ').unwrap_or(literal).to_string();
        each_doc_comment.push(literal);
    }

    // Arguments to whatever macro we are expanding to
    //
    // #[mln]
    // /// foo {}
    // (a, b, c)
    // ^^^^^^^^ this
    //
    // format!("foo {}", a, b, c)
    //                   ^^^^^^^ becomes this
    let arguments = arguments.expect(concat!(
        "the attributes apply to some kind of ",
        "expression. This expression MUST exist, if ",
        "it wasn't a group would ",
        "have panicked earlier"
    ));

    // The fully constructed string literal that we output
    //
    // #[mln]
    // /// foo
    // /// bar
    //
    // becomes this:
    //
    // "foo\nbar"
    let string = TokenTree::Literal(Literal::string(
        &each_doc_comment
            .into_iter()
            .map(Cow::Owned)
            .intersperse(Cow::Borrowed("\n"))
            .collect::<String>(),
    ));

    // Expand to a single string literal if we didn't receive any arguments
    if path.is_empty() {
        if !arguments.is_empty() {
            compile_error(
                Span::call_site(),
                "you aren't applying `#[multi]` to anything; expected a single `()` as the last expression",
            );
        }
        return TokenStream::from_iter([string]);
    }

    let args_iter = path.into_iter();

    // The path to the item
    //
    // #[mln(std::format!)]
    // /// foo bar
    // ();
    //       ^^^^^^^^^^^^
    let mut path = TokenStream::new();

    // Arguments passed to the macro
    //
    // #[mln(std::format!, hello)]
    // /// foo bar
    // (x);
    //                     ^^^^^
    //
    // These arguments are inserted BEFORE the string literal
    //
    // The above will expand to:
    //
    // format!(hello, "foo bar", x)
    let mut rest = TokenStream::new();

    // If we have finished parsing the "path" which is essentially
    // until we hit a ",". It is not a real path, because it can include "!" and other tokens
    let mut finish_path = false;

    for tt in args_iter {
        match tt {
            // Not part of the path
            TokenTree::Punct(ref punct) if *punct == ',' && !finish_path => {
                // finished parsing path, now everything else is inserted directly into the token
                // stream AFTER the macro receives its string argument
                finish_path = true;
                rest.extend([tt]);
            }
            // Part of the path.
            tt => {
                path.extend([tt]);
            }
        }
    }

    // The following:
    //
    // let a = #[mln]
    // /// foo
    // /// bar
    // (a, b);
    //
    // Expands into this:
    //
    // let a = format!(hello, "foo\nbar", a, b);
    TokenStream::from_iter(
        // format!(hello, "foo\nbar", a, b)
        // ^^^^^^^
        path.into_iter().chain([TokenTree::Group(Group::new(
            // format!(hello, "foo\nbar", a, b)
            //        ^                      ^
            Delimiter::Parenthesis,
            // format!(hello, "foo\nbar", a, b)
            //         ^^^^^^^^^^^^^^^^^^^^^^^
            TokenStream::from_iter(
                // format!(hello, "foo\nbar", a, b)
                //         ^^^^^
                rest.into_iter()
                    .chain([
                        // format!(hello, "foo\nbar", a, b)
                        //                ^^^^^^^^^
                        string,
                        // format!(hello, "foo\nbar", a, b)
                        //                         ^
                        TokenTree::Punct(Punct::new(',', Spacing::Joint)),
                    ])
                    // format!(hello, "foo\nbar", a, b)
                    //                           ^^^^
                    .chain(arguments),
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
