# proc\_nuhound
A library of Rust proc macros for nuhound error tracing.

Rust programmers often find the question mark operator invaluable in extracting values from
Result and Option and immediately returning to the calling context in the case of an Err or
None. This library provides a trio of proc macros that simplify the use of the nuhound type in
error tracing.

With the `disclose` feature enabled, the error messages contain the line number and column
number of the source file that originated the error. This functionality is provided by the
convert!, examine! and custom! macros. These macros are designed to help with the
simplification of error handling in a concise consistent Rust style in line with the nuhound
paradigm.

These macros require nuhound v0.2 or later.

For a fuller explantion of usage please refer to nuhound v0.2 onwards.

## License

This project is licensed under either:

- Apache License, Version 2.0, ([LICENSE.APACHE-2.0](LICENSE.apache-2.0) or
   https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE.MIT](LICENSE.MIT) or
   https://opensource.org/licenses/MIT)
