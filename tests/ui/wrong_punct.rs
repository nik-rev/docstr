fn main() {
    // comma instead of ! is invalid
    docstr::docstr!(writeln,
        s,
        /// hello
        /// {}
        "world"
    );

    // missing comma
    docstr::docstr!(writeln!
        s
        /// hello
        /// {}
        "world"
    );

    // missing comma and no exclamation mark
    docstr::docstr!(writeln
        s
        /// hello
        /// {}
        "world"
    );

    // no exclamation mark
    docstr::docstr!(writeln
        s,
        /// hello
        /// {}
        "world"
    );
}
