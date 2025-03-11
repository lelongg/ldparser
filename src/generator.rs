use crate::*;
use indent::indent_all_by;

const INDENTATION: usize = 2;

pub trait Generate {
    fn generate(self) -> String;
}

impl Generate for Vec<RootItem> {
    fn generate(self) -> String {
        use RootItem::*;
        let mut output = String::new();
        for item in self {
            match item {
                Statement(stmt) => {
                    output.push_str(&format!("{}\n", stmt.generate()));
                }
                Command(cmd) => {
                    output.push_str(&format!("{}\n", cmd.generate()));
                }
                Memory { regions } => {
                    output.push_str("MEMORY {\n");
                    for region in regions {
                        output.push_str(&format!(
                            "{}\n",
                            indent_all_by(INDENTATION, region.generate())
                        ));
                    }
                    output.push_str("}\n\n");
                }
                Sections { list } => {
                    output.push_str("SECTIONS {\n");
                    for section in list {
                        output.push_str(&format!(
                            "{}\n",
                            indent_all_by(INDENTATION, section.generate())
                        ));
                    }
                    output.push_str("}\n\n");
                }
            }
        }
        output
    }
}

impl Generate for Statement {
    fn generate(self) -> String {
        use Statement::*;
        match self {
            Assign {
                name,
                operator,
                expression,
            } => {
                format!(
                    "{} {} {};",
                    name,
                    operator.generate(),
                    expression.generate()
                )
            }
            Hidden { name, expression } => {
                format!("HIDDEN ({} = {});", name, expression.generate())
            }
            Provide { name, expression } => {
                format!("PROVIDE ({} = {});", name, expression.generate())
            }
            ProvideHidden { name, expression } => {
                format!("PROVIDE_HIDDEN ({} = {});", name, expression.generate())
            }
            Assert { expr, text } => {
                format!("ASSERT (({}), \"{}\");", expr.generate(), text)
            }
        }
    }
}

impl Generate for AssignOperator {
    fn generate(self) -> String {
        use AssignOperator::*;
        match self {
            Equals => "=".to_string(),
            Plus => "+=".to_string(),
            Minus => "-=".to_string(),
            Multiply => "*=".to_string(),
            Divide => "/=".to_string(),
            ShiftLeft => "<<=".to_string(),
            ShiftRight => ">>=".to_string(),
            And => "&=".to_string(),
            Or => "|=".to_string(),
        }
    }
}

impl Generate for Expression {
    fn generate(self) -> String {
        match self {
            Expression::Ident(ident) => ident.clone(),
            Expression::Number(num) => num.to_string(),
            Expression::Call {
                function,
                arguments,
            } => {
                let args: Vec<String> = arguments.into_iter().map(|arg| arg.generate()).collect();
                format!("{}({})", function, args.join(", "))
            }
            Expression::UnaryOp { operator, right } => {
                format!("{}{}", operator.generate(), right.generate())
            }
            Expression::BinaryOp {
                left,
                operator,
                right,
            } => {
                format!(
                    "{} {} {}",
                    left.generate(),
                    operator.generate(),
                    right.generate()
                )
            }
            Expression::TernaryOp {
                condition,
                left,
                right,
            } => {
                format!(
                    "{} ? {} : {}",
                    condition.generate(),
                    left.generate(),
                    right.generate()
                )
            }
        }
    }
}

impl Generate for UnaryOperator {
    fn generate(self) -> String {
        use UnaryOperator::*;
        match self {
            LogicNot => "!".to_string(),
            Minus => "-".to_string(),
            BitwiseNot => "~".to_string(),
        }
    }
}

impl Generate for BinaryOperator {
    fn generate(self) -> String {
        use BinaryOperator::*;
        match self {
            Multiply => "*".to_string(),
            Divide => "/".to_string(),
            ShiftLeft => "<<".to_string(),
            ShiftRight => ">>".to_string(),
            BitwiseAnd => "&".to_string(),
            BitwiseOr => "|".to_string(),
            LogicAnd => "&&".to_string(),
            LogicOr => "||".to_string(),
            Equals => "==".to_string(),
            NotEquals => "!=".to_string(),
            Lesser => "<".to_string(),
            Greater => ">".to_string(),
            LesserOrEquals => "<=".to_string(),
            GreaterOrEquals => ">=".to_string(),
            Plus => "+".to_string(),
            Minus => "-".to_string(),
            Remainder => "%".to_string(),
        }
    }
}

impl Generate for Command {
    fn generate(self) -> String {
        use Command::*;
        match self {
            Call { name, arguments } => {
                let args: Vec<String> = arguments.into_iter().map(|arg| arg.generate()).collect();
                format!("{}({});", name, args.join(", "))
            }
            Include { file } => format!("INCLUDE {};", file),
            Insert { .. } => unimplemented!(),
        }
    }
}

