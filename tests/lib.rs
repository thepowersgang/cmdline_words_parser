//

extern crate cmdline_words_parser;

use cmdline_words_parser::StrExt;

#[test]
fn non_escaped()
{
	let mut s = String::from("Hello world");
	let mut iter = s.parse_cmdline_words();
	assert_eq!(iter.next(), Some("Hello"));
	assert_eq!(iter.next(), Some("world"));
	assert_eq!(iter.next(), None);
}

#[test]
fn escaped_spaces()
{
	let mut s = String::from("Hello\\ world");
	let mut iter = s.parse_cmdline_words();
	assert_eq!(iter.next(), Some("Hello world"));
	assert_eq!(iter.next(), None);
}

#[test]
fn semi_complex()
{
	let mut s = String::from("Hello world \"space separated\" escaped\\ string");
	let mut iter = s.parse_cmdline_words();
	assert_eq!(iter.next(), Some("Hello"));
	assert_eq!(iter.next(), Some("world"));
	assert_eq!(iter.next(), Some("space separated"));
	assert_eq!(iter.next(), Some("escaped string"));
	assert_eq!(iter.next(), None);
}
