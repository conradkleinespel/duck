// Copyright 2014 The Rustastic SMTP Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Utility functions used in SMTP clients and SMTP servers.

use std::net::AddrParseError;
use std::net::IpAddr;
#[cfg(test)]
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

/// Returns the length of the longest subdomain found at the beginning
/// of the passed string.
///
/// A subdomain is as described
/// [in RFC 5321](http://tools.ietf.org/html/rfc5321#section-4.1.2).
pub fn get_subdomain(s: &str) -> Option<&str> {
    let mut i = 0;
    let mut len = 0;
    if s.len() > 0 && is_alnum(s.chars().nth(0).unwrap()) {
        i += 1;
        len = i;
        while i < s.len() {
            if is_alnum(s.chars().nth(i).unwrap()) {
                i += 1;
                len = i;
            } else if s.chars().nth(i).unwrap() == '-' {
                while i < s.len() && s.chars().nth(i).unwrap() == '-' {
                    i += 1;
                }
            } else {
                break;
            }
        }
    }
    match len {
        0 => None,
        _ => Some(&s[..len]),
    }
}

#[test]
fn test_get_subdomain() {
    // Allow alnum and dashes in the middle, no points.
    assert_eq!(Some("helZo-4-you"), get_subdomain("helZo-4-you&&&"));
    assert_eq!(Some("hePRo-4-you"), get_subdomain("hePRo-4-you.abc"));

    // Test with no content at the end.
    assert_eq!(Some("5---a-U-65"), get_subdomain("5---a-U-65"));
    assert_eq!(None, get_subdomain(""));

    // Disallow dash at the end.
    assert_eq!(Some("heS1o"), get_subdomain("heS1o-&&&"));
    assert_eq!(None, get_subdomain("-hello-world"));
}

/// Returns the length of the longest domain found at the beginning of
/// the passed string.
///
/// A domain is as described
/// [in RFC 5321](http://tools.ietf.org/html/rfc5321#section-4.1.2).
pub fn get_domain(s: &str) -> Option<&str> {
    match get_subdomain(s) {
        Some(sd1) => {
            let mut len = sd1.len();
            while len < s.len() && s.chars().nth(len).unwrap() == '.' {
                match get_subdomain(&s[len + 1..]) {
                    Some(sdx) => {
                        len += 1 + sdx.len();
                    }
                    None => {
                        break;
                    }
                }
            }
            Some(&s[..len])
        }
        None => None,
    }
}

#[test]
fn test_get_domain() {
    // Invalid domain.
    assert_eq!(None, get_domain(".hello"));
    assert_eq!(None, get_domain(""));
    assert_eq!(None, get_domain("----"));

    // Valid domains with dots and dashes.
    assert_eq!(Some("hello-rust.is.N1C3"), get_domain("hello-rust.is.N1C3"));
    assert_eq!(
        Some("hello-rust.is.N1C3"),
        get_domain("hello-rust.is.N1C3.")
    );
    assert_eq!(
        Some("hello-rust.is.N1C3"),
        get_domain("hello-rust.is.N1C3-")
    );
    assert_eq!(
        Some("hello-rust.is.N1C3"),
        get_domain("hello-rust.is.N1C3-.")
    );
    assert_eq!(
        Some("hello-rust.is.N1C3"),
        get_domain("hello-rust.is.N1C3-&")
    );
    assert_eq!(
        Some("hello-rust.is.N1C3"),
        get_domain("hello-rust.is.N1C3.&")
    );

    // Valid domains without dashes.
    assert_eq!(Some("hello.bla"), get_domain("hello.bla."));

    // Valid domains without dots.
    assert_eq!(Some("hello-bla"), get_domain("hello-bla."));
}

/// Returns the length of the longest atom found at the beginning of
/// the passed string.
///
/// An atom is as described
/// [in RFC 5321](http://tools.ietf.org/html/rfc5321#section-4.1.2).
pub fn get_atom(s: &str) -> Option<&str> {
    let mut len = 0;
    while len < s.len() {
        if is_atext(s.chars().nth(len).unwrap()) {
            len += 1
        } else {
            break;
        }
    }
    match len {
        0 => None,
        _ => Some(&s[..len]),
    }
}

