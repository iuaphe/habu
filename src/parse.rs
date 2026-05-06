use pest::{Parser, iterators::Pair};
use pest_derive::Parser;
use strum::EnumString;

#[derive(Parser)]
#[grammar = "habu.pest"]
pub struct ProgramParser;

fn insert_dents(source: &String) -> String {
    let lines = source.lines();
    let mut out_lines = vec![];
    let mut level = 0;
    for line in lines {
        if line.chars().all(|c| c == ' ') {
            // skip!
        } else {
            let mut i = 0;
            let chars: Vec<char> = line.chars().collect();
            while i < line.len() && chars[i] == ' ' {
                i += 1;
            }
            let this_level = i;
            if this_level > level {
                out_lines.push(format!(
                    "{}{}",
                    "[INDENT]".repeat((this_level - level) / 4),
                    // line[i..].to_string()
                    line,
                ));
            } else if this_level < level {
                out_lines.push(format!(
                    "{}{}",
                    "[DEDENT]".repeat((level - this_level) / 4),
                    // line[i..].to_string()
                    line,
                ));
            } else {
                out_lines.push(format!(
                    "{}{}",
                    "[NODENT]",
                    // line[i..].to_string()
                    line,
                ));
            }
            level = this_level;
        }
    }
    if level > 0 {
        out_lines.push(format!("{}", "[DEDENT]".repeat(level / 4)));
    }
    out_lines.join("\n")
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
    pub statements: Vec<Statement>,
}

#[derive(Debug, EnumString, PartialEq)]
pub enum BreakOrCont {
    #[strum(serialize = ">")]
    Break,
    #[strum(serialize = "^")]
    Cont,
}

