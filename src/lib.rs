/*
 *
 */
#![crate_type="lib"]
#![crate_name="cmdline_words_parser"]
#![cfg_attr(nightly, feature(core))]

pub trait StrExt
{
	fn parse_cmdline_words(&mut self) -> PosixShellWords;
}

impl StrExt for str
{
	fn parse_cmdline_words(&mut self) -> PosixShellWords {
		// SAFE: Should be ensuring correct (visible) UTF-8
		PosixShellWords(unsafe { ::std::mem::transmute(self) })
	}
}


pub struct PosixShellWords<'a>(&'a mut [u8]);

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
	//let (ret, tail) = slice.split_at_mut(idx);
	*slice = tail;
	ret
}

impl<'a> Iterator for PosixShellWords<'a>
{
	type Item = &'a str;
	fn next(&mut self) -> Option<&'a str> {
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
						mode = PosixEscapeMode::SingleQuote;
						None
						},
					v @ _ => Some(v),
					},
				PosixEscapeMode::OuterSlash => match byte
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
					},
				_ => unimplemented!(),
				};
			if let Some(b) = out {
				if outpos != i {
					assert!(outpos < i);
					self.0[i] = 0;	// DEFENSIVE. Mangle string at read position to ensure no strays
					self.0[outpos] = b;
					outpos += 1;
				}
			}
		}
		// Consume multiple separators
		while endpos < self.0.len() && self.0[endpos] == b' ' {
			self.0[endpos] = 0;
			endpos += 1;
		}
		
		let ret = &split_off_front_inplace_mut(&mut self.0, endpos)[..outpos];
		Some( ::std::str::from_utf8(ret).unwrap() )	//expect("POSIX Word spliting caused UTF-8 inconsistency") )
	}
}

