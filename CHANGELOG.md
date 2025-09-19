# v0.4.0

This release includes more improvement to the error messages, plus a few syntactic changes to make the macro more readable.

- The passed macro path must always be followed by a `!`

  ```rs
  docstr!(println!
        //       ^ exclamation mark is required now
    /// Hello, world!
  );
  ```

- The comma between the last argument before the first doc comment, and the doc comment itself is now required

  ```rs
  docstr!(write! w,
        //        ^ this comma was previously optional,
        //          but now it is required
    /// Hello, world!
  );
  ```

- The comma between passed macro path and the first argument is no longer accepted

  ```rs
  docstr!(write! w,
        //      ^ we would previously expect a comma here, but it is
        //        no longer accepted as the `!` is a better indicator
    /// Hello, world!
  );
  ```

# v0.3.0

- Improved error messages
- A few invalid cases which were previously accepted are now compile errors

# v0.2.0

Require at least 1 doc comment inside of `docstr!`

# v0.1.0

Initial release
