use std::fs::read_to_string;

use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "habu.pest"]
pub struct ProgramParser;

fn main() {
    let source_contents = read_to_string("hello.habu").unwrap();
    match ProgramParser::parse(Rule::program, &source_contents) {
        Ok(mut pairs) => {
            let pair = pairs.next().unwrap();
            // println!("{pair}");
            println!("{:?}", program(pair));
        }
        Err(err) => {
            println!("{}", err);
        }
    }
}

#[derive(Debug)]
pub struct Program {
    pub objects: Vec<TopLevelObject>,
}

#[derive(Debug)]
pub enum TopLevelObject {
    FunctionDefinition(FunctionDefinition),
    Statement(Statement),
}

#[derive(Debug)]
pub struct FunctionDefinition {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Block,
}

#[derive(Debug)]
pub struct Block {
    statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Assignment {
        target: String,
        value: Expression,
    },
    Declaration {
        target: String,
        value: Option<Expression>,
    },
    IfStatement {
        cond: Expression,
        then: Block,
        els: Option<Block>,
    },
    ForStatement {
        var: String,
        iterator: Expression,
        body: Block,
    },
    FunctionCall(FunctionCall),
}

#[derive(Debug)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Vec<Expression>,
}

#[derive(Debug)]
pub enum UnaryType {
    Before,
    After,
}

#[derive(Debug)]
pub enum Expression {
    Identifier(String),
    FunctionCall(FunctionCall),
    String(String),
    Range {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Number(i32),
    BinaryOperation {
        left: Box<Expression>,
        operator: String,
        right: Box<Expression>,
    },
    UnaryOperation {
        base: Box<Expression>,
        operator: String,
        unary_type: UnaryType,
    },
}

fn program(pair: Pair<Rule>) -> Program {
    Program {
        objects: pair.into_inner().flat_map(|x| tlo(x)).collect(),
    }
}

fn tlo(pair: Pair<Rule>) -> Option<TopLevelObject> {
    if let Some(inner) = pair.into_inner().next() {
        match inner.as_rule() {
            Rule::statement => Some(TopLevelObject::Statement(statement(inner))),
            Rule::functionDefinition => Some(TopLevelObject::FunctionDefinition(func_def(inner))),
            _ => unreachable!(),
        }
    } else {
        None
    }
}

fn block(pair: Pair<Rule>) -> Block {
    Block {
        statements: pair
            .into_inner()
            .filter(|p| p.as_rule() == Rule::statement)
            .map(statement)
            .collect(),
    }
}

fn func_def(pair: Pair<Rule>) -> FunctionDefinition {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::naryFunctionDefinition => {
            let mut inner = inner.into_inner();

            let name = inner.next().unwrap().as_str().to_string();

            let mut parameters = Vec::new();
            let mut body = Block { statements: vec![] };

            for item in inner {
                match item.as_rule() {
                    Rule::id => parameters.push(item.as_str().to_string()),
                    Rule::block => body = block(item),
                    _ => {}
                }
            }

            FunctionDefinition {
                name,
                parameters,
                body,
            }
        }
        Rule::binaryFunctionDefinition => unimplemented!(),
        _ => unreachable!(),
    }
}

fn statement(pair: Pair<Rule>) -> Statement {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::ifStatement => {
            let mut parts = inner.into_inner();
            let cond = expression(parts.next().unwrap());
            let then = block(parts.next().unwrap());
            let possibly_else = parts.next();
            let els = possibly_else.map(|p| block(p));
            Statement::IfStatement { cond, then, els }
        }
        Rule::forLoop => {
            let mut parts = inner.into_inner();
            let var = parts.next().unwrap().as_str();
            let iterator = expression(parts.next().unwrap());
            let body = block(parts.next().unwrap());
            Statement::ForStatement {
                var: var.to_string(),
                iterator,
                body,
            }
        }
        Rule::assignment => {
            let mut parts = inner.into_inner();
            let target = parts.next().unwrap().as_str().to_string();
            let value = expression(parts.next().unwrap());
            Statement::Assignment { target, value }
        }
        Rule::declaration => {
            let mut parts = inner.into_inner();
            Statement::Declaration {
                target: parts.next().unwrap().as_str().to_string(),
                value: parts.next().map(|x| expression(x)),
            }
        }
        Rule::functionCall | Rule::functionCallStatement => {
            Statement::FunctionCall(function_call(inner))
        }
        _ => unreachable!(),
    }
}

fn expression_atom(pair: Pair<Rule>) -> Expression {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::functionCall => Expression::FunctionCall(function_call(pair)),
        Rule::string => Expression::String(pair.as_str().to_string()),
        Rule::range => {
            let mut parts = pair.into_inner();
            let left = Box::new(expression(parts.next().unwrap()));
            let right = Box::new(expression(parts.next().unwrap()));
            Expression::Range { left, right }
        }
        Rule::grouping => expression(pair.into_inner().next().unwrap()),
        Rule::id => Expression::Identifier(pair.as_str().to_string()),
        Rule::number => Expression::Number(pair.as_str().to_string().parse::<i32>().unwrap()),
        _ => unreachable!(),
    }
}

fn expression(pair: Pair<Rule>) -> Expression {
    let mut inner = pair.into_inner().next().unwrap();

    let (base_expr, mut rest) = match inner.as_rule() {
        Rule::preOpExpr => {
            let mut parts = inner.into_inner();
            let operator = parts.next().unwrap().as_str().to_string();
            let base = Box::new(expression_atom(parts.next().unwrap()));
            (
                Expression::UnaryOperation {
                    base,
                    operator,
                    unary_type: UnaryType::Before,
                },
                parts,
            )
        }
        Rule::sufOpExpr => {
            let mut parts = inner.into_inner();
            let base = Box::new(expression_atom(parts.next().unwrap()));
            let operator = parts.next().unwrap().as_str().to_string();
            (
                Expression::UnaryOperation {
                    base,
                    operator,
                    unary_type: UnaryType::After,
                },
                parts,
            )
        }
        Rule::bareExpr => {
            let mut parts = inner.into_inner();
            (expression_atom(parts.next().unwrap()), parts)
        }
        _ => unreachable!(),
    };

    if rest.is_empty() {
        base_expr
    } else {
        let operator = rest.next().unwrap().as_str().to_string();
        let other_operand = expression(rest.next().unwrap());
        Expression::BinaryOperation {
            left: Box::new(base_expr),
            operator,
            right: Box::new(other_operand),
        }
    }

    // let left = match primary_pair.as_rule() {
    //     Rule::id => Expression::Identifier(primary_pair.as_str().to_string()),
    //     Rule::functionCall => Expression::FunctionCall(function_call(primary_pair)),
    //     Rule::grouping => expression(primary_pair.into_inner().next().unwrap()),
    //     _ => unreachable!(),
    // };

    // if let Some(op_pair) = inner.next() {
    //     let operator = op_pair.as_str().to_string();
    //     let right_pair = inner.next().unwrap();
    //     Expression::BinaryOperation {
    //         left: Box::new(left),
    //         operator,
    //         right: Box::new(expression(right_pair)),
    //     }
    // } else {
    //     left
    // }
}

fn function_call(pair: Pair<Rule>) -> FunctionCall {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();

    let mut arguments = Vec::new();
    if let Some(list_pair) = inner.next() {
        arguments = list_pair.into_inner().map(expression).collect();
    }

    FunctionCall { name, arguments }
}
