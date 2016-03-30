//! This crate provides a allocation-less way of parsing shell-escaped strings (which are escaped to be longer than the source)
//!
//! The trait `StrExt` provides a method `parse_cmdline_words` that returns an iterator of shell-escaped arguments/"words".
//! 
#![crate_type="lib"]
#![crate_name="cmdline_words_parser"]
#![cfg_attr(feature="no_std", no_std)]

#[cfg(feature="no_std")]
mod std {
	pub use core::marker;
	pub use core::mem;
	pub use core::str;
}

/// Extension trait providing mutable command-line parsing on strings
///
/// ```
/// use cmdline_words_parser::StrExt;
/// let mut cmdline = String::from(r"Hello\ World 'Second Argument'");
/// let mut parse = cmdline.parse_cmdline_words();
/// assert_eq!( parse.next(), Some("Hello World") );
/// assert_eq!( parse.next(), Some("Second Argument") );
/// assert_eq!( parse.next(), None );
/// ```
pub trait StrExt
{
	/// Output slice type (e.g str or OsStr)
	type OutSlice: ?Sized + StrExtOut;
	/// Returns an iterator of POSIX-esque command line arguments
	fn parse_cmdline_words(&mut self) -> PosixShellWords<Self::OutSlice>;
}

/// Parse a rust UTF-8 string, yeilding string slices
impl StrExt for str
{
	type OutSlice = str;
	fn parse_cmdline_words(&mut self) -> PosixShellWords<str> {
		// SAFE: Should be ensuring correct (visible) UTF-8
		PosixShellWords::new(unsafe { ::std::mem::transmute(self) })
	}
}
/// Parse a byte slice as a ASCII (or UTF-8/WTF-8/...) string, returning byte slices
impl StrExt for [u8]
{
	type OutSlice = [u8];
	fn parse_cmdline_words(&mut self) -> PosixShellWords<[u8]> {
		PosixShellWords::new(self)
	}
}
#[cfg(all(not(feature="no_std"), unix))]
impl StrExt for ::std::ffi::OsStr
{
	type OutSlice = ::std::ffi::OsStr;
	fn parse_cmdline_words(&mut self) -> PosixShellWords<Self::OutSlice> {
		// NOTE: Type assertion to ensure that assumption holds
		let _: &[u8] = <_ as ::std::os::unix::ffi::OsStrExt>::as_bytes(self);
		// TODO: Check that OsStr is ASCII based (i.e. UTF-8, or WTF-8, or a ascii-based codepage)
		PosixShellWords::new(unsafe { ::std::mem::transmute(self) })
	}
}
#[cfg(not(feature="no_std"))]
impl StrExt for String {
	type OutSlice = str;
	fn parse_cmdline_words(&mut self) -> PosixShellWords<Self::OutSlice> {
		// SAFE: Parser should ensure that once complete, only correct UTF-8 is visible
		PosixShellWords::new(unsafe { &mut **self.as_mut_vec() })
	}
}


/// Trait used to parameterise output types for StrExt
pub trait StrExtOut {
	fn from_bytes(bytes: &[u8]) -> Option<&Self>;
}
impl StrExtOut for str {
	fn from_bytes(bytes: &[u8]) -> Option<&Self> {
		::std::str::from_utf8(bytes).ok()
	}
}
impl StrExtOut for [u8] {
	fn from_bytes(bytes: &[u8]) -> Option<&Self> {
		Some(bytes)
	}
}
#[cfg(all(not(feature="no_std"), unix))]
impl StrExtOut for ::std::ffi::OsStr {
	fn from_bytes(bytes: &[u8]) -> Option<&Self> {
		// SAFE: OsStr is bytes, and string is only modified on ASCII characters
		Some( unsafe { ::std::mem::transmute(bytes) } )
	}
}

enum PosixEscapeMode
{
	Outer,
	OuterSlash,
	SingleQuote,
	SingleQuoteSlash,
	DoubleQuote,
	DoubleQuoteSlash,
}

fn split_off_front_inplace_mut<'a, T>(slice: &mut &'a mut [T], idx: usize) -> &'a mut [T] {
	let (ret, tail) = ::std::mem::replace(slice, &mut []).split_at_mut(idx);
	*slice = tail;
	ret
}

