use std::collections::HashMap;

use crate::parse::{Block, Expression, FunctionCall, Program, Statement, TopLevelObject, BreakOrCont};

pub struct Emitter {
    operator_map: HashMap<String, String>,
    level: usize,
}

impl Emitter {
    pub fn emit(&mut self, program: &Program) -> String {
        let mut result = self.indented(
            vec![
                "class Loop:",
                "    continue_flag = False",
                "    semaphore = 0",
            ],
            0,
        );
        for tlo in &program.objects {
            result += &self.tlo(tlo);
            result += &"\n";
        }
        result
    }

    fn tlo(&mut self, tlo: &TopLevelObject) -> String {
        match tlo {
            TopLevelObject::FunctionDefinition(function_definition) => {
                format!(
                    "def {}({}):\n{}",
                    function_definition.name,
                    function_definition.parameters.join(", "),
                    self.block(&function_definition.body)
                )
            }
            TopLevelObject::Statement(statement) => self.statement(statement),
        }
    }

    fn block(&mut self, block: &Block) -> String {
        self.level += 1;
        let result = block
            .statements
            .iter()
            .map(|s| " ".repeat(self.level * 4) + &self.statement(s) + "\n")
            .collect::<Vec<String>>()
            .join("");
        self.level -= 1;
        result
    }

    fn operator(&mut self, op: &String) -> String {
        self.operator_map.get(op).unwrap_or(op).to_string()
    }

    fn statement(&mut self, statement: &Statement) -> String {
        match statement {
            Statement::Assignment {
                target,
                operator,
                value,
            } => {
                if let Some(operator) = operator {
                    format!(
                        "{} = {} {} {}",
                        target,
                        target,
                        self.operator(operator),
                        self.expression(value)
                    )
                } else {
                    format!("{} = {}", target, self.expression(value))
                }
            }
            Statement::Declaration { target, value } => {
                if let Some(value) = value {
                    format!("let {} = {}", target, self.expression(value))
                } else {
                    format!("let {}", target)
                }
            }
            Statement::IfStatement { cond, then, els } => {
                if let Some(els) = els {
                    format!(
                        "if {}:\n{}{}else:\n{}",
                        self.expression(cond),
                        self.block(then),
                        " ".repeat(self.level * 4),
                        self.block(els)
                    )
                } else {
                    format!("if {}:\n{}", self.expression(cond), self.block(then),)
                }
            }
            Statement::ForStatement { var, iterator, body } => {
                format!(
                    "for {} in {}:\n{}{}",
                    var,
                    self.expression(iterator),
                    self.block(body),
                    if self.level > 0 {
                        self.indented(
                            vec![
                                "if Loop.semaphore > 0:",
                                "    Loop.semaphore -= 1",
                                "    if Loop.semaphore == 0 and Loop.continue_flag:",
                                "        Loop.continue_flag = False",
                                "        continue",
                                "    break",
                            ],
                            0,
                        )
                    } else {
                        "".to_string()
                    }
                )
            }
            Statement::FunctionCall(function_call) => self.function_call(&function_call),
            Statement::LoopInst { instruction, times } => {
                let times_val = times.as_ref().map(|e| self.expression(&e));
                match (times_val.as_deref(), instruction) {
                    (None, BreakOrCont::Break) | (Some("1"), BreakOrCont::Break) => "break".to_string(),
                    (None, BreakOrCont::Cont) | (Some("1"), BreakOrCont::Cont) => "continue".to_string(),
                    (Some(t), BreakOrCont::Break) => format!("Loop.semaphore = {t} - 1;break"),
                    (Some(t), BreakOrCont::Cont) => format!("Loop.semaphore = {t} - 1;Loop.continue_flag = True;break"),
                }
            }
        }
    }

    fn function_call(&mut self, function_call: &FunctionCall) -> String {
        format!(
            "{}({})",
            function_call.name,
            function_call
                .arguments
                .iter()
                .map(|a| self.expression(a))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }

    fn expression(&mut self, expression: &Expression) -> String {
        match expression {
            Expression::Identifier(id) => id.clone(),
            Expression::FunctionCall(function_call) => self.function_call(&function_call),
            Expression::String(string) => string.clone(),
            Expression::Range {
                left_interval: leftInterval,
                left,
                right,
                right_interval: rightInterval,
            } => {
                format!(
                    "range({}, {})",
                    if leftInterval == "[" {
                        format!("-({}//-1)", self.expression(&*left))
                    } else {
                        format!("int({})+1", self.expression(&*left))
                    },
                    if rightInterval == "]" {
                        format!("int({})+1", self.expression(&*right))
                    } else {
                        format!("-({}//-1)", self.expression(&*right))
                    }
                )
            }
            Expression::Float(number) => number.to_string(),
            Expression::Integer(number) => number.to_string(),
            Expression::BinaryOperation {
                left,
                operator,
                right,
            } => {
                format!(
                    "{} {} {}",
                    self.expression(&*left),
                    self.operator(operator),
                    self.expression(&*right)
                )
            }
            Expression::UnaryOperation {
                base,
                operator,
                unary_type,
            } => match unary_type {
                crate::parse::UnaryType::Before => {
                    format!("{}{}", self.operator(operator), self.expression(&*base))
                }
                crate::parse::UnaryType::After => {
                    format!("{}{}", self.expression(&*base), self.operator(operator))
                }
            },
        }
    }

    fn operator_map() -> HashMap<String, String> {
        let replacements = [
            ("=?", "=="),
            ("=/", "!="),
            (">_", ">="),
            ("<_", "<="),
            ("&|", "^"),
            ("|&", "^"),
            ("^", "**"),
        ];
        let mut map = HashMap::new();
        for (from, to) in &replacements {
            map.insert(from.to_string(), to.to_string());
        }
        map
    }

    fn indented(&mut self, lines: Vec<&str>, offset_level: usize) -> String {
        lines
            .iter()
            .map(|l| " ".repeat((self.level + offset_level) * 4) + l + "\n")
            .collect::<Vec<String>>()
            .join("")
    }

    pub fn new() -> Emitter {
        Emitter {
            operator_map: Emitter::operator_map(),
            level: 0,
        }
    }
}