#[derive(Debug)]
pub enum Statement {
    Assignment {
        target: String,
        operator: Option<String>,
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
    LoopInst {
        instruction: BreakOrCont,
        times: Option<Expression>,
    },
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
        left_interval: String,
        left: Box<Expression>,
        right: Box<Expression>,
        right_interval: String,
    },
    Float(f64),
    Integer(i64),
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

pub fn parse_program_from_source(
    source_contents: &String,
) -> Result<Program, pest::error::Error<Rule>> {
    let source_contents = insert_dents(&source_contents);
    println!("{}", source_contents);
    let mut pairs = ProgramParser::parse(Rule::program, &source_contents)?;
    let pair = pairs.next().expect("ERROR: Expected token");
    Ok(program(pair))
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
    let inner = pair
        .into_inner()
        .next()
        .expect("ERROR: Expected token in function definition");

    match inner.as_rule() {
        Rule::naryFunctionDefinition => {
            let mut inner = inner.into_inner();

            let name = inner
                .next()
                .expect("ERROR: Expected function name")
                .as_str()
                .to_string();

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
    let inner = pair
        .into_inner()
        .next()
        .expect("ERROR: Expected statement token");
    match inner.as_rule() {
        Rule::ifStatement => {
            let mut parts = inner.into_inner();
            let cond = expression(
                parts
                    .next()
                    .expect("ERROR: Expected if statement condition token"),
            );
            let then = block(
                parts
                    .next()
                    .expect("ERROR: Expected if statement block token"),
            );
            let possibly_else = parts.next();
            let els = possibly_else.map(|p| block(p));
            Statement::IfStatement { cond, then, els }
        }
        Rule::forLoop => {
            let mut parts = inner.into_inner();
            let var = parts
                .next()
                .expect("ERROR: Expected forLoop var token")
                .as_str();
            let iterator = expression(
                parts
                    .next()
                    .expect("ERROR: Expected for loop iterator token"),
            );
            let body = block(parts.next().expect("ERROR: Expected for loop body token"));
            Statement::ForStatement {
                var: var.to_string(),
                iterator,
                body,
            }
        }
        Rule::assignment => {
            let mut parts = inner.into_inner();
            let target = parts
                .next()
                .expect("ERROR: Expected assignment target token")
                .as_str()
                .to_string();

            let mut operator = None;
            if let Some(pair) = parts.peek() {
                if pair.as_rule() == Rule::op {
                    operator = Some(
                        parts
                            .next()
                            .expect("ERROR: Expected assignment operator token")
                            .as_str()
                            .to_string(),
                    );
                }
            }
            let value = expression(
                parts
                    .next()
                    .expect("ERROR: Expected assignment expression token"),
            );
            Statement::Assignment {
                target,
                operator,
                value,
            }
        }
        Rule::declaration => {
            let mut parts = inner.into_inner();
            Statement::Declaration {
                target: parts
                    .next()
                    .expect("ERROR: Expected declaration target token")
                    .as_str()
                    .to_string(),
                value: parts.next().map(|x| expression(x)),
            }
        }
        Rule::functionCall | Rule::functionCallStatement => {
            Statement::FunctionCall(function_call(inner))
        }
        Rule::loopInst => {
            let mut parts = inner.into_inner();
            Statement::LoopInst {
                instruction: parts
                    .next()
                    .expect("ERROR: Expected loop instruction token")
                    .as_str()
                    .parse::<BreakOrCont>()
                    .expect("Invalid loop instruction symbol again somehow"),
                times: parts.next().map(|x| expression(x)),
            }
        }
        _ => unreachable!(),
    }
}

fn expression_atom(pair: Pair<Rule>) -> Expression {
    let pair = pair
        .into_inner()
        .next()
        .expect("ERROR: Expected expression token");
    match pair.as_rule() {
        Rule::functionCall => Expression::FunctionCall(function_call(pair)),
        Rule::string => Expression::String(pair.as_str().to_string()),
        Rule::range => {
            let mut parts = pair.into_inner();
            let left_interval = parts
                .next()
                .expect("ERROR: Expected left range interval token")
                .to_string();
            let left = Box::new(expression(
                parts.next().expect("ERROR: Expected range left token"),
            ));
            let right = Box::new(expression(
                parts.next().expect("ERROR: Expected range right token"),
            ));
            let right_interval = parts
                .next()
                .expect("ERROR: Expected right range interval token")
                .to_string();
            Expression::Range {
                left_interval,
                left,
                right,
                right_interval,
            }
        }
        Rule::grouping => expression(
            pair.into_inner()
                .next()
                .expect("ERROR: Expected grouping token"),
        ),
        Rule::id => Expression::Identifier(pair.as_str().to_string()),
        Rule::integer => Expression::Integer(
            pair.as_str()
                .to_string()
                .parse::<i64>()
                .expect("ERROR: Expected integer token"),
        ),
        Rule::float => Expression::Float(
            pair.as_str()
                .to_string()
                .parse::<f64>()
                .expect("ERROR: Expected float token"),
        ),
        _ => unreachable!(),
    }
}

fn expression(pair: Pair<Rule>) -> Expression {
    // println!("exp: {}", pair);
    let inner = pair
        .into_inner()
        .next()
        .expect("ERROR: Expected expression token");

    let (base_expr, mut rest) = match inner.as_rule() {
        Rule::preOpExpr => {
            let mut parts = inner.into_inner();
            let operator = parts
                .next()
                .expect("ERROR: Expected preOpExpr operator token")
                .as_str()
                .to_string();
            let base = Box::new(expression_atom(
                parts.next().expect("ERROR: Expected preOpExpr base token"),
            ));
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
            let base = Box::new(expression_atom(
                parts.next().expect("ERROR: Expected sufOpExpr base token"),
            ));
            let operator = parts
                .next()
                .expect("ERROR: Expected sufOpExpr operator token")
                .as_str()
                .to_string();
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
            (
                expression_atom(parts.next().expect("ERROR: Expected bareExpr token")),
                parts,
            )
        }
        _ => unreachable!(),
    };

    if rest.is_empty() {
        base_expr
    } else {
        let operator = rest
            .next()
            .expect("ERROR: Expected expression operator token")
            .as_str()
            .to_string();
        let other_operand = expression(
            rest.next()
                .expect("ERROR: Expected expression other_operand token"),
        );
        Expression::BinaryOperation {
            left: Box::new(base_expr),
            operator,
            right: Box::new(other_operand),
        }
    }
}

fn function_call(pair: Pair<Rule>) -> FunctionCall {
    let mut inner = pair.into_inner();
    let name = inner
        .next()
        .expect("ERROR: Expected function name token")
        .as_str()
        .to_string();

    let mut arguments = Vec::new();
    if let Some(list_pair) = inner.next() {
        arguments = list_pair.into_inner().map(expression).collect();
    }

    FunctionCall { name, arguments }
}
