//! This crate provides a allocation-less way of parsing escaped strings (as the escaped form is longer
//! than the original).
//!
//! - [parse_posix] Parses strings using unix shell escaping rules (see [PosixShellWords] for exact rules)
//!
//! Note: This crate has no way of handling variable substitions.
#![crate_type="lib"]
#![crate_name="cmdline_words_parser"]
#![cfg_attr(not(feature="std"), no_std)]
#[cfg(feature="alloc")]
extern crate alloc;

#[cfg(not(feature="std"))]
mod std {
	pub use core::marker;
	pub use core::mem;
	pub use core::str;
}

pub use crate::posix::PosixShellWords;
mod posix;
//pub use crate::win32::Win32ShellWords;
//mod win32;

/// Parse string in a UNIX/POSIX-like manner
///
/// ```
/// let mut cmdline = String::from(r"Hello\ World 'Second Argument'");
/// let mut parse = ::cmdline_words_parser::parse_posix(&mut cmdline);
/// assert_eq!( parse.next(), Some("Hello World") );
/// assert_eq!( parse.next(), Some("Second Argument") );
/// assert_eq!( parse.next(), None );
/// ```
pub fn parse_posix<T: ?Sized + ByteString>(string: &mut T) -> PosixShellWords<T::OutSlice> {
	// SAFE: Should be ensuring correct (visible) UTF-8
	PosixShellWords::new(unsafe { string.as_mut_bytes() })
}
///// Parse a string using cmd.exe/win32 escaping rules
//pub fn parse_win32<T: ?Sized + StrExt>(string: &mut T) -> Win32ShellWords<T::OutSlice> {
//	Win32ShellWords::new(unsafe { string.as_mut_bytes() })
//}

/// Trait representing types that can be in-place parsed (i.e. ASCII-compatible byte strings)
pub trait ByteString
{
	/// Output slice type (e.g str or OsStr)
	type OutSlice: ?Sized + ByteStringSlice;

	/// Get the string as a mutable sequence of bytes
	unsafe fn as_mut_bytes(&mut self) -> &mut [u8];

	// TODO: Maybe use this instead of the extension trait?
	//fn output_from_bytes(bytes: &[u8]) -> Option<&Self::OutSlice>;
}

/// Parse a rust UTF-8 string, yeilding string slices
impl ByteString for str
{
	type OutSlice = str;

	#[doc(hidden)]
	/// UNSAFE: Caller mustn't violate UTF-8
	unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
		::std::mem::transmute(self)
	}
}
/// Parse a byte slice as a ASCII (or UTF-8/WTF-8/...) string, returning byte slices
impl ByteString for [u8]
{
	type OutSlice = [u8];
	#[doc(hidden)]
	/// Note: Actually safe
	unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
		self
	}
}
#[cfg(feature="std")]
impl ByteString for ::std::ffi::OsStr
{
	type OutSlice = ::std::ffi::OsStr;
	#[doc(hidden)]
	unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
		// NOTE: Type assertion to ensure that assumption holds
		let _: &[u8] = <_ as ::std::os::unix::ffi::OsStrExt>::as_bytes(self);
		// NOTE: OsStr is UTF-8 like on all platforms (Windows uses WTF-8)
		::std::mem::transmute(self)
	}
}
#[cfg(feature="alloc")]
impl ByteString for ::alloc::string::String {
	type OutSlice = str;
	#[doc(hidden)]
	unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
		&mut **self.as_mut_vec()
	}
}


/// Trait representing strings backed by byte arrays
#[doc(hidden)]
pub trait ByteStringSlice {
	fn from_bytes(bytes: &[u8]) -> Option<&Self>;
}
impl ByteStringSlice for str {
	fn from_bytes(bytes: &[u8]) -> Option<&Self> {
		::std::str::from_utf8(bytes).ok()
	}
}
impl ByteStringSlice for [u8] {
	fn from_bytes(bytes: &[u8]) -> Option<&Self> {
		Some(bytes)
	}
}
#[cfg(feature="std")]
impl ByteStringSlice for ::std::ffi::OsStr {
	fn from_bytes(bytes: &[u8]) -> Option<&Self> {
		// SAFE: OsStr is bytes, and string is only modified on ASCII characters
		Some( unsafe { ::std::mem::transmute(bytes) } )
	}
}

/// Helper: Splits the front off a mutable slice
fn split_off_front_inplace_mut<'a, T>(slice: &mut &'a mut [T], idx: usize) -> &'a mut [T] {
	let (ret, tail) = ::std::mem::replace(slice, &mut []).split_at_mut(idx);
	*slice = tail;
	ret
}
