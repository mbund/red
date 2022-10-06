#![allow(dead_code)]

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, one_of};
use nom::combinator::{opt, recognize};
use nom::error::{context, ContextError, ParseError, VerboseError};
use nom::multi::{many0, many1};
use nom::sequence::{separated_pair, terminated, tuple};
use nom::IResult;

type Res<T, U> = IResult<T, U, VerboseError<T>>;

#[derive(Debug, PartialEq)]
pub enum Address {
    Singular(SingularAddress),
    Range(AddressRange),
}

#[derive(Debug, PartialEq)]
pub enum AddressRange {
    Absolute((SingularAddress, SingularAddress)),
    Relative((SingularAddress, SingularAddress)),
    PageForward,
    PageBackward,
}

#[derive(Debug, PartialEq)]
pub struct SingularAddress {
    pub position: AddressPosition,
    pub offset: i64,
}

#[derive(Debug, PartialEq)]
pub enum AddressPosition {
    Default,
    CurrentLine,
    LastLine,
    Line(u64),
    // ForwardRegex(&'a str),
    // BackwardRegex(&'a str),
    // Mark,
}

fn decimal(i: &str) -> Res<&str, &str> {
    recognize(many1(terminated(one_of("0123456789"), many0(char('_')))))(i)
}

fn current_line_parser(i: &str) -> Res<&str, AddressPosition> {
    context("current line", tag("."))(i).map(|(i, _)| (i, AddressPosition::CurrentLine))
}

fn last_line_parser(i: &str) -> Res<&str, AddressPosition> {
    context("last line", tag("$"))(i).map(|(i, _)| (i, AddressPosition::LastLine))
}

fn line_parser(i: &str) -> Res<&str, AddressPosition> {
    context("line", decimal)(i).map(|(i, res)| (i, AddressPosition::Line(res.parse().unwrap())))
}

fn offset_parser(i: &str) -> Res<&str, i64> {
    context("offset", tuple((alt((tag("+"), tag("-"))), decimal)))(i).map(|(i, (sign, number))| {
        (
            i,
            number.parse::<i64>().unwrap() * (if sign == "-" { -1 } else { 1 }),
        )
    })
}

pub fn singular_address_parser(i: &str) -> Res<&str, SingularAddress> {
    context(
        "full_address",
        tuple((
            opt(alt((current_line_parser, last_line_parser, line_parser))),
            opt(offset_parser),
        )),
    )(i)
    .map(|(i, (address_position, offset))| {
        (
            i,
            SingularAddress {
                position: address_position.unwrap_or(AddressPosition::Default),
                offset: offset.unwrap_or(0),
            },
        )
    })
}

pub fn address_parser(i: &str) -> Res<&str, Address> {
    context("address", singular_address_parser)(i)
        .map(|(i, full_address)| (i, Address::Singular(full_address)))
}

pub fn address_range_parser(i: &str) -> Res<&str, Address> {
    context(
        "address_range",
        separated_pair(singular_address_parser, tag(","), singular_address_parser),
    )(i)
    .map(|(i, (addr1, addr2))| (i, Address::Range(AddressRange::Absolute((addr1, addr2)))))
}

pub fn either_address_parser(i: &str) -> Res<&str, Address> {
    context(
        "either_address",
        alt((address_range_parser, address_parser)),
    )(i)
}

#[derive(Debug, PartialEq)]
pub struct AppendCommand {
    pub addr: Address,
}
impl AppendCommand {
    const DEFAULT_ADDRESS: Address = Address::Range(AddressRange::Absolute((
        SingularAddress {
            position: AddressPosition::CurrentLine,
            offset: 0,
        },
        SingularAddress {
            position: AddressPosition::CurrentLine,
            offset: 0,
        },
    )));
}

#[derive(Debug, PartialEq)]
pub struct MoveCommand {
    prev_addr: Address,
    addr: SingularAddress,
}

#[derive(Debug, PartialEq)]
pub struct InsertCommand {
    pub addr: SingularAddress,
}

#[derive(Debug, PartialEq)]
pub struct PrintNoLinesCommand {
    pub addr: Address,
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Append(AppendCommand),
    Move(MoveCommand),
    Insert(InsertCommand),
    PrintNoLines(PrintNoLinesCommand),
}

pub fn append_parser(i: &str) -> Res<&str, Command> {
    context("append", tuple((either_address_parser, tag("a"))))(i)
        .map(|(i, (addr, _))| (i, Command::Append(AppendCommand { addr: addr })))
}

pub fn move_parser(i: &str) -> Res<&str, Command> {
    context(
        "append",
        tuple((address_parser, tag("m"), singular_address_parser)),
    )(i)
    .map(|(i, (addr1, _, addr2))| {
        (
            i,
            Command::Move(MoveCommand {
                prev_addr: addr1,
                addr: addr2,
            }),
        )
    })
}

pub fn insert_parser(i: &str) -> Res<&str, Command> {
    context("append", tuple((singular_address_parser, tag("i"))))(i)
        .map(|(i, (addr, _))| (i, Command::Insert(InsertCommand { addr: addr })))
}

pub fn print_no_lines_parser(i: &str) -> Res<&str, Command> {
    context("append", tuple((either_address_parser, tag("p"))))(i)
        .map(|(i, (addr, _))| (i, Command::PrintNoLines(PrintNoLinesCommand { addr: addr })))
}

pub fn command_parser(i: &str) -> Res<&str, Command> {
    context(
        "command",
        alt((
            append_parser,
            move_parser,
            insert_parser,
            print_no_lines_parser,
        )),
    )(i)
}

#[test]
fn command_parser_test() {
    assert_eq!(
        command_parser("47a"),
        Ok((
            "",
            Command::Append(AppendCommand {
                addr: Address::Singular(SingularAddress {
                    position: AddressPosition::Line(47),
                    offset: 0
                })
            })
        ))
    );

    assert_eq!(
        command_parser("7m$"),
        Ok((
            "",
            Command::Move(MoveCommand {
                prev_addr: Address::Singular(SingularAddress {
                    position: AddressPosition::Line(7),
                    offset: 0
                }),
                addr: SingularAddress {
                    position: AddressPosition::LastLine,
                    offset: 0
                }
            })
        ))
    );

    assert_eq!(
        command_parser("10+2+3a"),
        Ok((
            "",
            Command::Move(MoveCommand {
                prev_addr: Address::Singular(SingularAddress {
                    position: AddressPosition::Line(7),
                    offset: 0
                }),
                addr: SingularAddress {
                    position: AddressPosition::LastLine,
                    offset: 0
                }
            })
        ))
    );
}

#[test]
fn append_parser_test() {
    assert_eq!(
        append_parser("a"),
        Ok((
            "",
            Command::Append(AppendCommand {
                addr: Address::Singular(SingularAddress {
                    position: AddressPosition::Default,
                    offset: 0
                })
            })
        ))
    );

    assert_eq!(
        append_parser("30-4a"),
        Ok((
            "",
            Command::Append(AppendCommand {
                addr: Address::Singular(SingularAddress {
                    position: AddressPosition::Line(30),
                    offset: -4,
                })
            })
        ))
    );

    assert_eq!(
        append_parser("$a"),
        Ok((
            "",
            Command::Append(AppendCommand {
                addr: Address::Singular(SingularAddress {
                    position: AddressPosition::LastLine,
                    offset: 0,
                })
            })
        ))
    );
}
