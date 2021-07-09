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

@file    traits.rs
@brief   Reader traits for the markup library
 */

//a Position trait
//tt Position
pub trait Position : Clone + Copy + std::fmt::Debug + std::fmt::Display {
    fn none() -> Self;
}

//tt Character
pub trait Character : Clone + Copy + std::fmt::Debug + std::fmt::Display {
    fn is_eof(&self)     -> bool;
    fn is_not_rdy(&self) -> bool;
    fn as_char(&self)    -> Option<char>;
}

//tt Reader
pub trait Reader  : std::fmt::Debug {
    type Position : Position;
    type Char     : Character;
    type Error    : std::error::Error + 'static;
    fn next_char(&mut self) -> std::result::Result<Self::Char, Self::Error>;
    fn borrow_pos(&self) -> &Self::Position;
    fn fmt_context(&self, f: &mut std::fmt::Formatter, start:&Self::Position, end:&Self::Position) -> std::fmt::Result ;
}


