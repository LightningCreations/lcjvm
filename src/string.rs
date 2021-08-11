macro_rules! debug_unreachable {
    () => {{
        #[cfg(any(debug_assertions, feature = "paranoid"))]
        {
            unreachable!()
        }
    }};
    ($($tt:tt)+) => {{
        #[cfg(any(debug_assertions, feature = "paranoid"))]
        {
            unreachable!($($tt)+)
        }
    }};
}

use std::{
    borrow::{Borrow, BorrowMut, Cow},
    fmt::{Display, Formatter, Write},
    iter::{Copied, Enumerate, FusedIterator},
    ops::{Deref, DerefMut},
};

/// Represents a Slice of a String encoded in [Modified UTF-8](https://docs.oracle.com/en/java/javase/15/docs/api/java.base/java/io/DataInput.html#modified-utf-8).
///
/// This type is analogous to the str primitive type. As such, it is always required to be a valid Modified UTF-8 string
/// The two distinctions are that a ModifiedUtf8Str will never contain an embedded NUL byte, and characters will never exceed 3 bytes
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JStr([u8]);

mod sealed {
    pub trait Sealed {}
}

pub struct Bytes<'a>(Copied<std::slice::Iter<'a, u8>>);

impl<'a> Iterator for Bytes<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn count(self) -> usize {
        self.0.count()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n)
    }

    fn last(self) -> Option<Self::Item> {
        self.0.last()
    }
}

impl<'a> ExactSizeIterator for Bytes<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> FusedIterator for Bytes<'a> {}

impl<'a> DoubleEndedIterator for Bytes<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n)
    }
}

pub struct JCharIndecies<'a>(Enumerate<Bytes<'a>>);

#[allow(unreachable_code)]
impl<'a> Iterator for JCharIndecies<'a> {
    type Item = (usize, u16);

    fn next(&mut self) -> Option<(usize, u16)> {
        let (n, first) = self.0.next()?;

        if first & 0x80 == 0 {
            Some((n, first as u16))
        } else if first & 0xe0 == 0xc0 {
            let (_, next) = self.0.next().unwrap_or_else(|| {
                debug_unreachable!("Unexpected EOF in JStr");
                // SAFETY:
                // ModifiedUtf8Str is valid Modified Utf-8, so a multibyte character will have sufficient continuation bytes
                // Thus this line will never execute because self.0.next() will return Some.
                unsafe { core::hint::unreachable_unchecked() }
            });
            Some((n, ((first & 0x1f) as u16) << 6 | (next & 0x3f) as u16))
        } else {
            let (_, next1) = self.0.next().unwrap_or_else(|| {
                debug_unreachable!("Unexpected EOF in JStr");
                // SAFETY:
                // ModifiedUtf8Str is valid Modified Utf-8, so a multibyte character will have sufficient continuation bytes
                // Thus this line will never execute because self.0.next() will return Some.
                unsafe { core::hint::unreachable_unchecked() }
            });
            let (_, next2) = self.0.next().unwrap_or_else(|| {
                debug_unreachable!("Unexpected EOF in JStr");
                // SAFETY:
                // ModifiedUtf8Str is valid Modified Utf-8, so a multibyte character will have sufficient continuation bytes
                // Thus this line will never execute because self.0.next() will return Some.
                unsafe { core::hint::unreachable_unchecked() }
            });

            Some((
                n,
                ((first & 0x1f) as u16) << 12
                    | ((next1 & 0x3f) as u16) << 6
                    | (next2 & 0x3f) as u16,
            ))
        }
    }
}

///
/// An iterator over a &ModifiedUtf8Str that produces u16s that are valid java characters
pub struct JChars<'a>(Bytes<'a>);

