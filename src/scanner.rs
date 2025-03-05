// This project is licensed under either:
//
// - Apache License, Version 2.0, https://www.apache.org/licenses/LICENSE-2.0)
// - MIT license https://opensource.org/licenses/MIT)
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
//
//! A module that scans through a vector of chars

// A structure that holds the start and end position of detected comma delimited attributes
struct Attribute {
    start: usize,
    end: usize,
}

// A structure containing the working components of the scanner
pub(crate) struct Scanner {
    char_string: Vec<char>,
    length: usize,
    index: usize,
    attributes: Vec<Attribute>,
    mark: usize,
}

impl Scanner {
    // Create a new Scanner object from a vector of chars
    pub(crate) fn new(char_string: Vec<char>) -> Self {
        let length = char_string.len();
        Self {
            char_string,
            length,
            index: 0,
            attributes: Vec::new(),
            mark: 0,
        }
    }

    // Increment to the next character position
    pub(crate) fn next(&mut self) -> Option<char> {
        if self.index < self.length {
            self.index += 1;
            Some(self.char_string[self.index - 1])
        } else {
            None
        }
    }

    // return the character at the cursor position if there is on otherwise return None
    pub(crate) fn get_current(&self) -> Option<char> {
        if self.index < self.length {
            Some(self.char_string[self.index - 1])
        } else {
            None
        }
    }

    // save the start and end position of a detected attribute. The attribute can be shortened from
    // the right hand side to avoid (say) including the comma delimiter.
    pub(crate) fn save_attribute(&mut self, rshorten: usize) {
        let attribute = Attribute {
            start: self.mark,
            end: self.index - rshorten,
        };
        self.mark = self.index;
        self.attributes.push(attribute);
    }

    // Get and return a vector of attributes as String types
    pub(crate) fn get_string_attributes(&self) -> Vec<String> {
        let mut output = Vec::new();
        for attribute in &self.attributes {
            let attr: String = self.char_string[attribute.start..attribute.end].into_iter().collect();
            output.push(attr.trim().to_string())
        }
        output
    }

    // Check that a detected pipe character '|' is not at the start of a character string. This
    // would indicate invalid usage
    pub(crate) fn is_pipe_valid(&self) -> bool {
        let mut pointer = self.index - 1;
        while pointer > self.mark {
            if !self.char_string[pointer].is_whitespace() {
                return true;
            }
            pointer -= 1;
        }
        false
    }

    // Determine whether a quote character has been escaped
    pub(crate) fn is_escaped(&self) -> bool {
        if self.index == 0 {
            false
        } else {
            self.char_string[self.index - 1] == '\\'
        }
    }
}