#[test]
fn test_get_atom() {
    assert_eq!(None, get_atom(" ---"));
    assert_eq!(Some("!a{`"), get_atom("!a{`\\"));
    assert_eq!(Some("!a{`"), get_atom("!a{`"));
    assert_eq!(None, get_atom(""));
}

/// Returns the length of the longest dot-string found at the beginning
/// of the passed string.
///
/// A dot-string is as described
/// [in RFC 5321](http://tools.ietf.org/html/rfc5321#section-4.1.2).
pub fn get_dot_string(s: &str) -> Option<&str> {
    let mut len = 0;

    match get_atom(s) {
        Some(a1) => {
            len += a1.len();
            while len < s.len() && s.chars().nth(len).unwrap() == '.' {
                match get_atom(&s[len + 1..]) {
                    Some(a) => {
                        len += 1 + a.len();
                    }
                    None => {
                        break;
                    }
                }
            }
            Some(&s[..len])
        }
        None => None,
    }
}

#[test]
fn test_get_dot_string() {
    assert_eq!(None, get_dot_string(""));
    assert_eq!(None, get_dot_string(" fwefwe"));
    assert_eq!(Some("foo"), get_dot_string("foo..bar"));
    assert_eq!(Some("-`-.bla.ok"), get_dot_string("-`-.bla.ok "));
    assert_eq!(Some("-`-.bla.ok"), get_dot_string("-`-.bla.ok"));
    assert_eq!(Some("-`-.bla.ok"), get_dot_string("-`-.bla.ok."));
}

/// Checks whether a character is valid `atext` as described
/// [in RFC 5322](http://tools.ietf.org/html/rfc5322#section-3.2.3).
pub fn is_atext(c: char) -> bool {
    match c {
        '!' | '#' | '$' | '%' | '&' | '\'' | '*' | '+' | '-' | '/' | '=' | '?' | '^' | '_'
        | '`' | '{' | '|' | '}' | '~' => true,
        'A'..='Z' => true,
        'a'..='z' => true,
        '0'..='9' => true,
        _ => false,
    }
}

#[test]
fn test_is_atext() {
    // Valid atext.
    assert!(is_atext('!'));
    assert!(is_atext('#'));
    assert!(is_atext('$'));
    assert!(is_atext('%'));
    assert!(is_atext('&'));
    assert!(is_atext('\''));
    assert!(is_atext('*'));
    assert!(is_atext('+'));
    assert!(is_atext('-'));
    assert!(is_atext('/'));
    assert!(is_atext('='));
    assert!(is_atext('?'));
    assert!(is_atext('^'));
    assert!(is_atext('_'));
    assert!(is_atext('`'));
    assert!(is_atext('{'));
    assert!(is_atext('|'));
    assert!(is_atext('}'));
    assert!(is_atext('~'));
    assert!(is_atext('A'));
    assert!(is_atext('B'));
    assert!(is_atext('C'));
    assert!(is_atext('X'));
    assert!(is_atext('Y'));
    assert!(is_atext('Z'));
    assert!(is_atext('a'));
    assert!(is_atext('b'));
    assert!(is_atext('c'));
    assert!(is_atext('x'));
    assert!(is_atext('y'));
    assert!(is_atext('z'));
    assert!(is_atext('0'));
    assert!(is_atext('1'));
    assert!(is_atext('8'));
    assert!(is_atext('9'));

    // Invalid atext.
    assert!(!is_atext(' '));
    assert!(!is_atext('"'));
    assert!(!is_atext('('));
    assert!(!is_atext(')'));
    assert!(!is_atext(','));
    assert!(!is_atext('.'));
    assert!(!is_atext(':'));
    assert!(!is_atext(';'));
    assert!(!is_atext('<'));
    assert!(!is_atext('>'));
    assert!(!is_atext('@'));
    assert!(!is_atext('['));
    assert!(!is_atext(']'));
    assert!(!is_atext(127 as char));
}

