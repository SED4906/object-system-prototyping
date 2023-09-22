use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::IResult;
use nom::combinator::fail;
use object::Form;
use crate::object;

pub fn magic_identify(input: &[u8]) -> Form {
    let result = alt((magic_photo, magic_plaintext))(input);
    if result.is_ok() {
        result.unwrap().1
    } else {
        Form::Binary
    }
}

pub fn magic_photo(input: &[u8]) -> IResult<&[u8], Form> {
    magic_tiff(input)
}

pub fn magic_tiff(input: &[u8]) -> IResult<&[u8], Form> {
    let (input, _) = alt((tag(b"\x49\x49\x2A\x00"),tag(b"\x4D\x4D\x00\x2A")))(input)?;
    Ok((input, Form::Photo))
}

pub fn magic_plaintext(input: &[u8]) -> IResult<&[u8], Form> {
    let result = String::from_utf8(input.to_vec());
    if result.is_ok() {
        Ok((&[], Form::PlainText))
    } else {
        fail(&[])
    }
}