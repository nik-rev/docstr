# `docstr`

<!-- cargo-rdme start -->

[![crates.io](https://img.shields.io/crates/v/docstr?style=flat-square&logo=rust)](https://crates.io/crates/docstr)
[![docs.rs](https://img.shields.io/badge/docs.rs-docstr-blue?style=flat-square&logo=docs.rs)](https://docs.rs/docstr)
![license](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue?style=flat-square)
![msrv](https://img.shields.io/badge/msrv-1.56-blue?style=flat-square&logo=rust)
[![github](https://img.shields.io/github/stars/nik-rev/docstr)](https://github.com/nik-rev/docstr)

This crate provides a procedural macro for ergonomically creating multi-line string literals.
It is an alternative to [`indoc`](https://docs.rs/indoc/latest/indoc/).

```toml
[dependencies]
docstr = "0.1"
```

## Usage

`docstr!` takes documentation comments as arguments and converts them into a string

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

## Macros

`docstr!` can pass the generated string to any macro:

```rust
use docstr::docstr;

let age = 21;
let name = "Bob";
let colors = ["red", "green", "blue"];

let greeting: String = docstr!(format
                             //^^^^^^ the generated string is passed to `format!`
                             //       as the 1st argument
    /// Hello, my name is {name}.
    /// I am {age} years old!
    ///
    /// My favorite color is: {}

    // anything after the doc comments is passed directly at the end
    colors[1]
);
//^ above expands to: format!("...", colors[1])

assert_eq!(greeting, "Hello, my name is Bob.\nI am 21 years old!\n\nMy favorite color is: green");
```

Injecting arguments before the generated string is also possible.

```rust
docstr!(write, w
   /// Hello, world!
);
```

Expands to:

```rust
write!(w, "Hello, world!");
```

<!-- cargo-rdme end -->
