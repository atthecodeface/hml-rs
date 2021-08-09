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

@file    implementations.rs
@brief   Implementations of traits
 */

//a Documentation
/*!

# Implementations of [Reader]s etc

This module currently includes only a [Reader] implementation for
[String]. This permits a string to be read from a file and used in
parsers. The [Reader] implementation provided will display a proper
context for errors (similar to the Rust compiler).

!*/

//a Imports

//a Exports
pub mod string;