/// Checks if a character is alphanumeric 7 bit ASCII.
pub fn is_alnum(c: char) -> bool {
    match c {
        'A'..='Z' | 'a'..='z' | '0'..='9' => true,
        _ => false,
    }
}

#[test]
fn test_is_alnum() {
    let mut c = 0;
    while c <= 127 {
        // Keep separate assertions for each range to get better error messages.
        if c >= 'A' as u8 && c <= 'Z' as u8 {
            assert!(is_alnum(c as char));
        } else if c >= 'a' as u8 && c <= 'z' as u8 {
            assert!(is_alnum(c as char));
        } else if c >= '0' as u8 && c <= '9' as u8 {
            assert!(is_alnum(c as char));
        } else {
            assert!(!is_alnum(c as char));
        }
        c += 1;
    }
}

/// Returns the length of the longest quoted-string found at the beginning of
/// the passed string. The length includes escaping backslashes and double
/// quotes.
///
/// A quoted-string is as described
/// [in RFC 5321](http://tools.ietf.org/html/rfc5321#section-4.1.2).
pub fn get_quoted_string(s: &str) -> Option<&str> {
    let sl = s.len();
    // We need at least "".
    if sl >= 2 && s.chars().nth(0).unwrap() == '"' {
        // Length of 1 since we have the opening quote.
        let mut len = 1;
        loop {
            // Regular text.
            if len < sl && is_qtext_smtp(s.chars().nth(len).unwrap()) {
                len += 1;
                // Escaped text.
            } else if len + 1 < sl
                && is_quoted_pair_smtp(s.chars().nth(len).unwrap(), s.chars().nth(len + 1).unwrap())
            {
                len += 2;
            } else {
                break;
            }
        }
        if len < sl && s.chars().nth(len).unwrap() == '"' {
            Some(&s[..len + 1])
        } else {
            None
        }
    } else {
        None
    }
}

#[test]
fn test_get_quoted_string() {
    // Invalid.
    assert_eq!(None, get_quoted_string(""));
    assert_eq!(None, get_quoted_string(" "));
    assert_eq!(None, get_quoted_string("  "));
    assert_eq!(None, get_quoted_string(" \""));
    assert_eq!(None, get_quoted_string(" \" \""));
    assert_eq!(None, get_quoted_string("\""));
    assert_eq!(None, get_quoted_string("\"Rust{\\\\\\\"\\a}\\stic"));

    // Valid.
    assert_eq!(Some("\"\""), get_quoted_string("\"\""));
    assert_eq!(
        Some("\"Rust{\\\\\\\"\\a}\\stic\""),
        get_quoted_string("\"Rust{\\\\\\\"\\a}\\stic\"")
    );
    assert_eq!(
        Some("\"Rust{\\\\\\\"\\a}\\stic\""),
        get_quoted_string("\"Rust{\\\\\\\"\\a}\\stic\" ")
    );
}

/// Checks whether a character is valid `qtextSMTP` as described
/// [in RFC 5322](http://tools.ietf.org/html/rfc5322#section-3.2.3).
pub fn is_qtext_smtp(c: char) -> bool {
    match c as isize {
        32..=33 | 35..=91 | 93..=126 => true,
        _ => false,
    }
}

#[test]
fn test_is_qtext_smtp() {
    assert!(!is_qtext_smtp(31 as char));
    assert!(is_qtext_smtp(' '));
    assert!(is_qtext_smtp('!'));
    assert!(!is_qtext_smtp('"'));
    assert!(is_qtext_smtp('#'));
    assert!(is_qtext_smtp('$'));
    assert!(is_qtext_smtp('Z'));
    assert!(is_qtext_smtp('['));
    assert!(!is_qtext_smtp('\\'));
    assert!(is_qtext_smtp(']'));
    assert!(is_qtext_smtp('^'));
    assert!(is_qtext_smtp('}'));
    assert!(is_qtext_smtp('~'));
    assert!(!is_qtext_smtp(127 as char));
}

