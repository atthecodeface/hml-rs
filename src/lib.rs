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

@file    lib.rs
@brief   Markup library
 */

//a Documentation
#![warn(missing_docs)]
#![warn(missing_doc_code_examples)]
/*!

# Markup library

This library provides for markup language stream reading and writing,
using for example XML. It also provides an alternative form, HML
(human-readable markup language). All supported markup languages
provide for structured documents whose content is Unicode (and
therefore can be represented as Rust [String]s).

It provides a simple name and namespace system, which utilizes vectors
of strings that may be element names, attribute names, or URIs. A
markup namespaced-name can then be referred to by internal
identifiers, rather than pointers to strings.

Using the namespace system, the library provides for markup handling,
particularly streams of markup events. A document can be parsed from a
stream, with a markup reader producing the required stream of markup
events; the events themselves do not contain string references for the
element names, namespaces, and attribute names; rather they use the
namespace identifiers.

## HML - Human Markup Language

HML is a human-read/writeable markup language that provides a means
for structured data representation, encompassing most of the
capabilities of XML.


### HML Elements

HML uses one or more '#' symbols followed by a string to introduce a
new element; the number of '#' symbols indicating the depth of the
element. Elements can have attributes, which are always of the form
'<name>="<data>"'. After all the attributes for an element are
provided element contents - more elements, for example, can be
included. These elements, if they are content will have a depth of one
more than their parent.

A simple example would be

```text
#shopping
##meat  ###beef type="steak" weight="400g"
##fruit ###banana number="4" ###apple number="6" ###orange number="4" ###grapefruit number="1"
```

### HML element character content

Element character content data can be provided too: this is content of an
element that is Unicode text. In HML this must be presented as a
quoted string: quoted strings can simply be "<text>"; standard Rust
escapes '\n', '\x52' '\u{1234D};', '\\' will be expanded as per-Rust;
a raw string can be used with r"<text>", which has no expansion
performed. These single-quoted strings *must not* include any newlines
- i.e. they must appear on a single line in the HML
document. Multi-line content can be generated using #"..."#,
#"""..."""#, or raw variants r#"..."#.

Simple example data might be:

```text
#shopping
##meat  ###beef type="steak" weight="400g" "Use for spaghetti bolognese"
##fruit ###banana number="4" #"For banana splits \u{1f601}
May also need more ice cream "#
 ###apple number="6" r"Use for kid's breakfasts \ lunches"
```




HML provides for comments: these are introduced with ';', and the comment runs to the end of the line.
Comments are *not* permitted between attributes or an element and its attributes.

With a deeply nested document there would be a large number of '#' in
front of each tag; this is clearly undesirable. HML provides for boxed
elements to improve read/writeability: an element name can be suffixed
by '{', with the element requiring a corresponding closing tag
suffixed by '}' - the content tags within the element are then
expected to have a tag depth of just one '#'.

### HML database example

A simple example HML document for a DVD database might be:

```text
; The toplevel element is the library, which contains 'dvd' elements
; each dvd element should have a title and other data, and contents of director(s), producer(s) and actor(s)
; an actor should be named and have a part
#library
##dvd title="Wizard of Oz" release_date="15 Aug 1939" running_time="101"
###director name="Victor Fleming"
###actor name="Judy Garland"       part="Dorothy"
###actor name="Frank Morgan"       part="The Wizard"
###actor name="Ray Bolger"         part="Scarecrow"
###actor name="Bret Lahr"          part="Cowardly Lion"
###actor name="Jack Haley"         part="Tin Man"
###actor name="Margaret Hamilton"  part="Wicked Witch of the West"
###actor name="Billie Burke"       part="Glinda the Good Witch of the North"

##dvd title="Gone With the Wind" release_date="15 Dec 1939" running_time="221"
###director name="Victor Fleming"
###actor name="Clark Gable"         part="Rhett Butler"
###actor name="Vivien Leigh"        part="Scarlett O'Hara"
###actor name="Leslie Howard"       part="Ashley Wilkes"
###actor name="Olivia de Haviland"  part="Melanie Wilkes"

##dvd{ title="Mr. Smith Goes to Washington" release_data="17 Oct 1939" running_time="125"
#director name="Frank Capra"
#actor    name="Jean Arthur"    part="Clarissa Saunders"
#actor    name="James Stewart"  part="Jefferson Smith"
##dvd}
```

### HML document example

A simple example HML document is:

```text
#html
##head{
#title "A great document"
#link  rel="stylesheet" href="fantastic.css"
##head}
##body{
#h1 class="header" "A good section"
#p r#"""
This is the content of the document.
"""#
##body}
```



# HML reference

## HML document grammar

HML has a concept of depth, specified here usually with 'N'. The
grammar productions include some of <name>[N], and often this requires
N consecutive '#' characters as the introduction to the element.

```text
Document := <Misc[1]>* Element[1] <Misc[1]>*

Misc[N] := <Comment> | <ProcessingInstruction[N]> | <Declaration[N]>

Comment := ';' (<Char> - <Newline>)*

ProcessingInstruction[N] := '#'{N} '?' <Name> [<QuotedString>]

Declaration[N] := '#'{N} '!'<Type> <Name> [<QuotedString>] [DeclarationContent[N+1]*

DeclarationContent[N] := Comment | ProcessingInstruction[N] | <Declaration[N]>

Element[N] := ElementTag[N] ElementContent[N+1]*
            | ElementTagBoxOpen[N] ElementContent[1]* ElementTagBoxClose[N] (matching name)

ElementTag[N] := '#'{N} [<Name>:]<Name> [Attributes]

ElementTagBoxOpen[N]  := '#'{N} [<Name>:]<Name> '{' [Attributes]

ElementTagBoxClose[N] := '#'{N} [<Name>:]<Name> '}'

ElementContent[N] := Comment | ProcessingInstruction[N] | Element[N] | QuotedString

QuotedString :=   '"'     <EscapedContent without newlines> '"'
              |  'r"'     <Content without newlines> '"'
              |  '#"{M}'  <EscapedContent> '"{M}#'
              |  'r#"{M}' <Content> '"{M}#'

```

## HML quoted string notes

A raw quoted string is not parsed for escape sequences.

Multiline strings require #" or r#", with one or more '#' characters,
and terminate with #" with the same number of # characters.

Escapes supported are as per Rust:

```text
\0 => nul (unicode 0)
\t => tab (unicode 9)
\n => newline (unicode 10)
\r => carriage return (unicode 13)
\" => quotes (unicode 34)
\' => apostrophe (unicode 39)
\\ => backslash (unicode 92)
\xXX => XX as unicode hexadecimal (0 to 0x7f)
\u{XXXXXX}  => XXXXXX as unicode hexadecimal (24-bit value, fewer than 6 X are permitted)
```

!*/

pub mod escape;
pub mod hml;

// Expose names::{NSNameId, NSPrefixId, NSUriId, NSMap};
//        names::{Namespace, NamespaceStack, Name, Attribute, Attributes, Tag};
pub mod names;

// Expose markup::Span, Error, Result, Event, EventType
pub mod markup;

// Expose hml_reader::{Position, Character, Error, Reader, Span, Result}
pub mod reader;

// Expose hml_reader::{Lexer, Parser, ReaderError}
pub mod hml_reader;

mod implementations;
pub use implementations::string;

/*
mod types;
mod utils;

pub mod writer;
 */
