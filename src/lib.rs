// This project is licensed under either:
//
// - Apache License, Version 2.0, https://www.apache.org/licenses/LICENSE-2.0)
// - MIT license, https://opensource.org/licenses/MIT)
//
// Copyright 2025 Porter
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// MIT License
// 
// Copyright (c) 2025 Porter
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
//! A Rust library of proc-macros for nuhound error tracing
//! 
//! Rust programmers often find the question mark operator invaluable in extracting values from
//! Result and Option and immediately returning to the calling context in the case of an Err or
//! None. This library provides a trio of proc-macros that simplify the use of the nuhound type in
//! error tracing.
//!
//! With the `disclose` feature enabled, the error messages contain the line number and column
//! number of the source file that originated the error. This functionality is provided by the
//! convert!, examine! and custom! macros. These macros are designed to help simplify error
//! handling in a concise and consistent Rust style in line with the nuhound paradigm.
//!
//! These macros require nuhound v0.2 or later.
//!
//! For a fuller explanation of usage please refer to the nuhound crate v0.2 onwards.
//!

mod scanner;
use proc_macro::TokenStream;
use std::str::Chars;
use std::collections::HashMap;
use scanner::Scanner;

// An array of symmetric character pairs
const PAIRS: [(char, char); 4] = [('(', ')'), ('[', ']'), ('{', '}'), ('<', '>')];

// Scan through characters enclosed between symmetric character pairs
fn process_pairs(scanner: &mut Scanner, pairs: &HashMap<char, char>) {
    let exit = pairs[&scanner.get_current().unwrap()];
    loop {
        match scanner.next() {
            // Ignore '<' when in here to allow for less than situations
            Some(next) if next == '<' => (),
            Some(next) if pairs.contains_key(&next) => {
                process_pairs(scanner, pairs);
            }
            Some(next) if next == exit => {
                break;
            }
            Some(_) => (),
            None => break
        }
    }
}

// Scan through characters placed between double or single quotes remembering
// to ignore escaped quotes.
fn process_quotes(scanner: &mut Scanner) {
    let quote = scanner.get_current().unwrap();
    loop {
        match scanner.next() {
            Some(next) if next == quote && !scanner.is_escaped() => {
                break;
            }
            Some(_) => (),
            None => break
        }
    }
}

// Scan through the character string separating into comma delimited attributes and returning them
// as a vector of strings to the calling context.
fn analyse(char_string: Chars) -> Vec<String> {
    let pairs = HashMap::from(PAIRS);
    let mut scanner = Scanner::new(char_string.collect());
    loop {
        match scanner.next() {
            Some(next) if pairs.contains_key(&next) => {
                process_pairs(&mut scanner, &pairs);
            }
            Some(next) if next == '\'' && !scanner.is_escaped() => {
                process_quotes(&mut scanner);
            }
            Some(next) if next == '"' && !scanner.is_escaped() => {
                process_quotes(&mut scanner);
            }
            Some(next) if next == '|' => {
                if !scanner.is_pipe_valid() {
                    panic!("The pipe character is misplaced. Perhaps you intended to insert a 'closure' \
                           in which case it must be placed between curly brackets.\n\
                           E.g. {{|n| n + 3}}");
                }
            }
            Some(next) if next == ',' => {
                scanner.save_attribute(1);
            }
            Some(_) => (),
            None => break
        }
    }
    scanner.save_attribute(0);
    scanner.get_string_attributes()
}

