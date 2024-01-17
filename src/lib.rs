// Copyright (C) 2023 Intel Corporation
// SPDX-License-Identifier: Apache-2.0

//! Raw C String conversion and conversion trait
//!
//! If you want constant C strings, use `c"Hello, World"` as
//! [recently stabilized](https://github.com/rust-lang/rust/pull/117472) instead

#![deny(clippy::unwrap_used)]

use anyhow::{bail, Result};
use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::{CStr, CString},
};

struct RawCStrs(RefCell<HashMap<String, *mut i8>>);

impl Drop for RawCStrs {
    fn drop(&mut self) {
        self.0.borrow_mut().iter_mut().for_each(|(_, c)| unsafe {
            drop(CString::from_raw((*c) as *mut u8));
        });
        self.0.borrow_mut().clear();
    }
}

thread_local! {
    static RAW_CSTRS: RawCStrs = RawCStrs(RefCell::new(HashMap::new()));
}

/// Create a constant raw C string as a `*mut i8` from a Rust string reference. C Strings are cached,
/// and creating the same string twice will cost zero additional memory. This is useful when calling
/// C APIs that take a string as an argument, particularly when the string can't be known at compile
/// time, although this function is also efficient in space (but not time) when a constant string
/// is known. For compile-time constants, you can use `c_str!`.
///
/// # Safety
///
/// - Do *not* use [`String::from_raw_parts`] to convert the pointer back to a [`String`]. This
///   may cause a double free because the [`String`] will take ownership of the pointer. Use
///   [`CStr::from_ptr`] instead, and convert to a string with
///   `.to_str().expect("...").to_owned()` instead.
///
pub fn raw_cstr<S>(str: S) -> Result<*mut i8>
where
    S: AsRef<str>,
{
    RAW_CSTRS.with(|rc| {
        let mut raw_cstrs_map = rc.0.borrow_mut();
        let saved = raw_cstrs_map.get(str.as_ref());

        if let Some(saved) = saved {
            Ok(*saved)
        } else {
            let raw = CString::new(str.as_ref())?.into_raw() as *mut i8;
            raw_cstrs_map.insert(str.as_ref().to_string(), raw);
            Ok(raw)
        }
    })
}

/// A type that can be converted to a raw C string
pub trait AsRawCstr {
    /// Get a type as a raw C string
    fn as_raw_cstr(&self) -> Result<*mut i8>;
}

impl AsRawCstr for &'static [u8] {
    /// Get a static slice as a raw C string. Useful for interfaces.
    fn as_raw_cstr(&self) -> Result<*mut i8> {
        if self.last().is_some_and(|l| *l == 0) {
            Ok(self.as_ptr() as *const i8 as *mut i8)
        } else {
            bail!("Empty slice or last element is nonzero: {:?}", self);
        }
    }
}

impl AsRawCstr for *mut i8 {
    fn as_raw_cstr(&self) -> Result<*mut i8> {
        Ok(*self)
    }
}

impl AsRawCstr for &str {
    fn as_raw_cstr(&self) -> Result<*mut i8> {
        raw_cstr(self)
    }
}

impl AsRawCstr for String {
    fn as_raw_cstr(&self) -> Result<*mut i8> {
        raw_cstr(self)
    }
}

impl AsRawCstr for CString {
    fn as_raw_cstr(&self) -> Result<*mut i8> {
        // Make a copy of the string so that we can return a pointer to it
        raw_cstr(self.to_str()?)
    }
}

impl AsRawCstr for CStr {
    fn as_raw_cstr(&self) -> Result<*mut i8> {
        // Make a copy of the string so that we can return a pointer to it
        raw_cstr(self.to_str()?)
    }
}

impl AsRawCstr for &'static CStr {
    fn as_raw_cstr(&self) -> Result<*mut i8> {
        // No need to copy for static lifetime CStrs because the pointer
        // lifetime is also static
        Ok(self.as_ptr() as *mut i8)
    }
}
