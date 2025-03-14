use super::commands::{command, Command};
use super::memory::region;
use super::memory::Region;
use super::sections::section_command;
use super::sections::SectionCommand;
use super::statements::{statement, Statement};
use super::whitespace::opt_space;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::multi::many1;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug, PartialEq, Clone)]
pub enum RootItem {
    Statement(Statement),
    Command(Command),
    Memory { regions: Vec<Region> },
    Sections { list: Vec<SectionCommand> },
}

fn statement_item(input: &str) -> IResult<&str, RootItem> {
    map(statement, RootItem::Statement)(input)
}

fn command_item(input: &str) -> IResult<&str, RootItem> {
    map(command, RootItem::Command)(input)
}

fn memory_item(input: &str) -> IResult<&str, RootItem> {
    let (input, _) = tuple((tag("MEMORY"), wsc!(tag("{"))))(input)?;
    let (input, regions) = many1(wsc!(region))(input)?;
    let (input, _) = tag("}")(input)?;
    Ok((input, RootItem::Memory { regions }))
}

fn sections_item(input: &str) -> IResult<&str, RootItem> {
    let (input, _) = tuple((tag("SECTIONS"), wsc!(tag("{"))))(input)?;
    let (input, sections) = many1(wsc!(section_command))(input)?;
    let (input, _) = tag("}")(input)?;
    Ok((input, RootItem::Sections { list: sections }))
}

fn root_item(input: &str) -> IResult<&str, RootItem> {
    alt((statement_item, memory_item, sections_item, command_item))(input)
}

pub fn parse(input: &str) -> IResult<&str, Vec<RootItem>> {
    alt((many1(wsc!(root_item)), map(opt_space, |_| vec![])))(input)
}

#[cfg(test)]
mod tests {
    use crate::script::*;
    use std::fs::{self, File};
    use std::io::Read;

    #[test]
    fn test_empty() {
        assert_done_vec!(parse(""), 0);
        assert_done_vec!(parse("                               "), 0);
        assert_done_vec!(parse("      /* hello */              "), 0);
    }

    #[test]
    fn test_parse() {
        for entry in fs::read_dir("tests").unwrap() {
            let path = entry.unwrap().path();
            println!("testing: {:?}", path);
            let mut file = File::open(path).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            assert_done!(parse(&contents));
        }
    }
}