#[allow(unreachable_code)]
impl<'a> Iterator for JChars<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        let first = self.0.next()? as u16;

        if first & 0x80 == 0 {
            return Some(first); // Ascii
        } else if first & 0xe0 == 0xc0 {
            let next = self.0.next().unwrap_or_else(|| {
                debug_unreachable!("Unexpected EOF in JStr");
                // SAFETY:
                // ModifiedUtf8Str is valid Modified Utf-8, so a multibyte character will have sufficient continuation bytes
                // Thus this line will never execute because self.0.next() will return Some.
                unsafe { core::hint::unreachable_unchecked() }
            }) as u16;

            Some((first & 0x1f) << 6 | (next & 0x3f))
        } else
        /* if first&0xf0==0xe0*/
        {
            let next1 = self.0.next().unwrap_or_else(|| {
                debug_unreachable!("Unexpected EOF in JStr");
                // SAFETY:
                // ModifiedUtf8Str is valid Modified Utf-8, so a multibyte character will have sufficient continuation bytes
                // Thus this line will never execute because self.0.next() will return Some.
                unsafe { core::hint::unreachable_unchecked() }
            }) as u16;
            let next2 = self.0.next().unwrap_or_else(|| {
                debug_unreachable!("Unexpected EOF in JStr");
                // SAFETY:
                // ModifiedUtf8Str is valid Modified Utf-8, so a multibyte character will have sufficient continuation bytes
                // Thus this line will never execute because self.0.next() will return Some.
                unsafe { core::hint::unreachable_unchecked() }
            }) as u16;

            Some((first & 0xf) << 12 | (next1 & 0x3f) << 6 | (next2 & 0x3f))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Each byte can be 1-3 jchars, so the lower bound is /3
        (self.0.len() / 3, Some(self.0.len()))
    }
}

impl<'a> FusedIterator for JChars<'a> {}

pub struct Chars<'a>(JChars<'a>);

impl<'a> Iterator for Chars<'a> {
    type Item = char;
    #[allow(unreachable_code)]
    fn next(&mut self) -> Option<char> {
        let mut val = self.0.next()? as u32;
        if let 0xd800..=0xdbff = val {
            val = 0x10000
                + ((val & 0x3ff) << 10)
                + self.0.next().unwrap_or_else(|| {
                    debug_unreachable!("Unexpected EOF in JStr");
                    // SAFETY:
                    // ModifiedUtf8Str is valid Modified Utf-8, so a multibyte character will have sufficient continuation bytes
                    // Thus this line will never execute because self.0.next() will return Some.
                    unsafe { core::hint::unreachable_unchecked() }
                }) as u32
                & 0x3fff;
        }

        Some(<char>::from_u32(val).unwrap())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lo, hi) = self.0.size_hint();

        (lo / 2, hi)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ModifiedUtf8Error {
    pos: usize,
    len: Option<usize>,
}

impl ModifiedUtf8Error {
    pub fn valid_up_to(&self) -> usize {
        self.pos
    }

    pub fn error_len(&self) -> Option<usize> {
        self.len
    }
}

fn validate_modified_utf8(x: &[u8]) -> Result<(), ModifiedUtf8Error> {
    let mut iter = x.into_iter().enumerate();
    let mut pair_start = None;
    while let Some((pos, b)) = iter.next() {
        if *b == 0 {
            if let Some((_, pos)) = pair_start {
                return Err(ModifiedUtf8Error { pos, len: Some(3) }); // Ensure Erroneous unpaired surrogates are detected first
            }
            return Err(ModifiedUtf8Error { pos, len: Some(1) });
        } else if *b & 0xc0 == 0x80 {
            if let Some((_, pos)) = pair_start {
                return Err(ModifiedUtf8Error { pos, len: Some(3) }); // Same here
            }
            return Err(ModifiedUtf8Error { pos, len: Some(1) });
        } else if *b & 0xe0 == 0xc0 {
            if let Some((_, pos)) = pair_start {
                return Err(ModifiedUtf8Error { pos, len: Some(3) });
            }
            let (_, cont) = iter.next().ok_or(ModifiedUtf8Error {
                pos: pos + 1,
                len: None,
            })?;
            if *cont & 0xc0 != 0x80 {
                return Err(ModifiedUtf8Error { pos, len: Some(2) });
            }
        } else if *b & 0xf0 == 0xe0 {
            let (_, cont1) = iter.next().ok_or(ModifiedUtf8Error {
                pos: pos + 1,
                len: None,
            })?;
            if *cont1 & 0xc0 != 0x80 {
                return Err(ModifiedUtf8Error { pos, len: Some(3) });
            }
            let (_, cont2) = iter.next().ok_or(ModifiedUtf8Error {
                pos: pos + 2,
                len: None,
            })?;
            if *cont2 & 0xc0 != 0x80 {
                return Err(ModifiedUtf8Error { pos, len: Some(3) });
            }
            let val = ((*b & 0xf) as u16) << 12
                | ((*cont1 & 0x3f) as u16) << 6
                | ((*cont2 & 0x3f) as u16);
            if let 0xd800..=0xdbff = val {
                if let Some((_, pos)) = pair_start {
                    return Err(ModifiedUtf8Error { pos, len: Some(3) });
                }
                pair_start = Some((val, pos));
            } else if let 0xdc00..=0xdfff = val {
                if let Some((high, pos)) = pair_start {
                    let val = 0x10000 + (((high as u32) & 0x3fff) << 10) + ((val as u32) & 0x3fff);
                    if <char>::from_u32(val).is_none() {
                        return Err(ModifiedUtf8Error { pos, len: Some(6) });
                    }
                }
            } else if let Some((_, pos)) = pair_start {
                return Err(ModifiedUtf8Error { pos, len: Some(3) });
            }
        } else if *b & 0xf0 == 0xf0 {
            return Err(ModifiedUtf8Error { pos, len: Some(1) });
        } else if let Some((_, pos)) = pair_start {
            return Err(ModifiedUtf8Error { pos, len: Some(3) });
        }
    }

    Ok(())
}

impl JStr {
    pub fn from_str(x: &str) -> Result<&Self, ModifiedUtf8Error> {
        Self::from_modified_utf8(x.as_bytes())
    }
    pub fn from_modified_utf8(x: &[u8]) -> Result<&Self, ModifiedUtf8Error> {
        validate_modified_utf8(x)?;
        // SAFETY:
        // validation performed above, so x is valid Modified UTF-8
        Ok(unsafe { Self::from_modified_utf8_unchecked(x) })
    }