impl Generate for Region {
    fn generate(self) -> String {
        let length = if self.length % (1024 * 1024) == 0 {
            format!("{}M", self.length / (1024 * 1024))
        } else if self.length % 1024 == 0 {
            format!("{}K", self.length / 1024)
        } else {
            self.length.to_string()
        };
        format!(
            "{} : ORIGIN = 0x{:X}, LENGTH = {length}",
            self.name, self.origin
        )
    }
}

impl Generate for SectionCommand {
    fn generate(self) -> String {
        use SectionCommand::*;
        match self {
            Statement(stmt) => stmt.generate(),
            Command(cmd) => cmd.generate(),
            OutputSection {
                name,
                vma_address,
                s_type,
                lma_address,
                section_align,
                align_with_input,
                subsection_align,
                constraint,
                content,
                region,
                lma_region,
                fillexp,
            } => {
                let mut output = format!("{} ", name);
                if let Some(vma_address) = vma_address {
                    output.push_str(&format!("({}) ", vma_address.generate()));
                }
                if let Some(s_type) = s_type {
                    output.push_str(&format!("{} ", s_type.generate()));
                }
                output.push(':');
                if let Some(lma_address) = lma_address {
                    output.push_str(&format!(" AT({}),", lma_address.generate()));
                }
                if let Some(section_align) = section_align {
                    output.push_str(&format!(" ALIGN({}),", section_align.generate()));
                }
                if align_with_input {
                    output.push_str(" ALIGN_WITH_INPUT,");
                }
                if let Some(subsection_align) = subsection_align {
                    output.push_str(&format!(" SUBALIGN({}),", subsection_align.generate()));
                }
                if let Some(constraint) = constraint {
                    output.push_str(&format!(" {},", constraint.generate()));
                }
                output.push_str(" {\n");
                for cmd in content {
                    output.push_str(&format!("  {}\n", cmd.generate()));
                }
                output.push('}');
                if let Some(region) = region {
                    output.push_str(&format!(" >{},", region));
                }
                if let Some(lma_region) = lma_region {
                    output.push_str(&format!(" AT>{}:", lma_region));
                }
                if let Some(fillexp) = fillexp {
                    output.push_str(&format!(" ={};", fillexp.generate()));
                }
                output
            }
        }
    }
}

impl Generate for OutputSectionType {
    fn generate(self) -> String {
        use OutputSectionType::*;
        match self {
            NoLoad => "(NOLOAD)".to_string(),
            DSect => "(DSECT)".to_string(),
            Copy => "(COPY)".to_string(),
            Info => "(INFO)".to_string(),
            Overlay => "(OVERLAY)".to_string(),
        }
    }
}

impl Generate for OutputSectionConstraint {
    fn generate(self) -> String {
        use OutputSectionConstraint::*;
        match self {
            OnlyIfRo => "ONLY_IF_RO".to_string(),
            OnlyIfRw => "ONLY_IF_RW".to_string(),
        }
    }
}

impl Generate for OutputSectionCommand {
    fn generate(self) -> String {
        use OutputSectionCommand::*;
        match self {
            Statement(stmt) => stmt.generate(),
            Fill { expr } => format!("FILL({})", expr.generate()),
            Data { d_type, value } => format!("{} {}", d_type.generate(), value.generate()),
            InputSection { file, sections } => {
                let sections: Vec<String> = sections.into_iter().map(|s| s.generate()).collect();
                format!("{}({})", file.generate(), sections.join(", "))
            }
            KeepInputSection { file, sections } => {
                let sections: Vec<String> = sections.into_iter().map(|s| s.generate()).collect();
                format!("KEEP ({}({}))", file.generate(), sections.join(", "))
            }
        }
    }
}

impl Generate for DataType {
    fn generate(self) -> String {
        use DataType::*;
        match self {
            Byte => "BYTE".to_string(),
            Short => "SHORT".to_string(),
            Long => "LONG".to_string(),
            Quad => "QUAD".to_string(),
        }
    }
}

impl Generate for SectionPattern {
    fn generate(self) -> String {
        use SectionPattern::*;
        match self {
            Simple(name) => name.clone(),
            SortByName(name) => format!("SORT_BY_NAME({})", name),
            SortByAlignment(name) => format!("SORT_BY_ALIGNMENT({})", name),
            SortByInitPriority(name) => format!("SORT_BY_INIT_PRIORITY({})", name),
            SortNone(name) => format!("SORT_NONE({})", name),
            ExcludeFile { files, pattern } => {
                format!("EXCLUDE_FILE({}) {}", files.join(" "), pattern.generate())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs::{read_dir, File},
        io::Read,
    };

    #[test]
    fn test_round_trip() {
        for entry in read_dir("tests").unwrap() {
            let path = entry.unwrap().path();
            println!("testing: {path:?}");
            let mut file = File::open(path).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let parsed_items = parse(&contents).unwrap();
            let generated_content = parsed_items.clone().generate();
            let reparsed_items = parse(&generated_content).unwrap();
            assert_eq!(parsed_items, reparsed_items);
        }
    }
}
