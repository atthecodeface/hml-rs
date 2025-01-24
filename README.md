# hml-rs

A parser for human-readable markup language, which is in large part equally powerful with XML.

An example file is:

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

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
hml-rs = "0.3.0"
```

## Releases

Release notes are available in [RELEASES.md](RELEASES.md).

## License

Licensed under either of

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
