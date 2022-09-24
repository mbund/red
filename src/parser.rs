use nom::bytes::streaming::tag;
use nom::IResult;

pub fn i_parser(input: &str) -> IResult<&str, &str> {
    tag("i")(input)
}

pub fn dot_parser(input: &str) -> IResult<&str, &str> {
    tag(".")(input)
}