/// Checks if a pair of characters represent a `quoted-pairSMTP` as described
/// [in RFC 5321](http://tools.ietf.org/html/rfc5321#section-4.1.2)
pub fn is_quoted_pair_smtp(c1: char, c2: char) -> bool {
    c1 as isize == 92 && (c2 as isize >= 32 && c2 as isize <= 126)
}

#[test]
fn test_is_quoted_pair_smtp() {
    assert!(is_quoted_pair_smtp('\\', ' '));
    assert!(is_quoted_pair_smtp('\\', '!'));
    assert!(is_quoted_pair_smtp('\\', '}'));
    assert!(is_quoted_pair_smtp('\\', '~'));
    assert!(!is_quoted_pair_smtp(' ', ' '));
}

/// Returns the length of the longest at-domain found at the beginning of
/// the passed string.
///
/// An at-domain is as described
/// [in RFC 5321](http://tools.ietf.org/html/rfc5321#section-4.1.2).
pub fn get_at_domain(s: &str) -> Option<&str> {
    if s.len() > 1 && s.chars().nth(0).unwrap() == '@' {
        match get_domain(&s[1..]) {
            Some(d) => Some(&s[..1 + d.len()]),
            None => None,
        }
    } else {
        None
    }
}

#[test]
fn test_get_at_domain() {
    assert_eq!(None, get_at_domain(""));
    assert_eq!(None, get_at_domain("@"));
    assert_eq!(None, get_at_domain("@@"));
    assert_eq!(Some("@rust"), get_at_domain("@rust"));
    assert_eq!(Some("@rust"), get_at_domain("@rust{}"));
    assert_eq!(Some("@rustastic.org"), get_at_domain("@rustastic.org"));
}

/// Returns the length of the source routes found at the beginning of
/// the passed string.
///
/// Source routes are as described
/// [in RFC 5321](http://tools.ietf.org/html/rfc5321#section-4.1.2).
pub fn get_source_route(s: &str) -> Option<&str> {
    // The total length we have found for source routes.
    let mut len = 0;

    loop {
        // Get the current source route.
        match get_at_domain(&s[len..]) {
            Some(ad) => {
                len += ad.len();
                // Check if another source route is coming, if not, stop looking
                // for more source routes.
                if len < s.len() && s.chars().nth(len).unwrap() == ',' {
                    len += 1;
                } else {
                    break;
                }
            }
            None => {
                break;
            }
        }
    }

    // Expect the source route declaration to end with ':'.
    if len < s.len() && s.chars().nth(len).unwrap() == ':' {
        Some(&s[..len + 1])
    } else {
        None
    }
}

#[test]
fn test_get_source_route() {
    // Invalid.
    assert_eq!(None, get_source_route(""));
    assert_eq!(None, get_source_route("@rust,"));
    assert_eq!(None, get_source_route("@rust"));
    assert_eq!(None, get_source_route("@,@:"));
    assert_eq!(None, get_source_route("@rust,@troll"));
    assert_eq!(None, get_source_route("@rust,@tro{ll:"));

    // Valid.
    assert_eq!(Some("@rust,@troll:"), get_source_route("@rust,@troll:"));
    assert_eq!(
        Some("@rust.is,@troll:"),
        get_source_route("@rust.is,@troll:")
    );
}

/// If the string starts with an ipv6 as present in email addresses, ie `[Ipv6:...]`, get its
/// length. Else return `0`.
fn get_possible_mailbox_ipv6(ip: &str) -> Option<&str> {
    if ip.len() < 7 || &ip[..6] != "[Ipv6:" {
        None
    } else {
        let mut i = 6;
        while i < ip.len() && ip.chars().nth(i).unwrap() != ']' {
            i += 1;
        }
        if i < ip.len() && ip.chars().nth(i).unwrap() == ']' {
            Some(&ip[..i + 1])
        } else {
            None
        }
    }
}

