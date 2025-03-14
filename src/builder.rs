use crate::{commands::InsertOrder, generator::Generate, sections::OutputSection, *};

#[derive(Default)]
pub struct LinkerScriptBuilder {
    memory_builder: MemoryBuilder,
    section_builder: SectionBuilder,
    commands: Vec<Command>,
    statements: Vec<Statement>,
    additional_content: Vec<String>,
}

impl LinkerScriptBuilder {
    pub fn with_memory(mut self, memory_builder: MemoryBuilder) -> Self {
        self.memory_builder = memory_builder;
        self
    }

    pub fn with_sections(mut self, section_builder: SectionBuilder) -> Self {
        self.section_builder = section_builder;
        self
    }

    pub fn with_commands(mut self, commands: impl IntoIterator<Item = Command>) -> Self {
        self.commands = commands.into_iter().collect();
        self
    }

    pub fn with_command(mut self, command: Command) -> Self {
        self.commands.push(command);
        self
    }

    pub fn with_statements(mut self, statements: impl IntoIterator<Item = Statement>) -> Self {
        self.statements = statements.into_iter().collect();
        self
    }

    pub fn with_statement(mut self, statement: Statement) -> Self {
        self.statements.push(statement);
        self
    }

    pub fn with_additional_content(mut self, content: &str) -> Self {
        self.additional_content.push(content.to_string());
        self
    }
}

impl Generate for LinkerScriptBuilder {
    fn generate(self) -> String {
        let mut root_items = vec![
            RootItem::Memory {
                regions: self.memory_builder.regions,
            },
            RootItem::Sections {
                list: self.section_builder.sections,
            },
        ];
        for command in self.commands {
            root_items.push(RootItem::Command(command));
        }
        for statement in self.statements {
            root_items.push(RootItem::Statement(statement));
        }
        let mut script = root_items.generate();
        for content in self.additional_content {
            script.push('\n');
            script.push_str(&content);
        }
        script
    }
}

#[derive(Default)]
pub struct MemoryBuilder {
    pub regions: Vec<Region>,
}

impl MemoryBuilder {
    pub fn with_region(mut self, name: &str, origin: u64, length: u64) -> Self {
        self.regions.push(Region {
            name: name.to_string(),
            origin,
            length,
        });
        self
    }

    pub fn with_adjacent_region(mut self, name: &str, length: u64) -> Self {
        let last_region = self.regions.last().unwrap();
        self.regions.push(Region {
            name: name.to_string(),
            origin: last_region.origin + last_region.length,
            length,
        });
        self
    }
}

#[derive(Default)]
pub struct SectionBuilder {
    sections: Vec<SectionCommand>,
}

impl SectionBuilder {
    pub fn with_statement(
        mut self,
        name: &str,
        operator: AssignOperator,
        expression: Expression,
    ) -> Self {
        self.sections
            .push(SectionCommand::Statement(Statement::Assign {
                name: name.to_string(),
                operator,
                expression: Box::new(expression),
            }));
        self
    }

    pub fn with_command(mut self, command: Command) -> Self {
        self.sections.push(SectionCommand::Command(command));
        self
    }

    pub fn with_output(mut self, output_section: OutputSection) -> Self {
        self.sections
            .push(SectionCommand::OutputSection(output_section));
        self
    }
}

impl Command {
    pub fn call(name: &str, arguments: impl IntoIterator<Item = impl Into<Expression>>) -> Self {
        Command::Call {
            name: name.to_string(),
            arguments: arguments.into_iter().map(|expr| expr.into()).collect(),
        }
    }

    pub fn include(file: impl ToString) -> Self {
        Command::Include {
            file: file.to_string(),
        }
    }

    pub fn insert(order: InsertOrder, section: impl ToString) -> Self {
        Command::Insert {
            order,
            section: section.to_string(),
        }
    }
}

impl OutputSection {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn vma_address(mut self, expr: Expression) -> Self {
        self.vma_address = Some(Box::new(expr));
        self
    }

    pub fn section_type(mut self, s_type: OutputSectionType) -> Self {
        self.s_type = Some(s_type);
        self
    }

    pub fn lma_address(mut self, expr: Expression) -> Self {
        self.lma_address = Some(Box::new(expr));
        self
    }

    pub fn section_align(mut self, expr: Expression) -> Self {
        self.section_align = Some(Box::new(expr));
        self
    }

