use super::commands::{command, Command};
use super::expressions::expression;
use super::expressions::Expression;
use super::idents::pattern;
use super::idents::symbol;
use super::statements::{statement, Statement};
use super::whitespace::opt_space;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::cut;
use nom::combinator::map;
use nom::combinator::opt;
use nom::multi::many0;
use nom::multi::many1;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug, PartialEq, Clone)]
pub enum SectionCommand {
    Statement(Statement),
    Command(Command),
    OutputSection(OutputSection),
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct OutputSection {
    pub name: String,
    pub vma_address: Option<Box<Expression>>,
    pub s_type: Option<OutputSectionType>,
    pub lma_address: Option<Box<Expression>>,
    pub section_align: Option<Box<Expression>>,
    pub align_with_input: bool,
    pub subsection_align: Option<Box<Expression>>,
    pub constraint: Option<OutputSectionConstraint>,
    pub content: Vec<OutputSectionCommand>,
    pub region: Option<String>,
    pub lma_region: Option<String>,
    pub fillexp: Option<Box<Expression>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum OutputSectionCommand {
    Statement(Statement),
    Fill {
        expr: Box<Expression>,
    },
    Data {
        d_type: DataType,
        value: Box<Expression>,
    },
    InputSection {
        file: SectionPattern,
        sections: Vec<SectionPattern>,
    },
    KeepInputSection {
        file: SectionPattern,
        sections: Vec<SectionPattern>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum DataType {
    Byte,
    Short,
    Long,
    Quad,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SectionPattern {
    Simple(String),
    SortByName(String),
    SortByAlignment(String),
    SortByInitPriority(String),
    SortNone(String),
    ExcludeFile {
        files: Vec<String>,
        pattern: Box<SectionPattern>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum OutputSectionType {
    NoLoad,
    DSect,
    Copy,
    Info,
    Overlay,
}

#[derive(Debug, PartialEq, Clone)]
pub enum OutputSectionConstraint {
    OnlyIfRo,
    OnlyIfRw,
}

fn output_section_type(input: &str) -> IResult<&str, OutputSectionType> {
    alt((
        map(tag("(NOLOAD)"), |_| OutputSectionType::NoLoad),
        map(tag("(DSECT)"), |_| OutputSectionType::DSect),
        map(tag("(COPY)"), |_| OutputSectionType::Copy),
        map(tag("(INFO)"), |_| OutputSectionType::Info),
        map(tag("(OVERLAY)"), |_| OutputSectionType::Overlay),
    ))(input)
}

fn output_section_constraint(input: &str) -> IResult<&str, OutputSectionConstraint> {
    alt((
        map(tag("ONLY_IF_RO"), |_| OutputSectionConstraint::OnlyIfRo),
        map(tag("ONLY_IF_RW"), |_| OutputSectionConstraint::OnlyIfRw),
    ))(input)
}

fn sorted_sp(input: &str) -> IResult<&str, SectionPattern> {
    let (input, keyword) = alt((
        tag("SORT_BY_NAME"),
        tag("SORT_BY_ALIGNMENT"),
        tag("SORT_BY_INIT_PRIORITY"),
        tag("SORT_NONE"),
        tag("SORT"),
    ))(input)?;
    let (input, _) = cut(wsc!(tag("(")))(input)?;
    let (input, inner) = cut(pattern)(input)?;
    let (input, _) = cut(opt_space)(input)?;
    let (input, _) = cut(tag(")"))(input)?;
    Ok((
        input,
        match keyword {
            "SORT" | "SORT_BY_NAME" => SectionPattern::SortByName(inner.into()),
            "SORT_BY_ALIGNMENT" => SectionPattern::SortByAlignment(inner.into()),
            "SORT_BY_INIT_PRIORITY" => SectionPattern::SortByInitPriority(inner.into()),
            "SORT_NONE" => SectionPattern::SortNone(inner.into()),
            _ => panic!("wrong sort keyword"),
        },
    ))
}

fn exclude_file_sp(input: &str) -> IResult<&str, SectionPattern> {
    let (input, _) = tuple((tag("EXCLUDE_FILE"), opt_space, tag("(")))(input)?;
    let (input, files) = cut(many1(wsc!(map(pattern, String::from))))(input)?;
    let (input, _) = cut(tuple((tag(")"), opt_space)))(input)?;
    let (input, inner) = cut(section_pattern)(input)?;
    Ok((
        input,
        SectionPattern::ExcludeFile {
            files,
            pattern: Box::new(inner),
        },
    ))
}

fn simple_sp(input: &str) -> IResult<&str, SectionPattern> {
    map(pattern, |x: &str| SectionPattern::Simple(x.into()))(input)
}

fn section_pattern(input: &str) -> IResult<&str, SectionPattern> {
    alt((exclude_file_sp, sorted_sp, simple_sp))(input)
}

fn data_osc(input: &str) -> IResult<&str, OutputSectionCommand> {
    let (input, d_type) = alt((tag("BYTE"), tag("SHORT"), tag("LONG"), tag("QUAD")))(input)?;
    let (input, _) = wsc!(tag("("))(input)?;
    let (input, value) = expression(input)?;
    let (input, _) = tuple((wsc!(tag(")")), opt(tag(";"))))(input)?;
    Ok((
        input,
        OutputSectionCommand::Data {
            d_type: match d_type {
                "BYTE" => DataType::Byte,
                "SHORT" => DataType::Short,
                "LONG" => DataType::Long,
                "QUAD" => DataType::Quad,
                _ => panic!("invalid data type"),
            },
            value: Box::new(value),
        },
    ))
}

fn fill_osc(input: &str) -> IResult<&str, OutputSectionCommand> {
    let (input, _) = tuple((tag("FILL"), wsc!(tag("("))))(input)?;
    let (input, expr) = expression(input)?;
    let (input, _) = tuple((wsc!(tag(")")), opt(tag(";"))))(input)?;
    Ok((
        input,
        OutputSectionCommand::Fill {
            expr: Box::new(expr),
        },
    ))
}

fn statement_osc(input: &str) -> IResult<&str, OutputSectionCommand> {
    map(statement, OutputSectionCommand::Statement)(input)
}

fn input_osc(input: &str) -> IResult<&str, OutputSectionCommand> {
    let (input, file) = section_pattern(input)?;
    let (input, _) = opt_space(input)?;
    let (input, sections) = opt(delimited(
        wsc!(tag("(")),
        many1(wsc!(section_pattern)),
        wsc!(tag(")")),
    ))(input)?;
    Ok((
        input,
        OutputSectionCommand::InputSection {
            file,
            sections: sections.unwrap_or_default(),
        },
    ))
}

fn keep_osc(input: &str) -> IResult<&str, OutputSectionCommand> {
    let (input, _) = tuple((tag("KEEP"), wsc!(tag("("))))(input)?;
    let (input, inner) = input_osc(input)?;
    let (input, _) = wsc!(tag(")"))(input)?;
    Ok((
        input,
        match inner {
            OutputSectionCommand::InputSection { file, sections } => {
                OutputSectionCommand::KeepInputSection { file, sections }
            }
            _ => panic!("wrong output section command"),
        },
    ))
}

fn output_section_command(input: &str) -> IResult<&str, OutputSectionCommand> {
    alt((statement_osc, keep_osc, data_osc, fill_osc, input_osc))(input)
}

fn statement_sc(input: &str) -> IResult<&str, SectionCommand> {
    map(statement, SectionCommand::Statement)(input)
}

fn command_sc(input: &str) -> IResult<&str, SectionCommand> {
    map(command, SectionCommand::Command)(input)
}

fn output_sc(input: &str) -> IResult<&str, SectionCommand> {
    let (input, name) = alt((tag("/DISCARD/"), symbol))(input)?;
    let (input, _) = opt_space(input)?;
    let (input, s_type1) = opt(output_section_type)(input)?;
    let (input, vma) = wsc!(opt(expression))(input)?;
    let (input, s_type2) = opt(output_section_type)(input)?;
    let (input, _) = wsc!(tag(":"))(input)?;
    let (input, lma) = opt(delimited(tag("AT("), wsc!(expression), tag(")")))(input)?;
    let (input, _) = opt_space(input)?;
    let (input, section_align) = opt(delimited(tag("ALIGN("), wsc!(expression), tag(")")))(input)?;
    let (input, align_with_input) = wsc!(opt(tag("ALIGN_WITH_INPUT")))(input)?;
    let (input, subsection_align) =
        opt(delimited(tag("SUBALIGN("), wsc!(expression), tag(")")))(input)?;
    let (input, constraint) = wsc!(opt(output_section_constraint))(input)?;
    let (input, _) = wsc!(tag("{"))(input)?;
    let (input, content) = many0(wsc!(output_section_command))(input)?;
    let (input, _) = wsc!(tag("}"))(input)?;
    let (input, region) = opt(preceded(tag(">"), wsc!(symbol)))(input)?;
    let (input, lma_region) = opt(preceded(tag("AT>"), wsc!(symbol)))(input)?;
    let (input, fillexp) = opt(preceded(tag("="), wsc!(expression)))(input)?;
    let (input, _) = opt(tag(","))(input)?;
    Ok((
        input,
        SectionCommand::OutputSection(OutputSection {
            name: name.into(),
            vma_address: vma.map(Box::new),
            s_type: if s_type1.is_some() { s_type1 } else { s_type2 },
            lma_address: lma.map(Box::new),
            section_align: section_align.map(Box::new),
            align_with_input: align_with_input.is_some(),
            subsection_align: subsection_align.map(Box::new),
            constraint,
            content,
            region: region.map(String::from),
            lma_region: lma_region.map(String::from),
            fillexp: fillexp.map(Box::new),
        }),
    ))
}

pub fn section_command(input: &str) -> IResult<&str, SectionCommand> {
    alt((statement_sc, output_sc, command_sc))(input)
}

#[cfg(test)]
mod tests {
    use crate::sections::*;

    #[test]
    fn test_section_command() {
        assert_fail!(section_pattern("EXCLUDE_FILE (*a)"));
        assert_fail!(input_osc("EXCLUDE_FILE (*a)"));
        assert_done!(section_pattern("EXCLUDE_FILE ( *a *b ) .c"));
        assert_done!(input_osc("EXCLUDE_FILE ( *a *b ) *c"));

        assert_fail!(input_osc("EXCLUDE_FILE ( EXCLUDE_FILE ( *a *b ) *c ) .d"));
        assert_done!(input_osc("EXCLUDE_FILE ( *a ) *b ( .c )"));
        assert_done!(input_osc("EXCLUDE_FILE ( *a ) *b ( .c .d )"));
        assert_done!(input_osc(
            "EXCLUDE_FILE ( *a ) *b ( .c EXCLUDE_FILE ( *a ) .d )",
        ));

        assert_done!(output_section_command("[A-Z]*(.data)"));
        assert_done!(output_section_command(
            "LONG((__CTOR_END__ - __CTOR_LIST__) / 4 - 2)",
        ));
        assert_done!(output_section_command(
            "EXCLUDE_FILE (*crtend.o *otherfile.o) *(.ctors)",
        ));
        assert_done!(output_section_command(
            "*(EXCLUDE_FILE (*crtend.o *otherfile.o) .ctors)",
        ));
        assert_done!(output_section_command(
            "*(EXCLUDE_FILE (*a) .text EXCLUDE_FILE (*b) .c)",
        ));
        assert_done!(output_section_command("KEEP(SORT_BY_NAME(*)(.ctors))"));
        assert_done!(output_section_command("PROVIDE (__init_array_end = .);"));
        assert_done!(output_section_command("LONG(0);"));
        assert_done!(output_section_command("SORT(CONSTRUCTORS)"));
        assert_done!(output_section_command("*"));

        assert_done!(statement_osc("ASSERT(SIZEOF(.upper)==0,\"Test\");"));
        assert_done!(output_section_command(
            "ASSERT(SIZEOF(.upper)==0,\"Test\");",
        ));
        assert_done!(output_section_command("FILL(0xff);"));

        assert_done!(output_sc("/DISCARD/ : { *(.note.GNU-stack) }"));
        assert_done!(output_sc(".DATA : { [A-Z]*(.data) }"));
        assert_done!(output_sc(".infoD     : {} > INFOD"));

        assert_done!(output_sc(".a:{*(.b .c)*(.d .e)}"));
    }
}
