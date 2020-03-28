use crate::split_off_front_inplace_mut;
use crate::ByteStringSlice;

#[cfg(test)]
#[path="posix-test.rs"]
mod test;

enum PosixEscapeMode
{
	Outer,
	OuterSlash,
	SingleQuote,
	SingleQuoteSlash,
	DoubleQuote,
	DoubleQuoteSlash,
}

/// Iterator yeilding unescaped strings in the standard POSIX shell format
///
/// - Splits arguments on whitespace (space, tab, newline, and carriage return)
/// - Special character escapes (`\n`, `\t`, `\r`) for literal versions of control characters
/// - Supports single and double-quoted strings
///  - Single quoted strings only support single quote and backslash escaped (any other character is passed verbatim)
///  - Double quoted strings support a full set of escaped special characters.
/// - Interpreted characters can be escaped by prefixing with a backslash
pub struct PosixShellWords<'a,T:?Sized+ByteStringSlice>(&'a mut [u8], ::std::marker::PhantomData<T>);

impl<'a, T: ?Sized + ByteStringSlice> PosixShellWords<'a, T>
{
	pub(crate) fn new(input_bytes: &mut [u8]) -> PosixShellWords<T> {
		PosixShellWords(input_bytes, ::std::marker::PhantomData::<T>)
	}
}

impl<'a, T: ?Sized + ByteStringSlice + 'a> Iterator for PosixShellWords<'a, T>
{
	type Item = &'a T;
	fn next(&mut self) -> Option<&'a T> {
		// 1. Check for an empty string, this means the end has been reached.
		if self.0.len() == 0 {
			// TODO: Error when waiting for a character?
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

