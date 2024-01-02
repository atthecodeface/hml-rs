# Release 0.3.0 (2023-12-02)

- Removed the lexer, to use lexer_rs instead; this reduces code to
  support, and improves the error reporting capability; it is a
  significant change to the invocation of the HML parsing

- Added mapping of markup events to xml_rs events

**Contributors**: @atthecodeface

# Release 0.2.0 (2023-11-23)

- Changed borrow_*_str in namespace to *_str, with the user providing a default

- Removed 'new' from namespace_stack; use default instead

**Contributors**: @atthecodeface

# Release 0.1.4 (2021-06-23)

- Added documentation for the library and some simple code examples

**Contributors**: @atthecodeface

# Release 0.1.3 (2021-06-23)

- Bezier arcs and rounded corners now use
   a better approach using a matched polynomial
   to derive 'lambda'; this is now tested in
   regression to a good degree of accuracy.

**Contributors**: @atthecodeface

# Release 0.1.0 (2021-06-22)

- Publishing on crates.io for the first time

**Contributors**: @atthecodeface

