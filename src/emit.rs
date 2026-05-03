use crate::parse::{Block, Expression, FunctionCall, Program, Statement, TopLevelObject};

pub struct Emitter {
    level: usize,
}

impl Emitter {
    pub fn emit(&mut self, program: &Program) -> String {
        let mut result = String::new();
        for tlo in &program.objects {
            result += &self.tlo(tlo);
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
            TopLevelObject::Statement(statement) => todo!(),
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

    fn statement(&mut self, statement: &Statement) -> String {
        match statement {
            Statement::Assignment { target, value } => {
                format!("{} = {}", target, self.expression(value))
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
            Statement::ForStatement {
                var,
                iterator,
                body,
            } => {
                format!(
                    "for {} in {}:\n{}",
                    var,
                    self.expression(iterator),
                    self.block(body),
                )
            }
            Statement::FunctionCall(function_call) => self.function_call(&function_call),
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
            Expression::Range { left, right } => {
                format!(
                    "range({}, {})",
                    self.expression(&*left),
                    self.expression(&*right)
                )
            }
            Expression::Number(number) => number.to_string(),
            Expression::BinaryOperation {
                left,
                operator,
                right,
            } => {
                format!(
                    "{} {} {}",
                    self.expression(&*left),
                    operator,
                    self.expression(&*right)
                )
            }
            Expression::UnaryOperation {
                base,
                operator,
                unary_type,
            } => match unary_type {
                crate::parse::UnaryType::Before => {
                    format!("{}{}", operator, self.expression(&*base))
                }
                crate::parse::UnaryType::After => {
                    format!("{}{}", self.expression(&*base), operator)
                }
            },
        }
    }

    pub fn new() -> Emitter {
        Emitter { level: 0 }
    }
}
