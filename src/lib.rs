/*
 *
 */
#![crate_type="lib"]
#![crate_name="cmdline_words_parser"]
#![cfg_attr(nightly, feature(core_slice_ext))]

/// Extension trait providing mutable command-line parsing on strings
pub trait StrExt
{
	type OutSlice: ?Sized + StrExtOut;
	fn parse_cmdline_words(&mut self) -> PosixShellWords<Self::OutSlice>;
}

impl StrExt for str
{
	type OutSlice = str;
	fn parse_cmdline_words(&mut self) -> PosixShellWords<str> {
		// SAFE: Should be ensuring correct (visible) UTF-8
		PosixShellWords::new(unsafe { ::std::mem::transmute(self) })
	}
}
impl StrExt for ::std::ffi::OsStr
{
	type OutSlice = ::std::ffi::OsStr;
	fn parse_cmdline_words(&mut self) -> PosixShellWords<Self::OutSlice> {
		// SAFE: Should be ensuring correct (visible) UTF-8
		PosixShellWords::new(unsafe { ::std::mem::transmute(self) })
	}
}
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
impl StrExtOut for ::std::ffi::OsStr {
	fn from_bytes(bytes: &[u8]) -> Option<&Self> {
		Some( ::std::ffi::OsStr::new(bytes) )
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


pub struct PosixShellWords<'a,T:?Sized+StrExtOut>(&'a mut [u8], ::std::marker::PhantomData<T>);

impl<'a, T: ?Sized + StrExtOut> PosixShellWords<'a, T>
{
	fn new(input_bytes: &mut [u8]) -> PosixShellWords<T> {
		PosixShellWords(input_bytes, ::std::marker::PhantomData::<T>)
	}
}

impl<'a, T: ?Sized + StrExtOut> Iterator for PosixShellWords<'a, T>
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

