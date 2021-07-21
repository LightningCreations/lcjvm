use std::{
    borrow::{Borrow, BorrowMut, Cow},
    fmt::{Display, Formatter, Write},
    iter::{Copied, FusedIterator},
    ops::{Deref, DerefMut},
};

/// Represents a Slice of a String encoded in [Modified UTF-8](https://docs.oracle.com/en/java/javase/15/docs/api/java.base/java/io/DataInput.html#modified-utf-8).
///
/// This type is analogous to the str primitive type. As such, it is always required to be a valid Modified UTF-8 string
/// The two distinctions are that a ModifiedUtf8Str will never contain an embedded NUL byte, and characters will never exceed 3 bytes
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModifiedUtf8Str([u8]);

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

impl<'a> ExactSizeIterator for Bytes<'a> {}

impl<'a> FusedIterator for Bytes<'a> {}

impl<'a> DoubleEndedIterator for Bytes<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n)
    }
}

///
/// An iterator over a &ModifiedUtf8Str that produces u16s that are valid java characters
pub struct JChars<'a>(Bytes<'a>);

impl<'a> Iterator for JChars<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        let first = self.0.next()? as u16;

        if first & 0x80 == 0 {
            return Some(first); // Ascii
        } else if first & 0xe0 == 0xc0 {
            let next = self.0.next().unwrap_or_else(|| {
                debug_assert!(false, "Invalid Modified UTF-8 in ModifiedUTf8Str");
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
                debug_assert!(false, "Invalid Modified UTF-8 in ModifiedUTf8Str");
                // SAFETY:
                // ModifiedUtf8Str is valid Modified Utf-8, so a multibyte character will have sufficient continuation bytes
                // Thus this line will never execute because self.0.next() will return Some.
                unsafe { core::hint::unreachable_unchecked() }
            }) as u16;
            let next2 = self.0.next().unwrap_or_else(|| {
                debug_assert!(false, "Invalid Modified UTF-8 in ModifiedUTf8Str");
                // SAFETY:
                // ModifiedUtf8Str is valid Modified Utf-8, so a multibyte character will have sufficient continuation bytes
                // Thus this line will never execute because self.0.next() will return Some.
                unsafe { core::hint::unreachable_unchecked() }
            }) as u16;

            Some((first & 0xf) << 10 | (next1 & 0x3f) << 6 | (next2 & 0x3f))
        }
    }
}

pub struct Chars<'a>(JChars<'a>);

impl<'a> Iterator for Chars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        let mut val = self.0.next()? as u32;
        if let 0xd800..=0xdbff = val {
            val = 0x10000
                + ((val & 0x3ff) << 10)
                + self.0.next().unwrap_or_else(|| {
                    debug_assert!(false, "Invalid Modified UTF-8 in ModifiedUTf8Str");
                    // SAFETY:
                    // ModifiedUtf8Str is valid Modified Utf-8, so a multibyte character will have sufficient continuation bytes
                    // Thus this line will never execute because self.0.next() will return Some.
                    unsafe { core::hint::unreachable_unchecked() }
                }) as u32
                & 0x3fff;
        }

        Some(<char>::from_u32(val).unwrap())
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

impl ModifiedUtf8Str {
    pub fn from_str(x: &str) -> Result<&Self, ModifiedUtf8Error> {
        Self::from_modified_utf8(x.as_bytes())
    }
    pub fn from_modified_utf8(x: &[u8]) -> Result<&Self, ModifiedUtf8Error> {
        validate_modified_utf8(x)?;
        // SAFETY:
        // validation performed above, so x is valid Modified UTF-8
        Ok(unsafe { Self::from_modified_utf8_unchecked(x) })
    }

    ///
    /// Converts a byte slice into a ModifiedUtf8Str without validation
    /// x is required to be a valid [Modified Utf-8 string]
    ///
    pub unsafe fn from_modified_utf8_unchecked(x: &[u8]) -> &Self {
        // SAFETY:
        // x came from a reference so thus is valid. Lifetime of return value is tied to lifetime of x
        // The Safety Invariant is upheld by the precondition of the function
        return unsafe { &*(x as *const [u8] as *const ModifiedUtf8Str) };
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

    pub fn into_str(&self) -> Cow<str> {
        match std::str::from_utf8(&self.0) {
            Ok(s) => Cow::Borrowed(s),
            Err(_) => Cow::Owned(self.to_string()),
        }
    }

    pub fn from_utf8_str(st: &str) -> Cow<ModifiedUtf8Str> {
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
                Cow::Owned(ModifiedUtf8String(vec))
            }
        }
    }
}

impl AsRef<[u8]> for ModifiedUtf8Str {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<ModifiedUtf8Str> for ModifiedUtf8Str {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<ModifiedUtf8Str> for ModifiedUtf8Str {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl Display for ModifiedUtf8Str {
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

impl ::core::fmt::Debug for ModifiedUtf8Str {
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
pub struct ModifiedUtf8String(Vec<u8>);

impl ModifiedUtf8String {
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

    pub fn from_boxed_modified_utf8_str(st: Box<ModifiedUtf8Str>) -> Self {
        Self(Vec::from(unsafe {
            Box::from_raw(Box::into_raw(st) as *mut [u8])
        }))
    }
}

impl Deref for ModifiedUtf8String {
    type Target = ModifiedUtf8Str;

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // ModifiedUtf8String requires that it's content be valid
        unsafe { ModifiedUtf8Str::from_modified_utf8_unchecked(&self.0) }
    }
}

impl DerefMut for ModifiedUtf8String {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.0.as_mut() as *mut [u8] as *mut Self::Target) }
    }
}

impl AsRef<ModifiedUtf8Str> for ModifiedUtf8String {
    fn as_ref(&self) -> &ModifiedUtf8Str {
        self
    }
}

impl AsMut<ModifiedUtf8Str> for ModifiedUtf8String {
    fn as_mut(&mut self) -> &mut ModifiedUtf8Str {
        self
    }
}

impl Borrow<ModifiedUtf8Str> for ModifiedUtf8String {
    fn borrow(&self) -> &ModifiedUtf8Str {
        self
    }
}

impl BorrowMut<ModifiedUtf8Str> for ModifiedUtf8String {
    fn borrow_mut(&mut self) -> &mut ModifiedUtf8Str {
        self
    }
}

impl ToOwned for ModifiedUtf8Str {
    type Owned = ModifiedUtf8String;

    fn to_owned(&self) -> Self::Owned {
        // SAFETY:
        // self is valid Modified UTF-8
        unsafe { ModifiedUtf8String::from_modified_utf8_unchecked(Vec::from(&self.0)) }
    }
}