// The convert builder is used to create a macro that generates Nuhound type errors from any other
// error cause provided that they employ the Error trait. This includes Nuhound errors too.
fn convert_builder(item: String) -> String {
    let attributes = analyse(item.chars());
    if attributes.len() < 2 {
        panic!("Contains insufficient parameters");
    }
    let message = attributes[1..].join(", ");

    format!("
    {0}.report(|reason| {{
        let cause: &dyn ::std::error::Error = &reason;
        #[cfg(not(feature = \"disclose\"))]
        let inform = format!({1});
        #[cfg(feature = \"disclose\")]
        let inform = format!(\"{{0}}:{{1}}:{{2}}: {{3}}\", file!(), line!(), column!(), format!({1}));
        ::nuhound::Nuhound::link(inform, cause)
    }})
    ", attributes[0], message)
}

// The examine builder is used to create a macro that generates Nuhound type errors from other
// Nuhound errors. Unlike the convert builder, the causal error must be a Nuhound type which
// simplifies the generated code after compilation.
fn examine_builder(item: String) -> String {
    let attributes = analyse(item.chars());
    if attributes.len() < 2 {
        panic!("Contains insufficient parameters");
    }
    let message = attributes[1..].join(", ");

    format!("
    {0}.report(|cause| {{
        #[cfg(not(feature = \"disclose\"))]
        let inform = format!({1});
        #[cfg(feature = \"disclose\")]
        let inform = format!(\"{{0}}:{{1}}:{{2}}: {{3}}\", file!(), line!(), column!(), format!({1}));
        ::nuhound::Nuhound::new(inform).caused_by(cause)
    }})
    ", attributes[0], message)
}

// The custom builder is used to create a macro that generates a Nuhound error.
fn custom_builder(item: String) -> String {
    let attributes = analyse(item.chars());
    if attributes.is_empty() {
        panic!("Contains insufficient parameters");
    }
    let message = attributes.join(", ");

    format!("
    {{
        #[cfg(not(feature = \"disclose\"))]
        let inform = format!({0});
        #[cfg(feature = \"disclose\")]
        let inform = format!(\"{{0}}:{{1}}:{{2}}: {{3}}\", file!(), line!(), column!(), format!({0}));
        ::std::result::Result::Err(::nuhound::Nuhound::new(inform))
    }}
    ", message)
}

//  convert macro
/// A macro to prepare a `Nuhound` type error from any error type that implements the Error trait. This
/// also includes Nuhound errors. Resultant errors may be handled using the `?` operator or by simply
/// returning it to the calling context as a `Result::Err` directly.
///
/// The macro creates an error message that may optionally contain the name of the source file and
/// location of the error. This behaviour is enabled by compiling the code with the 'disclose'
/// feature.
///
/// This macro requires either `nuhound::ResultExtension` or `nuhound::OptionExtension` depending on
/// whether the code being checked returns a `Result` or an `Option`.
///
/// # Examples
/// The following example shows how the `convert` macro is used to report an error but still retain
/// the underlying error or errors that can be displayed using the `trace` method.
///
/// ```ignore
/// use nuhound::{Report, ResultExtension, convert};
///
/// fn my_result() -> Report<u32> {
///     let text = "NaN";
///     let value = convert!(text.parse::<u32>(), "Oh dear - '{}' could not be converted to an integer", text)?;
///     Ok(value)
/// }
///
/// fn layer2() -> Report<u32> {
///     let result = convert!(my_result(), "Next level failure")?;
///     Ok(result)
/// }
///
/// fn layer1() -> Report<u32> {
///     let result = convert!(layer2(), "Highest level failure")?;
///     Ok(result)
/// }
///
/// match layer1() {
///     Ok(value) => println!("Value = {value}"),
///     Err(e) => {
///         #[cfg(feature = "disclose")]
///         eprintln!("{}", e.trace());
///         #[cfg(not(feature = "disclose"))]
///         eprintln!("{}", e);
///     },
/// }
///
/// // using 'cargo run --features disclose' will emit the following message:
/// //
/// // 0: examples/nan.rs:16:22: Highest level failure
/// // 1: examples/nan.rs:11:22: Next level failure
/// // 2: examples/nan.rs:6:21: Oh dear - 'NaN' could not be converted to an integer
/// // 3: invalid digit found in string
/// //
/// // using 'cargo run' without the disclose feature will emit the following message:
/// //
/// // Highest level failure
/// //
/// // Notice that the error detail is no longer visible
///```
#[proc_macro]
pub fn convert(item: TokenStream) -> TokenStream {
    convert_builder(item.to_string()).parse().unwrap()
}

//  examine macro
/// A macro to prepare a `Nuhound` type error from previously handled `Nuhound` error(s). Whilst the
/// `convert` macro is completely error type agnostic provided the error handler implements the
/// `Error` trait, the `examine` macro requires much less binary code to implement and hence is
/// more efficient. Resultant errors are handled using the `?` operator by simply returning it to the
/// calling context as a `Result::Err` directly.
///
/// The macro creates an error message that may optionally contain the name of the source file and
/// location of the error. This behaviour is enabled by compiling the code with the `disclose`
/// feature.
///
/// # Examples
/// The following example shows how the `examine` macro is used to report an error but still retain
/// the underlying error or errors that can be displayed using the `trace` method.
///
/// ```ignore
/// use nuhound::{Report, ResultExtension, convert, examine};
///
/// fn my_result() -> Report<u32> {
///     let text = "NaN";
///     // We need to use the convert macro here, because the underlying error is of type
///     // ParseIntError
///     let value = convert!(text.parse::<u32>(), "Oh dear - '{}' could not be converted to an integer", text)?;
///     Ok(value)
/// }
///
/// fn layer2() -> Report<u32> {
///     // The examine macro can be used here because the called context has already converted the
///     // ParseIntError to Nuhound 
///     let result = examine!(my_result(), "Next level failure")?;
///     Ok(result)
/// }
///
/// fn layer1() -> Report<u32> {
///     // The examine macro can be used here because the called context is already Nuhound ready.
///     let result = examine!(layer2(), "Highest level failure")?;
///     Ok(result)
/// }
///
/// match layer1() {
///     Ok(value) => println!("Value = {value}"),
///     Err(e) => {
///         #[cfg(feature = "disclose")]
///         eprintln!("{}", e.trace());
///         #[cfg(not(feature = "disclose"))]
///         eprintln!("{}", e);
///     },
/// }
///
/// // using `cargo run --features disclose` will emit the following message:
/// //
/// // 0: examples/nan.rs:16:22: Highest level failure
/// // 1: examples/nan.rs:11:22: Next level failure
/// // 2: examples/nan.rs:6:21: Oh dear - 'NaN' could not be converted to an integer
/// // 3: invalid digit found in string
/// //
/// // using `cargo run` without the disclose feature will emit the following message:
/// //
/// // Highest level failure
/// //
/// // Notice that the error detail is no longer visible
///```
#[proc_macro]
pub fn examine(item: TokenStream) -> TokenStream {
    examine_builder(item.to_string()).parse().unwrap()
}

//  custom macro
/// A macro to prepare a `Nuhound` type error. Whilst the `convert` and `examine` macros are
/// designed to respond to previously handled errors, the `custom` macro will always generate a
/// `Nuhound` error without any pre-conditions. Resultant errors may be handled using the `?`
/// operator or by simply returning it to the calling context as a `Result::Err` directly. Normal
/// usage of this macro would suggest using it inside a condition block and returning the error to
/// the calling context with `return custom!("My error message");`.
///
/// This macro creates an error message that may optionally contain the name of the source file and
/// location of the error. This behaviour is enabled by compiling the code with the `disclose`
/// feature.
///
/// # Examples
/// The following example shows how the `custom` macro is used in conjunction with the `examine`
/// macro to report an error but still retain the originating error that can be displayed using the
/// `trace` method.
///
/// ```ignore
/// use nuhound::{Report, ResultExtension, examine, custom};
///
/// fn my_result() -> Report<u32> {
///     let value = 99;
///     if value == 99 {
///         return custom!("Oh dear - '{value}' was not expected");
///     }
///     Ok(value)
/// }
///
/// fn layer2() -> Report<u32> {
///     let result = examine!(my_result(), "Next level failure")?;
///     Ok(result)
/// }
///
/// fn layer1() -> Report<u32> {
///     let result = examine!(layer2(), "Highest level failure")?;
///     Ok(result)
/// }
///
/// match layer1() {
///     Ok(value) => println!("Value = {value}"),
///     Err(e) => {
///         #[cfg(feature = "disclose")]
///         eprintln!("{}", e.trace());
///         #[cfg(not(feature = "disclose"))]
///         eprintln!("{}", e);
///     },
/// }
///
/// // using `cargo run --features disclose` will emit the following message:
/// //
/// // 0: examples/custom.rs:18:22: Highest level failure
/// // 1: examples/custom.rs:13:22: Next level failure
/// // 2: examples/custom.rs:7:20: Oh dear - '99' was not expected
/// //
/// // using `cargo run` without the disclose feature will emit the following message:
/// //
/// // Highest level failure
/// //
/// // Notice that the error detail is no longer visible
///```
#[proc_macro]
pub fn custom(item: TokenStream) -> TokenStream {
    custom_builder(item.to_string()).parse().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_builder() {
        const ATTRIBUTES: &str = r##""Oh dear this failed because of {}", text"##;
        let result = custom_builder(ATTRIBUTES.to_string());
        let result_parts: Vec<&str> = result.split("\n")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let required = vec![
            "{",
            "#[cfg(not(feature = \"disclose\"))]",
            "let inform = format!(\"Oh dear this failed because of {}\", text);",
            "#[cfg(feature = \"disclose\")]",
            "let inform = format!(\"{0}:{1}:{2}: {3}\", file!(), line!(), column!(), format!(\"Oh dear this failed because of {}\", text));",
            "::std::result::Result::Err(::nuhound::Nuhound::new(inform))",
            "}",
        ];

        println!("{result_parts:#?}");
        assert_eq!(result_parts, required);
    }

    #[test]
    fn test_examine_builder() {
        const ATTRIBUTES: &str = r##"text.parse::<u32>(), "Oh dear - '{}' could not be converted to an integer", text"##;
        let result = examine_builder(ATTRIBUTES.to_string());
        let result_parts: Vec<&str> = result.split("\n")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let required = vec![
            "text.parse::<u32>().report(|cause| {",
            "#[cfg(not(feature = \"disclose\"))]",
            "let inform = format!(\"Oh dear - '{}' could not be converted to an integer\", text);",
            "#[cfg(feature = \"disclose\")]",
            "let inform = format!(\"{0}:{1}:{2}: {3}\", file!(), line!(), column!(), format!(\"Oh dear - '{}' could not be converted to an integer\", text));",
            "::nuhound::Nuhound::new(inform).caused_by(cause)",
            "})",
        ];
        println!("{result_parts:#?}");
        assert_eq!(result_parts, required);
    }

    #[test]
    fn test_covert_builder() {
        const ATTRIBUTES: &str = r##"text.parse::<u32>(), "Oh dear - '{}' could not be converted to an integer", text"##;
        let result = convert_builder(ATTRIBUTES.to_string());
        let result_parts: Vec<&str> = result.split("\n")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let required = vec![
            "text.parse::<u32>().report(|reason| {",
            "let cause: &dyn ::std::error::Error = &reason;",
            "#[cfg(not(feature = \"disclose\"))]",
            "let inform = format!(\"Oh dear - '{}' could not be converted to an integer\", text);",
            "#[cfg(feature = \"disclose\")]",
            "let inform = format!(\"{0}:{1}:{2}: {3}\", file!(), line!(), column!(), format!(\"Oh dear - '{}' could not be converted to an integer\", text));",
            "::nuhound::Nuhound::link(inform, cause)",
            "})",
        ];
        println!("{result_parts:#?}");
        assert_eq!(result_parts, required);
    }

    #[test]
    fn normal() {
        const ATTRIBUTES: &str = r##"text.parse::<u32>(), 
            "Oh dear - '{}' could not be converted to an integer", 
            text"##;
        let char_string = ATTRIBUTES.chars();
        let required = vec! [
            "text.parse::<u32>()",
            "\"Oh dear - '{}' could not be converted to an integer\"",
            "text",
        ];

        let result = analyse(char_string);
        println!("{result:#?}");
        assert_eq!(result, required);
    }

    #[test]
    fn extended() {
        const ATTRIBUTES: &str = r##" text.parse::<u32, char>(35 < 8), r#"Oh dear - '{}' could, not be converted to an integer"#, text   "##; 
        let char_string = ATTRIBUTES.chars();
        let required = vec! [
            "text.parse::<u32, char>(35 < 8)",
            "r#\"Oh dear - '{}' could, not be converted to an integer\"#",
            "text",
        ];

        let result = analyse(char_string);
        println!("{result:#?}");
        assert_eq!(result, required);
    }
}