    pub fn from_modified_utf8_mut(x: &mut [u8]) -> Result<&mut Self, ModifiedUtf8Error> {
        validate_modified_utf8(x)?;
        // SAFETY:
        // Validation performed above
        Ok(unsafe { Self::from_modified_utf8_unchecked_mut(x) })
    }

    ///
    /// Converts a byte slice into a ModifiedUtf8Str without validation
    /// x is required to be a valid [Modified Utf-8 string]()
    ///
    pub unsafe fn from_modified_utf8_unchecked(x: &[u8]) -> &Self {
        // SAFETY:
        // x came from a reference so thus is valid. Lifetime of return value is tied to lifetime of x
        // The Safety Invariant is upheld by the precondition of the function
        unsafe { &*(x as *const [u8] as *const JStr) }
    }

    pub unsafe fn from_modified_utf8_unchecked_mut(x: &mut [u8]) -> &mut Self {
        // SAFETY:
        // x came from a reference so thus is valid. Lifetime of return value is tied to lifetime of x
        // The Safety Invariant is upheld by the precondition of the function
        unsafe { &mut *(x as *mut [u8] as *mut JStr) }
    }

    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub const fn len(&self) -> usize {
        self.0.len()
    }

    pub const fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.0.as_mut_ptr()
    }

