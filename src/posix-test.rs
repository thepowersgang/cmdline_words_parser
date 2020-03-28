//!
//! Tests for the POSIX/UNIX parser
//!
use crate::parse_posix;

#[test]
fn non_escaped()
{
	let mut s = String::from("Hello world");
	let mut iter = parse_posix(&mut s);
	assert_eq!(iter.next(), Some("Hello"));
	assert_eq!(iter.next(), Some("world"));
	assert_eq!(iter.next(), None);
}

#[test]
fn escaped_spaces()
{
	let mut s = String::from("Hello\\ world");
	let mut iter = parse_posix(&mut s);
	assert_eq!(iter.next(), Some("Hello world"));
	assert_eq!(iter.next(), None);
}

#[test]
fn semi_complex()
{
	let mut s = String::from(r##"Hello world "double quoted (\")" '"single quoted (\')"'  escaped\ string"##);
	let mut iter = parse_posix(&mut s);
	assert_eq!(iter.next(), Some("Hello"));
	assert_eq!(iter.next(), Some("world"));
	assert_eq!(iter.next(), Some("double quoted (\")"));
	assert_eq!(iter.next(), Some("\"single quoted (')\""));
	assert_eq!(iter.next(), Some("escaped string"));
	assert_eq!(iter.next(), None);
}
