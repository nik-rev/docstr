#![cfg(test)]
use docstr::docstr;

const AGE: u32 = 19;

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}

#[test]
fn empty() {
    const A: &str = docstr!(
        ///
    );

    assert_eq!(A, "");
}

/// Works with constants
#[test]
fn constant() {
    const A: &str = docstr!(
        /// foo
        /// bar
    );

    assert_eq!(A, "foo\nbar", "join with newline");

    const B: &str = docstr!(
        /// foo
        /// bar
        ///
    );

    assert_eq!(B, "foo\nbar\n", "newline at end");
}

/// Can do string interpolation
#[test]
fn format() {
    assert_eq!(
        docstr!(format!
            /// Hello, my name is {}
            /// and I am {AGE} years old
            "Bob"
        ),
        format!("Hello, my name is Bob\nand I am {AGE} years old")
    );
}

/// NO macro, but `{}`
#[test]
fn fake_interpolation() {
    assert_eq!(
        docstr!(
            /// I am {AGE} years old
        ),
        "I am {AGE} years old"
    );
}

/// String interpolation using a custom macro
#[test]
fn formatln() {
    macro_rules! formatln {
        ($($tt:tt)*) => {
            format!($($tt)*) + "\n"
        };
    }

    assert_eq!(
        docstr!(
            formatln!
            /// Hello, my name is {}
            /// and I am {AGE} years old
            "Bob"
        ),
        format!("Hello, my name is Bob\nand I am {AGE} years old\n")
    );
}

/// Accepts arguments before the string
#[test]
fn writeln() {
    use std::fmt::Write as _;
    let mut s = String::new();
    docstr!(writeln!
        s,
        /// hello
        /// {}
        "world"
    )
    .unwrap();

    assert_eq!(s, "hello\nworld\n");

    // Same, but a comma after `s`

    let mut s = String::new();
    docstr!(writeln!
        s,
        /// hello
        /// {}
        "world"
    )
    .unwrap();

    assert_eq!(s, "hello\nworld\n");
}

#[test]
fn escape() {
    assert_eq!(
        docstr!(
            /// hello "world" ' \ ! ()
            /// ///\\/\// \u{0032}
        ),
        "hello \"world\" ' \\ ! ()\n///\\\\/\\// \\u{0032}"
    );
}