    pub fn align_with_input(mut self, align: bool) -> Self {
        self.align_with_input = align;
        self
    }

    pub fn subsection_align(mut self, expr: Expression) -> Self {
        self.subsection_align = Some(Box::new(expr));
        self
    }

    pub fn constraint(mut self, constraint: OutputSectionConstraint) -> Self {
        self.constraint = Some(constraint);
        self
    }

    pub fn add_command(mut self, command: OutputSectionCommand) -> Self {
        self.content.push(command);
        self
    }

    pub fn add_commands(
        mut self,
        commands: impl IntoIterator<Item = OutputSectionCommand>,
    ) -> Self {
        self.content.extend(commands);
        self
    }

    pub fn region(mut self, region_name: impl ToString) -> Self {
        self.region = Some(region_name.to_string());
        self
    }

    pub fn lma_region(mut self, region_name: impl ToString) -> Self {
        self.lma_region = Some(region_name.to_string());
        self
    }

    pub fn fillexp(mut self, expr: Expression) -> Self {
        self.fillexp = Some(Box::new(expr));
        self
    }
}

impl OutputSectionCommand {
    pub fn statement(statement: Statement) -> Self {
        OutputSectionCommand::Statement(statement)
    }

    pub fn fill(expression: Expression) -> Self {
        OutputSectionCommand::Fill {
            expr: Box::new(expression),
        }
    }

    pub fn data(d_type: DataType, expression: Expression) -> Self {
        OutputSectionCommand::Data {
            d_type,
            value: Box::new(expression),
        }
    }

    pub fn input_section(
        file: SectionPattern,
        sections: impl IntoIterator<Item = SectionPattern>,
    ) -> Self {
        OutputSectionCommand::InputSection {
            file,
            sections: sections.into_iter().collect(),
        }
    }

    pub fn keep_input_section(
        file: SectionPattern,
        sections: impl IntoIterator<Item = SectionPattern>,
    ) -> Self {
        OutputSectionCommand::KeepInputSection {
            file,
            sections: sections.into_iter().collect(),
        }
    }
}

impl SectionPattern {
    pub fn simple(s: impl ToString) -> Self {
        Self::Simple(s.to_string())
    }

    pub fn sort_by_name(s: impl ToString) -> Self {
        Self::SortByName(s.to_string())
    }

    pub fn sort_by_alignment(s: impl ToString) -> Self {
        Self::SortByAlignment(s.to_string())
    }

    pub fn sort_by_init_priority(s: impl ToString) -> Self {
        Self::SortByInitPriority(s.to_string())
    }

    pub fn sort_none(s: impl ToString) -> Self {
        Self::SortNone(s.to_string())
    }

    pub fn exclude_file(
        files: impl IntoIterator<Item = impl ToString>,
        section_pattern: SectionPattern,
    ) -> Self {
        Self::ExcludeFile {
            files: files.into_iter().map(|s| s.to_string()).collect(),
            pattern: Box::new(section_pattern),
        }
    }
}

impl Statement {
    pub fn assign(
        name: impl ToString,
        operator: AssignOperator,
        expression: impl Into<Expression>,
    ) -> Self {
        Self::Assign {
            name: name.to_string(),
            operator,
            expression: Box::new(expression.into()),
        }
    }

    pub fn hidden(name: impl ToString, expression: impl Into<Expression>) -> Self {
        Self::Hidden {
            name: name.to_string(),
            expression: Box::new(expression.into()),
        }
    }

    pub fn provide(name: impl ToString, expression: impl Into<Expression>) -> Self {
        Self::Provide {
            name: name.to_string(),
            expression: Box::new(expression.into()),
        }
    }

    pub fn provide_hidden(name: impl ToString, expression: impl Into<Expression>) -> Self {
        Self::ProvideHidden {
            name: name.to_string(),
            expression: Box::new(expression.into()),
        }
    }

    pub fn assert(expression: impl Into<Expression>, text: impl ToString) -> Self {
        Self::Assert {
            expr: Box::new(expression.into()),
            text: text.to_string(),
        }
    }
}

impl From<&str> for Expression {
    fn from(value: &str) -> Self {
        expressions::expression(value)
            .map(|(_, expression)| expression)
            .unwrap()
    }
}

pub fn kb(value: u64) -> u64 {
    value * 1024
}

pub fn mb(value: u64) -> u64 {
    value * 1024 * 1024
}