    /// SAFETY:
    /// The caller must ensure that invalid Modified UTF-8 strings are not written to this function
    pub unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }

    pub fn bytes(&self) -> Bytes {
        Bytes(self.0.iter().copied())
    }

    pub fn chars(&self) -> Chars {
        Chars(self.jchars())
    }

    pub fn jchars(&self) -> JChars {
        JChars(self.bytes())
    }

    pub fn is_ascii(&self) -> bool {
        self.bytes().all(|b| b < 0x80)
    }

    pub fn make_ascii_lowercase(&mut self) {
        for b in &mut self.0 {
            if 0x40 < *b && *b < 0x5b {
                *b |= 0x20;
            }
        }
    }

    pub fn make_ascii_uppercase(&mut self) {
        for b in &mut self.0 {
            if 0x60 < *b && *b < 0x7b {
                *b &= !0x20;
            }
        }
    }

    pub fn encode_char(c: char, bytes: &mut [u8; 6]) -> &JStr {
        let x = c as u32;
        if x < 0x80 {
            bytes[0] = x as u8;
            unsafe { Self::from_modified_utf8_unchecked(&bytes[..1]) }
        } else if x < 0x800 {
            bytes[0] = 0xc0 | ((x >> 6) & 0x1f) as u8;
            bytes[1] = 0x80 | (x & 0x3f) as u8;
            unsafe { Self::from_modified_utf8_unchecked(&bytes[..2]) }
        } else if x < 0x80000 {
            bytes[0] = 0xe0 | ((x >> 12) & 0xf) as u8;
            bytes[1] = 0x80 | ((x >> 6) & 0x3f) as u8;
            bytes[2] = 0x80 | (x & 0x3f) as u8;
            unsafe { Self::from_modified_utf8_unchecked(&bytes[..3]) }
        } else {
            let mut u16 @ [h, w] = [0; 2];
            c.encode_utf16(&mut u16);
            bytes[0] = 0xe0 | ((h >> 12) & 0xf) as u8;
            bytes[1] = 0x80 | ((h >> 6) & 0x3f) as u8;
            bytes[2] = 0x80 | (h & 0x3f) as u8;
            bytes[3] = 0xe0 | ((w >> 12) & 0xf) as u8;
            bytes[4] = 0x80 | ((w >> 6) & 0x3f) as u8;
            bytes[5] = 0x80 | (w & 0x3f) as u8;
            unsafe { Self::from_modified_utf8_unchecked(&bytes[..3]) }
        }
    }

    pub fn into_str(&self) -> Cow<str> {
        match std::str::from_utf8(&self.0) {
            Ok(s) => Cow::Borrowed(s),
            Err(_) => Cow::Owned(self.to_string()),
        }
    }

    pub fn from_utf8_str(st: &str) -> Cow<JStr> {
        match Self::from_str(st) {
            Ok(st) => Cow::Borrowed(st),
            Err(e) => {
                let mut bytes = st.as_bytes();
                let mut vec = Vec::new();
                let (prefix, rest) = bytes.split_at(e.valid_up_to());
                vec.extend_from_slice(prefix);
                bytes = rest;
                loop {
                    // SAFETY:
                    // bytes is a valid UTF-8 string
                    let c = unsafe { std::str::from_utf8_unchecked(bytes) }
                        .chars()
                        .next()
                        .unwrap();
                    bytes = &bytes[..c.len_utf8()];
                    if c as u32 == 0 {
                        vec.extend_from_slice(&[0xc0, 0x80]); // Modified Utf-8 uses 2 bytes to encode the Null Character
                    } else {
                        let mut utf16 = [0u16; 2];
                        let utf16 = c.encode_utf16(&mut utf16);
                        for &mut u in utf16 {
                            if u < 0x80 {
                                vec.push(u as u8);
                            } else if u < 0x800 {
                                vec.extend_from_slice(&[
                                    ((u >> 6) | 0xc0) as u8,
                                    (u & 0x3f | 0x80) as u8,
                                ]);
                            } else {
                                vec.extend_from_slice(&[
                                    ((u >> 12) | 0xe0) as u8,
                                    ((u >> 6) & 0x3f | 0x80) as u8,
                                    (u & 0x3f | 0x80) as u8,
                                ])
                            }
                        }
                    }

                    if let Err(e) = self::validate_modified_utf8(bytes) {
                        let (prefix, rest) = bytes.split_at(e.valid_up_to());
                        vec.extend_from_slice(prefix);
                        bytes = rest;
                    } else {
                        vec.extend_from_slice(bytes);
                        break;
                    }
                }

                // SAFETY:
                // We have validated the contents inserted into `vec`
                Cow::Owned(JString(vec))
            }
        }
    }

    pub fn escape_debug(&self) -> EscapeDebug {}
}

