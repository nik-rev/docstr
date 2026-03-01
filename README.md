# `docstr`

<!-- cargo-reedme: start -->

<!-- cargo-reedme: info-start

    Do not edit this region by hand
    ===============================

    This region was generated from Rust documentation comments by `cargo-reedme` using this command:

        cargo reedme

    for more info: https://github.com/nik-rev/cargo-reedme

cargo-reedme: info-end -->

[![crates.io](https://img.shields.io/crates/v/docstr?style=flat-square&logo=rust)](https://crates.io/crates/docstr)
[![docs.rs](https://img.shields.io/docsrs/docstr?style=flat-square&logo=docs.rs)](https://docs.rs/docstr)
![license](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue?style=flat-square)
![msrv](https://img.shields.io/badge/msrv-1.65-blue?style=flat-square&logo=rust)
[![github](https://img.shields.io/github/stars/nik-rev/docstr)](https://github.com/nik-rev/docstr)

This crate provides a macro [`docstr!`](https://docs.rs/docstr/0.4.9/docstr/macro.docstr.html) for ergonomically creating multi-line string literals.

```toml
docstr = "0.4"
```

Note: `docstr` does not have any dependencies such as `syn` or `quote`, so compile-speeds are very fast.

## Usage

[`docstr!`](https://docs.rs/docstr/0.4.9/docstr/macro.docstr.html) takes documentation comments as arguments and converts them into a string

```rust
use docstr::docstr;

let hello_world_in_c: &'static str = docstr!(
    /// #include <stdio.h>
    ///
    /// int main(int argc, char **argv) {
    ///     printf("hello world\n");
    ///     return 0;
    /// }
);

assert_eq!(hello_world_in_c, r#"#include <stdio.h>

int main(int argc, char **argv) {
    printf("hello world\n");
    return 0;
}"#)
```

## Composition

[`docstr!`](https://docs.rs/docstr/0.4.9/docstr/macro.docstr.html) can pass the generated string to any macro. This example shows the string being forwarded to the [`format!`](https://doc.rust-lang.org/stable/alloc/macro.format.html) macro:

```rust
let name = "Bob";
let age = 21;

let greeting: String = docstr!(format!
    /// Hello, my name is {name}.
    /// I am {} years old!
    age
);

assert_eq!(greeting, "\
Hello, my name is Bob.
I am 21 years old!");
```

This is great because there’s just a single macro, `docstr!`, that can do anything. No need for `docstr_format!`, `docstr_println!`, `docstr_write!`, etc.

### How composition works

If the first argument to `docstr!` is a path to a macro, that macro will be called. This invocation:

```rust
let greeting: String = docstr!(format!
    /// Hello, my name is {name}.
    /// I am {} years old!
    age
);
```

Is equivalent to this:

```rust
let greeting: String = format!("\
Hello, my name is {name}.
I am {} years old!"
    age,
);
```

You can inject arguments before the format string:

```rust
docstr!(write! w
   /// Hello, world!
);
```

Expands to:

```rust
write!(w, "Hello, world!");
```

## Global Import

This will make `docstr!` globally accessible in your entire crate, without needing to import it:

```rust
#[macro_use(docstr)]
extern crate docstr;
```

<!-- cargo-reedme: end -->
