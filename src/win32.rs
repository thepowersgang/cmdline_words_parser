use crate::split_off_front_inplace_mut;
use crate::ByteStringSlice;

/// Iterator yeilding unescaped strings parsed in Win32 (cmd.exe) format
///
/// - Splits arguments on whitespace (space, tab, newline, and carriage return)
/// - A quote enters "quote mode", ended via either EOL or another " (closing quote cannot be escaped)
/// - '^' escapes everything
pub struct Win32ShellWords<'a,T:?Sized+ByteStringSlice>(&'a mut [u8], ::std::marker::PhantomData<T>);

impl<'a, T: ?Sized + ByteStringSlice> Win32ShellWords<'a, T>
{
	pub(crate) fn new(input_bytes: &mut [u8]) -> Win32ShellWords<T> {
		Win32ShellWords(input_bytes, ::std::marker::PhantomData::<T>)
	}
}
impl<'a, T: ?Sized + ByteStringSlice + 'a> Iterator for Win32ShellWords<'a, T>
{
	type Item = &'a T;
	fn next(&mut self) -> Option<&'a T> {
		// 1. Check for an empty string, this means the end has been reached.
		if self.0.len() == 0 {
			// TODO: Error when waiting for a character?
			return None;
		}

		enum State {
			Normal,
			Quote,
			Escape,
		}
		let mut mode = State::Normal;
		// 2. Iterate byte-wise along string until something special is hit
		let mut outpos = 0;
		let mut endpos = self.0.len();
		for i in 0 .. self.0.len()
		{
			let byte = self.0[i];
			let out = match mode
				{
				State::Normal => match byte
					{
					// TODO: Consume separators
					b' ' => { endpos = i; break; },
					b'\t' | b'\n' | b'\r' => { endpos = i; break; },
					b'^' => {
						mode = State::Escape;
						continue
						},
					b'"' => {
						mode = State::Quote;
						continue
						},
					v @ _ => v,
					},
				State::Quote => match byte
					{
					// <LF> ends the quote early (and terminates the current token)
					b'\n' => { endpos = i; break; },
					// A quote ends the quote mode and switches back to normal
					b'"' => {
						mode = State::Normal;
						continue
						},
					v @ _ => v,
					},
				State::Escape => match byte
					{
					// <LF> can't be escaped
					b'\n' => { endpos = i-1; break; },
					v @ _ => v,
					},
				};

			if outpos != i {
				assert!(outpos < i);
				self.0[i] = 0;	// DEFENSIVE: Zero out string at read position to ensure that there's no invalid UTF-8
				self.0[outpos] = out;
			}
			else {
				// Input = output, don't change
			}
			outpos += 1;
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