impl AsRef<[u8]> for JStr {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<JStr> for JStr {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<JStr> for JStr {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl Display for JStr {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        let mut inner = self.as_bytes();
        loop {
            let st = std::str::from_utf8(inner);
            match st {
                Ok(s) => break s.fmt(fmt),
                Err(e) => {
                    let (prefix, rest) = inner.split_at(e.valid_up_to());
                    // SAFETY:
                    // The prefix of an array passed to from_utf8 that returns an error, of length `e.valid_up_to()` is valid UTF-8
                    unsafe { std::str::from_utf8_unchecked(prefix) }.fmt(fmt)?;
                    if rest[0] == 0xc0 && rest[1] == 0x80 {
                        unsafe { char::from_u32_unchecked(0) }.fmt(fmt)?;
                        inner = &rest[2..];
                    } else {
                        // We have a surrogate pair
                        // Let's decode it
                        let (char, tail) = rest.split_at(6);
                        inner = tail;
                        Chars(JChars(Bytes(char.iter().copied())))
                            .next()
                            .unwrap()
                            .fmt(fmt)?;
                    }
                }
            }
        }
    }
}

impl ::core::fmt::Debug for JStr {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        fmt.write_char('"')?;
        let mut inner = self.as_bytes();
        loop {
            let st = std::str::from_utf8(inner);
            match st {
                Ok(s) => break std::fmt::Display::fmt(&s.escape_debug(), fmt)?,
                Err(e) => {
                    let (prefix, rest) = inner.split_at(e.valid_up_to());
                    // SAFETY:
                    // The prefix of an array passed to from_utf8 that returns an error, of length `e.valid_up_to()` is valid UTF-8
                    std::fmt::Display::fmt(
                        &unsafe { std::str::from_utf8_unchecked(prefix) }.escape_debug(),
                        fmt,
                    )?;
                    if rest[0] == 0xc0 && rest[1] == 0x80 {
                        std::fmt::Display::fmt(
                            &unsafe { char::from_u32_unchecked(0) }.escape_debug(),
                            fmt,
                        )?;
                        inner = &rest[2..];
                    } else {
                        // We have a surrogate pair
                        // Let's decode it
                        let (char, tail) = rest.split_at(6);
                        inner = tail;
                        std::fmt::Display::fmt(
                            &Chars(JChars(Bytes(char.iter().copied())))
                                .next()
                                .unwrap()
                                .escape_debug(),
                            fmt,
                        )?;
                    }
                }
            }
        }
        fmt.write_char('"')
    }
}

pub struct FromModifiedUtf8Error {
    err: ModifiedUtf8Error,
    vec: Vec<u8>,
}

impl FromModifiedUtf8Error {
    pub fn as_bytes(&self) -> &[u8] {
        &self.vec
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.vec
    }

    pub fn modified_utf8_error(&self) -> ModifiedUtf8Error {
        self.err
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JString(Vec<u8>);

impl JString {
    pub fn from_modified_utf8(vec: Vec<u8>) -> Result<Self, FromModifiedUtf8Error> {
        if let Err(err) = self::validate_modified_utf8(&vec) {
            Err(FromModifiedUtf8Error { err, vec })
        } else {
            // SAFETY:
            // bytes is validated above, and thus is valid Modified UTF-8
            Ok(Self(vec))
        }
    }

    pub unsafe fn from_modified_utf8_unchecked(vec: Vec<u8>) -> Self {
        Self(vec)
    }

    pub fn from_boxed_modified_utf8_str(st: Box<JStr>) -> Self {
        Self(Vec::from(unsafe {
            Box::from_raw(Box::into_raw(st) as *mut [u8])
        }))
    }

    pub fn encode_utf16(&self) -> Vec<u16> {
        self.jchars().collect()
    }
}

impl Deref for JString {
    type Target = JStr;

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // ModifiedUtf8String requires that it's content be valid
        unsafe { JStr::from_modified_utf8_unchecked(&self.0) }
    }
}

impl DerefMut for JString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.0.as_mut() as *mut [u8] as *mut Self::Target) }
    }
}

impl AsRef<JStr> for JString {
    fn as_ref(&self) -> &JStr {
        self
    }
}

impl AsMut<JStr> for JString {
    fn as_mut(&mut self) -> &mut JStr {
        self
    }
}

impl Borrow<JStr> for JString {
    fn borrow(&self) -> &JStr {
        self
    }
}

impl BorrowMut<JStr> for JString {
    fn borrow_mut(&mut self) -> &mut JStr {
        self
    }
}

impl ToOwned for JStr {
    type Owned = JString;

    fn to_owned(&self) -> Self::Owned {
        // SAFETY:
        // self is valid Modified UTF-8
        unsafe { JString::from_modified_utf8_unchecked(Vec::from(&self.0)) }
    }
}

impl core::fmt::Debug for JString {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        <JStr as core::fmt::Debug>::fmt(self, f)
    }
}

impl Display for JString {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        <JStr as Display>::fmt(self, f)
    }
}
