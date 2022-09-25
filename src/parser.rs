use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_while, take_while1};
use nom::character::complete::{char, one_of};
use nom::combinator::{map_res, opt, recognize};
use nom::error::context;
use nom::multi::{many0, many1};
use nom::sequence::{delimited, terminated, tuple};
use nom::IResult;

pub enum AddressRange {
    Absolute((Address, Address)),
    Relative((Address, Address)),
    PageForward,
    PageBackward,
}

#[derive(Debug, PartialEq)]
pub struct Address {
    position: AddressPosition,
    offset: i64,
}

#[derive(Debug, PartialEq)]
enum AddressPosition {
    CurrentLine,
    LastLine,
    Line(u64),
    // ForwardRegex(&'a str),
    // BackwardRegex(&'a str),
    // Mark,
}

fn decimal(i: &str) -> IResult<&str, &str> {
    recognize(many1(terminated(one_of("0123456789"), many0(char('_')))))(i)
}

fn current_line_parser(i: &str) -> IResult<&str, AddressPosition> {
    context("current line", tag("."))(i).map(|(i, _)| (i, AddressPosition::CurrentLine))
}

fn last_line_parser(i: &str) -> IResult<&str, AddressPosition> {
    context("last line", tag("$"))(i).map(|(i, _)| (i, AddressPosition::LastLine))
}

fn line_parser(i: &str) -> IResult<&str, AddressPosition> {
    context("line", decimal)(i).map(|(i, res)| (i, AddressPosition::Line(res.parse().unwrap())))
}

fn address_position_parser(i: &str) -> IResult<&str, AddressPosition> {
    context(
        "address position",
        alt((current_line_parser, last_line_parser, line_parser)),
    )(i)
}

fn offset_parser(i: &str) -> IResult<&str, i64> {
    context("offset", tuple((alt((tag("+"), tag("-"))), decimal)))(i).map(|(i, (sign, number))| {
        (
            i,
            number.parse::<i64>().unwrap() * (if sign == "-" { -1 } else { 1 }),
        )
    })
}

pub fn address_parser(i: &str) -> IResult<&str, Address> {
    context(
        "address",
        tuple((opt(address_position_parser), opt(offset_parser))),
    )(i)
    .map(|(i, (address_position, offset))| {
        (
            i,
            Address {
                position: address_position.unwrap_or(AddressPosition::CurrentLine),
                offset: offset.unwrap_or(0),
            },
        )
    })
}

#[test]
fn address_parser_test() {
    assert_eq!(
        address_parser(""),
        Ok((
            "",
            Address {
                position: AddressPosition::CurrentLine,
                offset: 0
            }
        ))
    );

    assert_eq!(
        address_parser("+44"),
        Ok((
            "",
            Address {
                position: AddressPosition::CurrentLine,
                offset: 44
            }
        ))
    );

    assert_eq!(
        address_parser("."),
        Ok((
            "",
            Address {
                position: AddressPosition::CurrentLine,
                offset: 0
            }
        ))
    );

    assert_eq!(
        address_parser("$"),
        Ok((
            "",
            Address {
                position: AddressPosition::LastLine,
                offset: 0
            }
        ))
    );
    assert_eq!(
        address_parser("1"),
        Ok((
            "",
            Address {
                position: AddressPosition::Line(1),
                offset: 0
            }
        ))
    );

    assert_eq!(
        address_parser("100+33"),
        Ok((
            "",
            Address {
                position: AddressPosition::Line(100),
                offset: 33
            }
        ))
    );

    assert_eq!(
        address_parser("100-33"),
        Ok((
            "",
            Address {
                position: AddressPosition::Line(100),
                offset: -33
            }
        ))
    );
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Append(Address),
}

pub fn append_parser(i: &str) -> IResult<&str, Command> {
    context("current line", tuple((address_parser, tag("a"))))(i)
        .map(|(i, (address, _))| (i, Command::Append(address)))
}

#[test]
fn append_parser_test() {
    assert_eq!(
        append_parser("a"),
        Ok((
            "",
            Command::Append(Address {
                position: AddressPosition::CurrentLine,
                offset: 0
            })
        ))
    );

    assert_eq!(
        append_parser("30-4a"),
        Ok((
            "",
            Command::Append(Address {
                position: AddressPosition::Line(30),
                offset: -4
            })
        ))
    );

    assert_eq!(
        append_parser("$a"),
        Ok((
            "",
            Command::Append(Address {
                position: AddressPosition::LastLine,
                offset: 0
            })
        ))
    );
}
