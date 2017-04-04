#[macro_use]
extern crate nom;

#[macro_use]
mod macros;

mod whitespace;
mod numbers;
mod symbols;
mod expressions;
mod statements;
mod memory;
mod sections;
mod commands;

#[cfg(test)]
mod tests;

pub fn parse(script: &str) {
    commands::script(script).unwrap();
}