/// Iterator yeilding unescaped strings in the standard POSIX shell format
///
/// - Splits arguments on whitespace (space, tab, newline, and carriage return)
/// - Special character escapes (`\n`, `\t`, `\r`) for literal versions of control characters
/// - Supports single and double-quoted strings
///  - Single quoted strings only support single quote and backslash escaped (any other character is passed verbatim)
///  - Double quoted strings support a full set of escaped special characters.
/// - Interpreted characters can be escaped by prefixing with a backslash
pub struct PosixShellWords<'a,T:?Sized+StrExtOut>(&'a mut [u8], ::std::marker::PhantomData<T>);

impl<'a, T: ?Sized + StrExtOut> PosixShellWords<'a, T>
{
	fn new(input_bytes: &mut [u8]) -> PosixShellWords<T> {
		PosixShellWords(input_bytes, ::std::marker::PhantomData::<T>)
	}
}

impl<'a, T: ?Sized + StrExtOut + 'a> Iterator for PosixShellWords<'a, T>
{
	type Item = &'a T;
	fn next(&mut self) -> Option<&'a T> {
		// 1. Check for an empty string, this means the end has been reached.
		if self.0.len() == 0 {
			return None;
		}
		
		// 2. Iterate byte-wise along string until something special is hit
		let mut outpos = 0;
		let mut endpos = self.0.len();
		let mut mode = PosixEscapeMode::Outer;
		for i in 0 .. self.0.len()
		{
			let byte = self.0[i];
			let out = match mode
				{
				PosixEscapeMode::Outer => match byte
					{
					b' ' => { endpos = i; break; },
					// TODO: Should tab/newline/return be breaks too?
					b'\t' | b'\n' | b'\r' => { endpos = i; break; },
					b'\\' => {
						mode = PosixEscapeMode::OuterSlash;
						None
						},
					b'\'' => {
						mode = PosixEscapeMode::SingleQuote;
						None
						},
					b'"' => {
						mode = PosixEscapeMode::DoubleQuote;
						None
						},
					v @ _ => Some(v),
					},
				PosixEscapeMode::OuterSlash => {
					mode = PosixEscapeMode::Outer;
					match byte
					{
					v @ b' ' => Some(v),
					v @ b'\t' => Some(v),
					v @ b'\n' => Some(v),
					v @ b'\r' => Some(v),
					v @ b'\'' => Some(v),
					v @ b'\"' => Some(v),
					v @ b'\\' => Some(v),
					b'n' => Some(b'\n'),
					b'r' => Some(b'\r'),
					b't' => Some(b'\t'),
					_ => None,	// TODO: What to to on an invalid escape?
					}},
				PosixEscapeMode::SingleQuote => match byte
					{
					b'\\' => {
						mode = PosixEscapeMode::SingleQuoteSlash;
						None
						},
					b'\'' => {
						mode = PosixEscapeMode::Outer;
						None
						},
					v @ _ => Some(v),
					},
				PosixEscapeMode::SingleQuoteSlash => {
					mode = PosixEscapeMode::SingleQuote;
					match byte
					{
					v @ b'\'' => Some(v),
					v @ b'\\' => Some(v),
					_ => None,	// TODO: What to to on an invalid escape?
					}},
				PosixEscapeMode::DoubleQuote => match byte
					{
					b'\\' => {
						mode = PosixEscapeMode::DoubleQuoteSlash;
						None
						},
					b'"' => {
						mode = PosixEscapeMode::Outer;
						None
						},
					v @ _ => Some(v),
					},
				PosixEscapeMode::DoubleQuoteSlash => {
					mode = PosixEscapeMode::DoubleQuote;
					match byte
					{
					v @ b'\'' => Some(v),
					v @ b'\"' => Some(v),
					v @ b'\\' => Some(v),
					b'n' => Some(b'\n'),
					b'r' => Some(b'\r'),
					b't' => Some(b'\t'),
					_ => None,	// TODO: What to to on an invalid escape?
					}},
				};
			if let Some(b) = out {
				if outpos != i {
					assert!(outpos < i);
					self.0[i] = 0;	// DEFENSIVE. Mangle string at read position to ensure no strays
					self.0[outpos] = b;
				}
				outpos += 1;
			}
		}
		// Consume multiple separators
		while endpos < self.0.len() && self.0[endpos] == b' ' {
			self.0[endpos] = 0;
			endpos += 1;
		}
		
		let ret = &split_off_front_inplace_mut(&mut self.0, endpos)[..outpos];
		Some( T::from_bytes(ret).expect("POSIX Word spliting caused UTF-8 inconsistency") )
	}
}

