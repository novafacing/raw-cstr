# Raw CStr

Utilities for working with raw C strings from Rust. This primarily is intended to
provide a constant-like C-string at runtime by using a thread local cache. Its usage is
extremely niche, and is intended to be used by [tsffs](https://github.com/intel/tsffs)
and other crates which compile to cdylibs which are loaded by non-rust programs.