#[test]
fn test_get_possible_mailbox_ipv6() {
    assert_eq!(Some("[Ipv6:434]"), get_possible_mailbox_ipv6("[Ipv6:434]"));
    assert_eq!(
        Some("[Ipv6:434]"),
        get_possible_mailbox_ipv6("[Ipv6:434][]")
    );
    assert_eq!(Some("[Ipv6:]"), get_possible_mailbox_ipv6("[Ipv6:]"));
    assert_eq!(Some("[Ipv6:]"), get_possible_mailbox_ipv6("[Ipv6:]a"));
    assert_eq!(Some("[Ipv6:::1]"), get_possible_mailbox_ipv6("[Ipv6:::1]"));
    assert_eq!(None, get_possible_mailbox_ipv6("[Ipv6:434"));
    assert_eq!(None, get_possible_mailbox_ipv6("[Ipv"));
}

/// If the string starts with an ipv4 as present in email addresses, ie `[...]`, get its
/// length. Else return `0`.
fn get_possible_mailbox_ipv4(ip: &str) -> Option<&str> {
    if ip.len() < 3
        || ip.chars().nth(0).unwrap() != '['
        || ip.chars().nth(1).unwrap() > '9'
        || ip.chars().nth(1).unwrap() < '0'
    {
        None
    } else {
        let mut i = 1;
        while i < ip.len() && ip.chars().nth(i).unwrap() != ']' {
            i += 1;
        }
        if i < ip.len() && ip.chars().nth(i).unwrap() == ']' {
            Some(&ip[..i + 1])
        } else {
            None
        }
    }
}

#[test]
fn test_get_possible_mailbox_ipv4() {
    assert_eq!(Some("[1]"), get_possible_mailbox_ipv4("[1]"));
    assert_eq!(Some("[1]"), get_possible_mailbox_ipv4("[1]1"));
    assert_eq!(None, get_possible_mailbox_ipv4("[Ipv6:]"));
    assert_eq!(None, get_possible_mailbox_ipv4("[]"));
}

/// Get the IPv4 or IPv6 as used in the foreign part of an email address.
pub fn get_mailbox_ip(s: &str) -> Option<(&str, IpAddr)> {
    get_possible_mailbox_ipv4(s)
        .and_then(|ip| {
            // The IP without prefix / suffix.
            let stripped_ip = &s[
            // Start after the prefix "[" as in "[127.0.0.1]"
            1 ..
            // Go until before the suffix "]".
            ip.len() - 1
        ];

            // Try to parse the IP address.
            let res: Result<IpAddr, AddrParseError> = FromStr::from_str(stripped_ip);

            // Turn the result into an Option.
            res.map(|addr| (ip, addr)).ok()
        })
        .or(get_possible_mailbox_ipv6(s).and_then(|ip| {
            // The IP without prefix / suffix.
            let stripped_ip = &s[
            // Start after the prefix "[Ipv6:" as in "[Ipv6:::1]"
            6 ..
            // Go until before the suffix "]".
            ip.len() - 1
        ];

            // Try to parse the IP address.
            let res: Result<IpAddr, AddrParseError> = FromStr::from_str(stripped_ip);

            // Turn the result into an Option.
            res.map(|addr| (ip, addr)).ok()
        }))
}

#[test]
fn test_get_possible_mailbox() {
    // IPv6
    assert_eq!(
        Some((
            "[Ipv6:::1]",
            IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))
        )),
        get_mailbox_ip("[Ipv6:::1]")
    );

    // IPv4
    assert_eq!(
        Some(("[127.0.0.1]", IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))),
        get_mailbox_ip("[127.0.0.1]")
    );

    // Missing end bracket.
    assert_eq!(None, get_mailbox_ip("[Ipv6:434"));

    // No version.
    assert_eq!(None, get_mailbox_ip("[Ipv"));

    // Nothing in there.
    assert_eq!(None, get_mailbox_ip("[]"));
}
