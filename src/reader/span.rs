/*a Copyright

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

  http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

@file    span.rs
@brief   Span used within a reader consisting of 2 `Position`s
 */

//a Imports
use crate::reader::Position;

//a Span
//tp Span
/// The [Span] type is used in the [crate::reader::Error] type, and
/// rather than have that type be generic on a [Span]-trait type a
/// fixed-but-generic approach is taken.
///
/// For a simple span type the content
#[derive(Copy, Clone, Debug)]
pub struct Span<P: Position> {
    start: P,
    end: P,
}

//ip Span
impl<P: Position> Span<P> {
    //fp new_at
    /// Create a new span starting and ending at a position
    pub fn new_at(posn: &P) -> Self {
        Self {
            start: *posn,
            end: *posn,
        }
    }

    //cp end_at
    /// Extend self to an end position
    pub fn end_at(mut self, posn: &P) -> Self {
        self.end = *posn;
        self
    }

    //mp start
    /// Borrow the start position of the span
    pub fn start(&self) -> &P {
        &self.start
    }

    //mp end
    /// Borrow the end position of the span
    pub fn end(&self) -> &P {
        &self.end
    }

    //zz All done
}

//ip Display for Span
impl<P: Position> std::fmt::Display for Span<P> {
    //mp fmt
    /// Format for humans
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}->{}", self.start, self.end)
    }
}

//ip crate::markup::Span for Span
impl<P: Position> crate::markup::Span for Span<P> {}
