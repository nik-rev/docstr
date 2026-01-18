# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

[Unreleased]: https://github.com/nik-rev/docstr/compare/v0.4.6...HEAD

## [v0.4.6] - 2025-10-13

[v0.4.6]: https://github.com/nik-rev/docstr/compare/v0.4.5...v0.4.6

## [v0.4.5] - 2025-10-13

[v0.4.5]: https://github.com/nik-rev/docstr/compare/v0.4.4...v0.4.5

## [v0.4.4] - 2025-09-21

[v0.4.4]: https://github.com/nik-rev/docstr/compare/v0.4.3...v0.4.4

## [v0.4.3] - 2025-09-21

[v0.4.3]: https://github.com/nik-rev/docstr/compare/v0.4.2...v0.4.3

## [v0.4.2] - 2025-09-19

[v0.4.2]: https://github.com/nik-rev/docstr/compare/v0.4.1...v0.4.2

## [v0.4.1] - 2025-09-19

[v0.4.1]: https://github.com/nik-rev/docstr/compare/v0.4.0...v0.4.1

### Changed

The comma between the last argument before the first doc comment, and the doc comment itself is now optional

```rs
docstr!(write! w,
      //        ^ this comma was required, but now it is optional
  /// Hello, world!
);
```

## [v0.4.0] - 2025-09-19

[v0.4.0]: https://github.com/nik-rev/docstr/compare/v0.3.0...v0.4.0

### Changed

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

## [v0.3.0] - 2025-09-19

[v0.3.0]: https://github.com/nik-rev/docstr/compare/v0.2.0...v0.3.0

## Changed

- Improved error messages

## Removed

- A few invalid cases which were previously accepted are now compile errors

## [v0.2.0] - 2025-09-19

[v0.2.0]: https://github.com/nik-rev/docstr/compare/v0.1.1...v0.2.0

### Changed

- Require at least 1 doc comment inside of `docstr!`

## [v0.1.1] - 2025-09-19

[v0.1.1]: https://github.com/nik-rev/docstr/compare/v0.1.0...v0.1.1

## [v0.1.0] - 2025-09-18

[v0.1.0]: https://github.com/nik-rev/docstr/releases/v0.1.0
