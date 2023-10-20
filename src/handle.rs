//! TODO: Module description here...
//!
//! [Handle specification](https://atproto.com/specs/handle)

use std::fmt::Display;

use nom::{
    branch::alt,
    character::complete::{alpha1, alphanumeric0, alphanumeric1},
    combinator::eof,
    multi::{many0, many1},
    sequence::tuple,
    IResult,
};
use nom_supreme::{error::ErrorTree, final_parser::final_parser, tag::complete::tag};

/// TODO: Structure description here...
#[derive(Debug)]
pub struct Handle {
    handle: Box<str>,
}

/// Implements `DID:PLC` lookup by accessing `https://plc.directory/`.
/// Returns values from the array with key `alsoKnownAs` in `plc.directory`'s JSON response.
/// On return the `at://` prefix is discarded.

// TODO: Make it clear we are reading a `DID Document` here.
//      See <https://atproto.com/guides/identity#identifiers> for details.
impl Handle {
    pub async fn from_did(did: &str) -> Result<Option<Vec<Handle>>, reqwest::Error> {
        let qurl = format!("https://plc.directory/{did}");

        #[rustfmt::skip]
        let json = reqwest::get(qurl)
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(match &json["alsoKnownAs"] {
            serde_json::Value::Array(values) => Some(
                values
                    .iter()
                    .filter(|v| matches!(v, serde_json::Value::String(_)))
                    .map(|s| Handle {
                        handle: s.to_string().replace("at://", "").into_boxed_str(),
                    })
                    .collect::<Vec<Handle>>(),
            ),
            _ => None,
        })
    }
}

impl Display for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.handle)
    }
}

impl<'a> TryFrom<&'a str> for Handle {
    type Error = ErrorTree<&'a str>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        parse_into_handle(value)
    }
}

/// A parser for handle
//  constraints:
//      handle total max 253 char
//      handle normalized to lowercase
//      { segment `.` } segment `.` tld
//      segment min 1
//      segment max 63
//      segment start/end allowed `a-z`, `0-9`
//      segment allowed `a-z`, `0-9`, `-`
//      tld start allowed `a-z`
//      tld all else like segment
//
//      tld disallowed: .alt .arpa .example .internal .invalid .local .localhost .onion
//
fn parse_into_handle(input: &str) -> Result<Handle, ErrorTree<&str>> {
    final_parser(parse)(input)
}

// This is a validation-only parser, its only change is to_lower...
fn parse(i: &str) -> IResult<&str, Handle, ErrorTree<&str>> {
    tuple((many1(tuple((segment, tag(".")))), tld, eof))(i)?;

    let handle = i.to_lowercase().to_string().into_boxed_str();
    Ok(("", Handle { handle }))
}

fn segment(i: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    alt((alphanumeric1, segment_long))(i)
}

fn segment_long(i: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    (tuple((alphanumeric1, many0(inner), alphanumeric1)))(i)?;
    Ok(("", i))
}

fn tld(i: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    alt((alpha1, tld_long))(i)
}

fn tld_long(i: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    (tuple((alpha1, many0(inner), alphanumeric1)))(i)?;
    Ok(("", i))
}

fn inner(i: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    alt((alphanumeric0, tag("-")))(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Valid syntax
    #[test]
    fn valid1() {
        let result = Handle::try_from("shuntingyard.bsky.social");
        assert!(result.is_ok());
    }

    #[test]
    fn valid2() {
        let result = Handle::try_from("8.cn");
        assert!(result.is_ok());
    }

    #[test]
    fn valid3() {
        let result = Handle::try_from("name.t--t");
        assert!(result.is_ok());
    }

    #[test]
    fn valid4() {
        let result = Handle::try_from("XXX.LCS.MIT.EDU");
        assert!(result.is_ok());
    }

    #[test]
    fn valid5() {
        let result = Handle::try_from("a.co");
        assert!(result.is_ok());
    }

    #[test]
    fn valid6() {
        let result = Handle::try_from("xn--notarealidn.com");
        assert!(result.is_ok());
    }

    #[test]
    fn valid7() {
        let result = Handle::try_from("xn--fiqa61au8b7zsevnm8ak20mc4a87e.xn--fiqs8s");
        assert!(result.is_ok());
    }

    #[test]
    fn valid8() {
        let result = Handle::try_from("xn--ls8h.test");
        assert!(result.is_ok());
    }

    #[test]
    fn valid9() {
        let result = Handle::try_from("example.t");
        assert!(result.is_ok());
    }

    // Invalid syntax
    #[test]
    fn invalid1() {
        let result = Handle::try_from("org");
        assert!(result.is_err());
    }

    #[test]
    fn invalid3() {
        let result = Handle::try_from("john..test");
        assert!(result.is_err());
    }

    #[test]
    fn invalid4() {
        let result = Handle::try_from("xn--bcher-.tld");
        assert!(result.is_err());
    }

    #[test]
    fn invalid5() {
        let result = Handle::try_from("john.0");
        assert!(result.is_err());
    }

    #[test]
    fn invalid6() {
        let result = Handle::try_from("cn8");
        assert!(result.is_err());
    }

    #[test]
    fn invalid7() {
        let result = Handle::try_from("www.masełkowski.pl.com");
        assert!(result.is_err());
    }

    #[test]
    fn invalid8() {
        let result = Handle::try_from("org");
        assert!(result.is_err());
    }

    #[test]
    fn invalid9() {
        let result = Handle::try_from("name.org.");
        assert!(result.is_err());
    }

    #[test]
    fn private_ipv4() {
        let result = Handle::try_from("tobias.192.168.0.1");
        assert!(result.is_err());
    }

    #[test]
    fn leading_dash() {
        let result = Handle::try_from("-some.org");
        assert!(result.is_err());
    }

    #[test]
    fn trailing_dash() {
        let result = Handle::try_from("some.org-");
        assert!(result.is_err());
    }

    #[test]
    fn two_dashes() {
        let result = Handle::try_from("some.d-a-shy.org");
        assert!(result.is_err());
    }

    // Excessive length
    #[test]
    fn segment_longer_than_63() {
        let result = Handle::try_from(
            "2gzyxa5ihm7nsggfxnu52rck2vv4rvmdlkiu3zzui5du4xyclen53widadfskjhrat7qeOUPFE.com",
        );
        assert!(result.is_err());
    }

    // Disallowed tlds
    #[test]
    fn disallowed_tld1() {
        let result = Handle::try_from("laptop.local");
        assert!(result.is_err());
    }
}
