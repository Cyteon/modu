use crate::ast::AST;
use crate::lexer::{Token, LexingError};
use crate::eval::eval;

use logos::Logos;
use std::collections::HashMap;
use std::vec;

pub fn handle_nested_ast(mut ast: Vec<AST>, temp_ast: Vec<AST>, current_line: usize) -> Result<Vec<AST>, (String, usize)> {
    if ast.is_empty() {
        return Ok(temp_ast);
    }

    let last = ast.pop().unwrap();

    match last {
        AST::Function { name, args, mut body, line } => {
            if let Some(last_body_expr) = body.pop() {
                match last_body_expr {
                    AST::IfStatement { condition, body: if_body, line: if_line } => {
                        let updated_body = handle_nested_ast(if_body, temp_ast, current_line)?;

                        body.push(AST::IfStatement {
                            condition,
                            body: updated_body,
                            line: if_line,
                        });
                    }

                    AST::Null => {
                        body.extend(temp_ast);
                    }

                    other => {
                        body.push(other);
                        body.extend(temp_ast);
                    }
                }
            } else {
                body.extend(temp_ast);
            }

            ast.push(AST::Function {
                name,
                args,
                body,
                line,
            });

            Ok(ast)
        }

        AST::IfStatement { condition, mut body, line } => {
            if let Some(last_body_expr) = body.pop() {
                match last_body_expr {
                    AST::IfStatement { condition: if_condition, body: if_body, line: if_line } => {
                        let updated_body = handle_nested_ast(if_body, temp_ast, current_line)?;

                        body.push(AST::IfStatement {
                            condition: if_condition,
                            body: updated_body,
                            line: if_line,
                        });
                    }

                    AST::Null => {
                        body.extend(temp_ast);
                    }

                    other => {
                        body.push(other);
                        body.extend(temp_ast);
                    }
                }
            } else {
                body.extend(temp_ast);
            }

            ast.push(AST::IfStatement {
                condition,
                body,
                line,
            });

            Ok(ast)
        }

        _ => {
            ast.push(last);
            ast.extend(temp_ast);

            Ok(ast)
        }
    }
}

pub fn handle_nested_arguments(last: AST, arg: AST) -> Result<AST, (String, usize)> {
    match last.clone() {
        AST::Call { name, mut args, line } => {
            let last_arg = args.pop().unwrap_or(AST::Null);

            match (last_arg.clone(), arg.clone()) {
                (AST::Call { name: inner_name, args: mut inner_args, line: inner_line }, AST::Rparen) => {
                    let last_inner = inner_args.pop().unwrap_or(AST::Null);

                    match last_inner {
                        AST::Rparen => {
                            args.push(AST::Call {
                                name: inner_name.clone(),
                                args: inner_args,
                                line: inner_line,
                            });

                            args.push(AST::Rparen);
                        }

                        AST::Call { name: inner_inner_name, args: inner_inner_args, line: inner_inner_line } => {
                            let new_call = handle_nested_arguments(AST::Call {
                                name: inner_inner_name,
                                args: inner_inner_args,
                                line: inner_inner_line,
                            }, AST::Rparen)?;

                            inner_args.push(new_call);

                            args.push(AST::Call {
                                name: inner_name,
                                args: inner_args,
                                line: inner_line,
                            });
                        }

                        AST::PropertyCall { object, property, args: inner_inner_args, line: inner_inner_line } => {
                            let new_call = handle_nested_arguments(AST::PropertyCall {
                                object,
                                property,
                                args: inner_inner_args,
                                line: inner_inner_line,
                            }, AST::Rparen)?;

                            inner_args.push(new_call);

                            args.push(AST::Call {
                                name: inner_name,
                                args: inner_args,
                                line: inner_line,
                            });
                        }

                        _ => {
                            inner_args.push(last_inner);

                            let new_call = handle_nested_arguments(AST::Call {
                                name: inner_name,
                                args: inner_args,
                                line: inner_line,
                            }, AST::Rparen)?;

                            args.push(new_call);
                        }
                    }
                }

                (AST::PropertyCall { object, property, args: mut inner_args, line: inner_line }, AST::Rparen) => {
                    let last_inner = inner_args.pop().unwrap_or(AST::Null);

                    match last_inner {
                        AST::Rparen => {
                            args.push(AST::PropertyCall {
                                object,
                                property,
                                args: inner_args,
                                line: inner_line,
                            });

                            args.push(AST::Rparen);
                        }

                        AST::Call { name: inner_inner_name, args: inner_inner_args, line: inner_inner_line } => {
                            let new_call = handle_nested_arguments(AST::Call {
                                name: inner_inner_name,
                                args: inner_inner_args,
                                line: inner_inner_line,
                            }, AST::Rparen)?;

                            inner_args.push(new_call);

                            args.push(AST::PropertyCall {
                                object,
                                property,
                                args: inner_args,
                                line: inner_line,
                            });
                        }

                        AST::PropertyCall { object: inner_object, property: inner_property, args: inner_inner_args, line: inner_inner_line } => {
                            let new_call = handle_nested_arguments(AST::PropertyCall {
                                object: inner_object,
                                property: inner_property,
                                args: inner_inner_args,
                                line: inner_inner_line,
                            }, AST::Rparen)?;

                            inner_args.push(new_call);

                            args.push(AST::PropertyCall {
                                object,
                                property,
                                args: inner_args,
                                line: inner_line,
                            });
                        }

                        _ => {
                            inner_args.push(last_inner);

                            let new_call = handle_nested_arguments(AST::PropertyCall {
                                object,
                                property,
                                args: inner_args,
                                line: inner_line,
                            }, AST::Rparen)?;

                            args.push(new_call);
                        }
                    }
                }

                (AST::Call { name: inner_name, args: inner_args, line: inner_line }, _) => {
                    match arg {
                        AST::Rparen => {
                            args.push(AST::Call {
                                name: inner_name.clone(),
                                args: inner_args,
                                line: inner_line,
                            });
                        }

                        _ => {
                            match inner_args.clone().pop().unwrap_or(AST::Null) {
                                AST::Rparen => {
                                    args.push(AST::Call {
                                        name: inner_name.clone(),
                                        args: inner_args,
                                        line: inner_line,
                                    });

                                    args.push(arg);
                                }

                                _ => {
                                    let new_call = handle_nested_arguments(AST::Call {
                                        name: inner_name,
                                        args: inner_args,
                                        line: inner_line,
                                    }, arg)?;
        
                                    args.push(new_call);
                                }
                            }
                        }
                    }
                }

                (AST::PropertyCall { object, property, args: inner_args, line: inner_line }, _) => {
                    match arg {
                        AST::Rparen => {
                            args.push(AST::PropertyCall {
                                object,
                                property,
                                args: inner_args,
                                line: inner_line,
                            });
                        }

                        _ => {
                            match inner_args.clone().pop().unwrap_or(AST::Null) {
                                AST::Rparen => {
                                    args.push(AST::PropertyCall {
                                        object,
                                        property,
                                        args: inner_args,
                                        line: inner_line,
                                    });

                                    args.push(arg);
                                }

                                _ => {
                                    let new_call = handle_nested_arguments(AST::PropertyCall {
                                        object,
                                        property,
                                        args: inner_args,
                                        line: inner_line,
                                    }, arg)?;

                                    args.push(new_call);
                                }
                            }
                        }
                    }
                }

                (AST::PropertyAccess { object, property, line }, AST::Identifer(name)) => {
                    args.push(AST::PropertyAccess {
                        object,
                        property: Some(name.clone()),
                        line,
                    });
                }

                (AST::PropertyAccess { object, property, line }, AST::Lparen) => {
                    args.push(AST::PropertyCall {
                        object,
                        property,
                        args: vec![],
                        line,
                    });
                }

                (AST::Identifer(ident), AST::Lparen) => {
                    args.push(AST::Call { name: ident.clone(), args: vec![], line });
                }

                (AST::Identifer(ident), AST::Dot) => {
                    args.push(AST::PropertyAccess {
                        object: Some(ident.clone()),
                        property: None,
                        line,
                    });
                }

                (AST::Null, _) => {
                    args.push(arg);
                }

                (val, AST::Minus) => {
                    if val == AST::Comma {
                        args.push(val);

                        args.push(AST::Subtraction {
                            left: Box::new(AST::Null),
                            right: Box::new(AST::Null),
                            line,
                        });
                    } else {
                        args.push(AST::Subtraction {
                            left: Box::new(val),
                            right: Box::new(AST::Null),
                            line,
                        });
                    }
                }

                (AST::Subtraction { left, right, line }, val) => {
                    if *right == AST::Null && val != AST::Comma {
                        args.push(AST::Subtraction {
                            left,
                            right: Box::new(val),
                            line,
                        });
                    } else {
                        if last_arg != AST::Null {
                            args.push(last_arg);
                        }

                        args.push(arg);
                    }
                }

                (AST::Minus, _) => {
                    args.push(AST::Subtraction {
                        left: Box::new(AST::Null),
                        right: Box::new(arg),
                        line,
                    });
                }

                (val, AST::Plus) => {
                    if val == AST::Comma {
                        args.push(val);

                        args.push(AST::Addition {
                            left: Box::new(AST::Null),
                            right: Box::new(AST::Null),
                            line,
                        });
                    } else {
                        args.push(AST::Addition {
                            left: Box::new(val),
                            right: Box::new(AST::Null),
                            line,
                        });
                    }
                }

                
                (AST::Addition { left, right, line }, val) => {
                    if *right == AST::Null && arg != AST::Comma {
                        args.push(AST::Addition {
                            left,
                            right: Box::new(val),
                            line,
                        });
                    } else {
                        if last_arg != AST::Null {
                            args.push(last_arg);
                        }

                        args.push(arg);
                    }
                }

                (AST::Plus, _) => {
                    args.push(AST::Addition {
                        left: Box::new(AST::Null),
                        right: Box::new(arg),
                        line,
                    });
                }

                _ => {
                    if last_arg != AST::Null {
                        args.push(last_arg);
                    }

                    args.push(arg);
                }
            }

            Ok(AST::Call {
                name: name.clone(),
                args,
                line: line,
            })
        }

        AST::PropertyCall { object, property, mut args, line } => {
            let last_arg = args.pop().unwrap_or(AST::Null);

            match (last_arg.clone(), arg.clone()) {
                (AST::Call { name: inner_name, args: mut inner_args, line: inner_line }, AST::Rparen) => {
                    let last_inner = inner_args.pop().unwrap_or(AST::Null);

                    match last_inner {
                        AST::Rparen => {
                            args.push(AST::Call {
                                name: inner_name.clone(),
                                args: inner_args,
                                line: inner_line,
                            });

                            args.push(AST::Rparen);
                        }

                        AST::Call { name: inner_inner_name, args: inner_inner_args, line: inner_inner_line } => {
                            let new_call = handle_nested_arguments(AST::Call {
                                name: inner_inner_name,
                                args: inner_inner_args,
                                line: inner_inner_line,
                            }, AST::Rparen)?;

                            inner_args.push(new_call);

                            args.push(AST::Call {
                                name: inner_name,
                                args: inner_args,
                                line: inner_line,
                            });
                        }

                        AST::PropertyCall { object, property, args: inner_inner_args, line: inner_inner_line } => {
                            let new_call = handle_nested_arguments(AST::PropertyCall {
                                object,
                                property,
                                args: inner_inner_args,
                                line: inner_inner_line,
                            }, AST::Rparen)?;

                            inner_args.push(new_call);

                            args.push(AST::Call {
                                name: inner_name,
                                args: inner_args,
                                line: inner_line,
                            });
                        }

                        _ => {
                            inner_args.push(last_inner);

                            let new_call = handle_nested_arguments(AST::Call {
                                name: inner_name,
                                args: inner_args,
                                line: inner_line,
                            }, AST::Rparen)?;

                            args.push(new_call);
                        }
                    }
                }

                (AST::PropertyCall { object, property, args: mut inner_args, line: inner_line }, AST::Rparen) => {
                    let last_inner = inner_args.pop().unwrap_or(AST::Null);

                    match last_inner {
                        AST::Rparen => {
                            args.push(AST::PropertyCall {
                                object,
                                property,
                                args: inner_args,
                                line: inner_line,
                            });

                            args.push(AST::Rparen);
                        }

                        AST::Call { name: inner_inner_name, args: inner_inner_args, line: inner_inner_line } => {
                            let new_call = handle_nested_arguments(AST::Call {
                                name: inner_inner_name,
                                args: inner_inner_args,
                                line: inner_inner_line,
                            }, AST::Rparen)?;

                            inner_args.push(new_call);

                            args.push(AST::PropertyCall {
                                object,
                                property,
                                args: inner_args,
                                line: inner_line,
                            });
                        }

                        AST::PropertyCall { object: inner_object, property: inner_property, args: inner_inner_args, line: inner_inner_line } => {
                            let new_call = handle_nested_arguments(AST::PropertyCall {
                                object: inner_object,
                                property: inner_property,
                                args: inner_inner_args,
                                line: inner_inner_line,
                            }, AST::Rparen)?;

                            inner_args.push(new_call);

                            args.push(AST::PropertyCall {
                                object,
                                property,
                                args: inner_args,
                                line: inner_line,
                            });
                        }

                        _ => {
                            inner_args.push(last_inner);

                            let new_call = handle_nested_arguments(AST::PropertyCall {
                                object,
                                property,
                                args: inner_args,
                                line: inner_line,
                            }, AST::Rparen)?;

                            args.push(new_call);
                        }
                    }
                }

                (AST::Call { name: inner_name, args: inner_args, line: inner_line }, _) => {
                    match arg {
                        AST::Rparen => {
                            args.push(AST::Call {
                                name: inner_name.clone(),
                                args: inner_args,
                                line: inner_line,
                            });
                        }

                        _ => {
                            match inner_args.clone().pop().unwrap_or(AST::Null) {
                                AST::Rparen => {
                                    args.push(AST::Call {
                                        name: inner_name.clone(),
                                        args: inner_args,
                                        line: inner_line,
                                    });

                                    args.push(arg);
                                }

                                _ => {
                                    let new_call = handle_nested_arguments(AST::Call {
                                        name: inner_name,
                                        args: inner_args,
                                        line: inner_line,
                                    }, arg)?;
        
                                    args.push(new_call);
                                }
                            }
                        }
                    }
                }

                (AST::PropertyCall { object, property, args: inner_args, line: inner_line }, _) => {
                    match arg {
                        AST::Rparen => {
                            args.push(AST::PropertyCall {
                                object,
                                property,
                                args: inner_args,
                                line: inner_line,
                            });
                        }

                        _ => {
                            match inner_args.clone().pop().unwrap_or(AST::Null) {
                                AST::Rparen => {
                                    args.push(AST::PropertyCall {
                                        object,
                                        property,
                                        args: inner_args,
                                        line: inner_line,
                                    });

                                    args.push(arg);
                                }

                                _ => {
                                    let new_call = handle_nested_arguments(AST::PropertyCall {
                                        object,
                                        property,
                                        args: inner_args,
                                        line: inner_line,
                                    }, arg)?;

                                    args.push(new_call);
                                }
                            }
                        }
                    }
                }

                (AST::PropertyAccess { object, property, line }, AST::Identifer(name)) => {
                    args.push(AST::PropertyAccess {
                        object,
                        property: Some(name.clone()),
                        line,
                    });
                }

                (AST::PropertyAccess { object, property, line }, AST::Lparen) => {
                    args.push(AST::PropertyCall {
                        object,
                        property,
                        args: vec![],
                        line,
                    });
                }

                (AST::Identifer(ident), AST::Lparen) => {
                    args.push(AST::Call { name: ident.clone(), args: vec![], line });
                }

                (AST::Identifer(ident), AST::Dot) => {
                    args.push(AST::PropertyAccess {
                        object: Some(ident.clone()),
                        property: None,
                        line,
                    });
                }

                (AST::Null, _) => {
                    args.push(arg);
                }

                (val, AST::Minus) => {
                    if val == AST::Comma {
                        args.push(val);

                        args.push(AST::Subtraction {
                            left: Box::new(AST::Null),
                            right: Box::new(AST::Null),
                            line,
                        });
                    } else {
                        args.push(AST::Subtraction {
                            left: Box::new(val),
                            right: Box::new(AST::Null),
                            line,
                        });
                    }
                }

                (AST::Subtraction { left, right, line }, val) => {
                    if *right == AST::Null && val != AST::Comma {
                        args.push(AST::Subtraction {
                            left,
                            right: Box::new(val),
                            line,
                        });
                    } else {
                        if last_arg != AST::Null {
                            args.push(last_arg);
                        }

                        args.push(arg);
                    }
                }

                (AST::Minus, _) => {
                    args.push(AST::Subtraction {
                        left: Box::new(AST::Null),
                        right: Box::new(arg),
                        line,
                    });
                }

                (val, AST::Plus) => {
                    if val == AST::Comma {
                        args.push(val);

                        args.push(AST::Addition {
                            left: Box::new(AST::Null),
                            right: Box::new(AST::Null),
                            line,
                        });
                    } else {
                        args.push(AST::Addition {
                            left: Box::new(val),
                            right: Box::new(AST::Null),
                            line,
                        });
                    }
                }

                
                (AST::Addition { left, right, line }, val) => {
                    if *right == AST::Null && arg != AST::Comma {
                        args.push(AST::Addition {
                            left,
                            right: Box::new(val),
                            line,
                        });
                    } else {
                        if last_arg != AST::Null {
                            args.push(last_arg);
                        }

                        args.push(arg);
                    }
                }


                (AST::Plus, _) => {
                    args.push(AST::Addition {
                        left: Box::new(AST::Null),
                        right: Box::new(arg),
                        line,
                    });
                }

                _ => {
                    if last_arg != AST::Null {
                        args.push(last_arg);
                    }

                    args.push(arg);
                }
            }

            Ok(AST::PropertyCall {
                object: object.clone(),
                property: property.clone(),
                args,
                line,
            })
        }

        _ => {
            Ok(last)
        }
    }
}

pub fn clean_args(obj: AST) -> AST {
    match obj {
        AST::Call { name, args, line } => {
            let mut new_args = Vec::new();

            for arg in args {
                match arg {
                    AST::Call { name: arg_name, args: arg_args, line: arg_line } => {
                        new_args.push(clean_args(AST::Call {
                            name: arg_name,
                            args: arg_args,
                            line: arg_line,
                        }));
                    }

                    AST::PropertyCall { object, property, args: arg_args, line: arg_line } => {
                        new_args.push(clean_args(AST::PropertyCall {
                            object,
                            property,
                            args: arg_args,
                            line: arg_line,
                        }));
                    }

                    AST::Rparen
                    | AST::Comma => {}

                    _ => {
                        new_args.push(arg);
                    }
                }
            }

            AST::Call {
                name,
                args: new_args,
                line,
            }
        }

        AST::PropertyCall { object, property, args, line } => {
            let mut new_args = Vec::new();

            for arg in args {
                match arg {
                    AST::Call { name: arg_name, args: arg_args, line: arg_line } => {
                        new_args.push(clean_args(AST::Call {
                            name: arg_name,
                            args: arg_args,
                            line: arg_line,
                        }));
                    }

                    AST::PropertyCall { object, property, args: arg_args, line: arg_line } => {
                        new_args.push(clean_args(AST::PropertyCall {
                            object,
                            property,
                            args: arg_args,
                            line: arg_line,
                        }));
                    }

                    AST::Rparen
                    | AST::Comma => {}

                    _ => {
                        new_args.push(arg);
                    }
                }
            }

            AST::PropertyCall {
                object,
                property,
                args: new_args,
                line,
            }
        }

        AST::IfStatement { condition, body, line } => {
            let mut new_body = vec![];

            for expr in body {
                new_body.push(clean_args(expr));
            }

            AST::IfStatement {
                condition,
                body: new_body,
                line,
            }
        }

        AST::Function { name, args, body, line } => {
            let mut new_body = vec![];

            for expr in body {
                new_body.push(clean_args(expr));
            }

            AST::Function {
                name,
                args,
                body: new_body,
                line,
            }
        }

        _ => obj
    }
}

pub fn parse(input: &str, context: &mut HashMap<String, AST>) -> Result<(), (String, usize)> {
    let verbose = std::env::args().collect::<Vec<String>>()
                            .iter().any(|arg| arg == "--verbose");

    let mut ast = Vec::new();
    let mut line_map = HashMap::new();
    let mut current_line = 0;
    let mut bodies_deep = 0;

    for line in input.split("\n") {
        current_line += 1;

        let mut lexer = Token::lexer(line);

        let mut temp_ast = Vec::new();
        let mut body_starts = false;

        while let Some(token) = lexer.next() {
            match token {
                Ok(Token::Import) => {
                    temp_ast.push(AST::Import {
                        file: None,
                        as_: None,
                        line: current_line,
                    });
                }

                Ok(Token::As) => {
                    let value = temp_ast.pop().unwrap_or(AST::Null);

                    match value {
                        AST::Import { file, line, .. } => {
                            if file.is_none() {
                                return Err(("Expected a file before 'as'".to_string(), current_line));
                            }

                            temp_ast.push(AST::Import {
                                file,
                                as_: None,
                                line,
                            });
                        }

                        _ => {
                            return Err(("Expected an import before 'as'".to_string(), current_line));
                        }
                    }
                }

                Ok(Token::Star) => {
                    let value = temp_ast.pop().unwrap_or(AST::Null);

                    match value {
                        AST::Import { file, as_: _, line } => {
                            if file.is_none() {
                                return Err(("Expected a file before '*'".to_string(), current_line));
                            }

                            temp_ast.push(AST::Import {
                                file,
                                as_: Some("*".to_string()),
                                line,
                            });
                        }

                        _ => {
                            return Err(("Expected an import before '*'".to_string(), current_line));
                        }
                    }
                }

                Ok(Token::Let) => {
                    temp_ast.push(AST::LetDeclaration {
                        name: None,
                        value: Box::new(AST::Null),
                        line: current_line,
                    });
                }

                Ok(Token::Fn) => {
                    temp_ast.push(AST::Function {
                        name: String::new(),
                        args: Vec::new(),
                        body: Vec::new(),
                        line: current_line,
                    });
                }

                Ok(Token::If) => {
                    temp_ast.push(AST::IfStatement {
                        condition: Box::new(AST::Null),
                        body: Vec::new(),
                        line: current_line,
                    });
                }

                Ok(Token::IsEqual) => {
                    let value = temp_ast.pop().unwrap_or(AST::Null);

                    match value {
                        AST::Null => {
                            return Err(("Expected an value before '=='".to_string(), current_line));
                        }

                        _ => {
                            match temp_ast.pop().unwrap_or(AST::Null) {
                                AST::Null => {
                                    return Err(("Expected an value before '=='".to_string(), current_line));
                                }

                                AST::IfStatement { mut condition, body, line } => {
                                    condition = Box::new(AST::IsEqual {
                                        left: Box::new(value),
                                        right: Box::new(AST::Null),
                                        line,
                                    });

                                    temp_ast.push(AST::IfStatement {
                                        condition,
                                        body,
                                        line,
                                    });
                                }

                                _ => {
                                    return Err(("Expected an if statement before '=='".to_string(), current_line));
                                }
                            }
                        }
                    }
                }

                Ok(Token::IsUnequal) => {
                    let value = temp_ast.pop().unwrap_or(AST::Null);

                    match value {
                        AST::Null => {
                            return Err(("Expected an value before '!='".to_string(), current_line));
                        }

                        _ => {
                            match temp_ast.pop().unwrap_or(AST::Null) {
                                AST::Null => {
                                    return Err(("Expected an value before '!='".to_string(), current_line));
                                }

                                AST::IfStatement { mut condition, body, line } => {
                                    condition = Box::new(AST::IsUnequal {
                                        left: Box::new(value),
                                        right: Box::new(AST::Null),
                                        line,
                                    });

                                    temp_ast.push(AST::IfStatement {
                                        condition,
                                        body,
                                        line,
                                    });
                                }

                                _ => {
                                    return Err(("Expected an if statement before '!='".to_string(), current_line));
                                }
                            }
                        }
                    }
                }

                Ok(Token::Dot) => {
                    let value = temp_ast.pop().unwrap_or(AST::Null);

                    match value {
                        AST::Identifer(name) => {
                            temp_ast.push(AST::PropertyAccess {
                                object: Some(name),
                                property: None,
                                line: current_line,
                            });
                        }

                        AST::Function { name, args, body, line } => {
                            temp_ast.push(AST::Function {
                                name,
                                args,
                                body,
                                line,
                            });
                        }

                        AST::Call { name, args, line } => {
                            if args.is_empty() {
                                return Err(("Expected a property before '.' in function call".to_string(), current_line));
                            } else {
                                let new_call = handle_nested_arguments(AST::Call {
                                    name,
                                    args,
                                    line,
                                }, AST::Dot)?;

                                temp_ast.push(new_call);
                            }
                        }

                        AST::PropertyCall { object, property, args, line } => {
                            if args.is_empty() {
                                return Err(("Expected a property before '.' in property call".to_string(), current_line));
                            } else {
                                let new_call = handle_nested_arguments(AST::PropertyCall {
                                    object,
                                    property,
                                    args,
                                    line,
                                }, AST::Dot)?;

                                temp_ast.push(new_call);
                            }
                        }

                        AST::LetDeclaration { name, value, line } => {
                            let value = *value;

                            match value  {
                                AST::Identifer(ident) => {
                                    temp_ast.push(AST::LetDeclaration {
                                        name,
                                        value: Box::new(AST::PropertyAccess {
                                            object: Some(ident),
                                            property: None,
                                            line,
                                        }),
                                        line,
                                    });
                                }

                                AST::Call { name: call_name, args: call_args, line: call_line } => {
                                    let new_call = handle_nested_arguments(AST::Call {
                                        name: call_name,
                                        args: call_args,
                                        line: call_line,
                                    }, AST::Dot)?;

                                    temp_ast.push(AST::LetDeclaration {
                                        name: Some(lexer.slice().to_string()),
                                        value: Box::new(new_call),
                                        line,
                                    });
                                }

                                AST::PropertyCall { object, property, args: call_args, line: call_line } => {
                                    let new_call = handle_nested_arguments(AST::PropertyCall {
                                        object,
                                        property,
                                        args: call_args,
                                        line: call_line,
                                    }, AST::Dot)?;

                                    temp_ast.push(AST::LetDeclaration {
                                        name: Some(lexer.slice().to_string()),
                                        value: Box::new(new_call),
                                        line,
                                    });
                                }

                                _ => {
                                    return Err(("Expected an identifer before '.' in let declaration".to_string(), current_line));
                                }
                            }
                        }

                        _ => {
                            return Err((format!("Unexpected {:?} before '.'", value), current_line));
                        }
                    }
                }
    
                Ok(Token::Identifer) => {
                    let value = temp_ast.pop().unwrap_or(AST::Null);
                    
                    match value {
                        AST::Import { file, as_: _, line } => {
                            temp_ast.push(AST::Import {
                                file,
                                as_: Some(lexer.slice().to_string()),
                                line,
                            });
                        }

                        AST::LetDeclaration { name, value, line } => {
                            if name.is_none() {
                                temp_ast.push(AST::LetDeclaration {
                                    name: Some(lexer.slice().to_string()),
                                    value,
                                    line,
                                });
                            } else {
                                let value = *value;

                                match value {
                                    AST::Null => {
                                        temp_ast.push(AST::LetDeclaration {
                                            name,
                                            value: Box::new(AST::Identifer(lexer.slice().to_string())),
                                            line,
                                        });
                                    }

                                    AST::Addition { left, right: _, line } => {
                                        temp_ast.push(AST::LetDeclaration {
                                            name,
                                            value: Box::new(AST::Addition {
                                                left,
                                                right: Box::new(AST::Identifer(lexer.slice().to_string())),
                                                line,
                                            }),
                                            line,
                                        });
                                    }

                                    AST::Subtraction { left, right: _, line } => {
                                        temp_ast.push(AST::LetDeclaration {
                                            name,
                                            value: Box::new(AST::Subtraction {
                                                left,
                                                right: Box::new(AST::Identifer(lexer.slice().to_string())),
                                                line,
                                            }),
                                            line,
                                        });
                                    }

                                    AST::PropertyAccess { object, property: _, line } => {
                                        temp_ast.push(AST::LetDeclaration {
                                            name,
                                            value: Box::new(AST::PropertyAccess {
                                                object,
                                                property: Some(lexer.slice().to_string()),
                                                line,
                                            }),
                                            line,
                                        });
                                    }

                                    AST::Call { name: call_name, mut args, line } => {
                                        let new_call = handle_nested_arguments(AST::Call {
                                            name: call_name,
                                            args,
                                            line,
                                        }, AST::Identifer(lexer.slice().to_string()))?;

                                        temp_ast.push(AST::LetDeclaration {
                                            name,
                                            value: Box::new(new_call),
                                            line,
                                        });
                                    }

                                    AST::PropertyCall { object, property, mut args, line } => { 
                                        let new_call = handle_nested_arguments(AST::PropertyCall {
                                            object,
                                            property,
                                            args,
                                            line,
                                        }, AST::Identifer(lexer.slice().to_string()))?;

                                        temp_ast.push(AST::LetDeclaration {
                                            name,
                                            value: Box::new(new_call),
                                            line,
                                        });
                                    }

                                    _ => {
                                        return Err(("Unexpected identifier after let declaration".to_string(), current_line));
                                    }
                                }
                            }
                        }

                        AST::Call { name, mut args, line } => {
                            let value = args.pop().unwrap_or(AST::Null);

                            match value {
                                AST::PropertyAccess { object, property: _, line } => {
                                    args.push(AST::PropertyAccess {
                                        object,
                                        property: Some(lexer.slice().to_string()),
                                        line,
                                    });
                                }

                                AST::Addition { left, right: _, line } => {
                                    args.push(AST::Addition {
                                        left,
                                        right: Box::new(AST::Identifer(lexer.slice().to_string())),
                                        line,
                                    });
                                }

                                AST::Subtraction { left, right: _, line } => {
                                    args.push(AST::Subtraction {
                                        left,
                                        right: Box::new(AST::Identifer(lexer.slice().to_string())),
                                        line,
                                    });
                                }

                                AST::Call { name: call_name, args: arg_args, line } => {
                                    let new_call = handle_nested_arguments(AST::Call {
                                        name: call_name,
                                        args: arg_args,
                                        line,
                                    }, AST::Identifer(lexer.slice().to_string()))?;

                                    args.push(new_call);
                                }

                                AST::PropertyCall { object, property, args: property_args, line } => {
                                    let new_call = handle_nested_arguments(AST::PropertyCall {
                                        object,
                                        property,
                                        args: property_args,
                                        line,
                                    }, AST::Identifer(lexer.slice().to_string()))?;

                                    args.push(new_call);
                                }

                                AST::Null => {
                                    args.push(AST::Identifer(lexer.slice().to_string()));
                                }

                                _ => {
                                    args.push(value);

                                    args.push(AST::Identifer(lexer.slice().to_string()));
                                }
                            }

                            temp_ast.push(AST::Call {
                                name,
                                args,
                                line,
                            });
                        }

                        AST::PropertyCall { object, property, mut args, line } => {
                            let arg = args.pop().unwrap_or(AST::Null);

                            match arg {
                                AST::PropertyAccess { object, property: _, line } => {
                                    args.push(AST::PropertyAccess {
                                        object,
                                        property: Some(lexer.slice().to_string()),
                                        line,
                                    });
                                }

                                AST::Addition { left, right: _, line } => {
                                    args.push(AST::Addition {
                                        left,
                                        right: Box::new(AST::Identifer(lexer.slice().to_string())),
                                        line,
                                    });
                                }

                                AST::Subtraction { left, right: _, line } => {
                                    args.push(AST::Subtraction {
                                        left,
                                        right: Box::new(AST::Identifer(lexer.slice().to_string())),
                                        line,
                                    });
                                }

                                AST::Call { name: call_name, args: arg_args, line } => {
                                    let new_call = handle_nested_arguments(AST::Call {
                                        name: call_name,
                                        args: arg_args,
                                        line,
                                    }, AST::Identifer(lexer.slice().to_string()))?;

                                    args.push(new_call);
                                }

                                AST::PropertyCall { object, property, args: property_args, line } => {
                                    let new_call = handle_nested_arguments(AST::PropertyCall {
                                        object,
                                        property,
                                        args: property_args,
                                        line,
                                    }, AST::Identifer(lexer.slice().to_string()))?;

                                    args.push(new_call);
                                }

                                AST::Null => {
                                    args.push(AST::Identifer(lexer.slice().to_string()));
                                }

                                _ => {
                                    args.push(arg);
                                    args.push(AST::Identifer(lexer.slice().to_string()));
                                }
                            }

                            temp_ast.push(AST::PropertyCall {
                                object,
                                property,
                                args,
                                line,
                            });
                        }

                        AST::Function { name, mut args, body, line } => {
                            if name.is_empty() {
                                temp_ast.push(AST::Function {
                                    name: lexer.slice().to_string(),
                                    args,
                                    body,
                                    line,
                                });
                            } else {
                                args.push(lexer.slice().to_string());

                                temp_ast.push(AST::Function {
                                    name,
                                    args,
                                    body,
                                    line,
                                });
                            }
                        }

                        AST::PropertyAccess { object, property: _, line } => {
                            temp_ast.push(AST::PropertyAccess {
                                object,
                                property: Some(lexer.slice().to_string()),
                                line,
                            });
                        }

                        AST::IfStatement { condition, body, line } => {
                            if let AST::IsEqual { left, right, line } = condition.as_ref() {
                                if let AST::Null = **right {
                                    temp_ast.push(AST::IfStatement {
                                        condition: Box::new(AST::IsEqual {
                                            left: left.clone(),
                                            right: Box::new(AST::Identifer(lexer.slice().to_string())),
                                            line: *line,
                                        }),
                                        body,
                                        line: *line,
                                    });
                                } else {
                                    return Err(("Expected a value before '=='".to_string(), current_line));
                                }
                            } else {
                                if let AST::IsUnequal { left, right, line } = condition.as_ref() {
                                    if let AST::Null = **right {
                                        temp_ast.push(AST::IfStatement {
                                            condition: Box::new(AST::IsUnequal {
                                                left: left.clone(),
                                                right: Box::new(AST::Identifer(lexer.slice().to_string())),
                                                line: *line,
                                            }),
                                            body,
                                            line: *line,
                                        });
                                    } else {
                                        return Err(("Expected a value before '!='".to_string(), current_line));
                                    }
                                } else {
                                    temp_ast.push(AST::IfStatement {
                                        condition,
                                        body,
                                        line,
                                    });
        
                                    temp_ast.push(AST::Identifer(lexer.slice().to_string()));
                                }
                            }
                        }

                        AST::Return { value, line } => {
                            if let AST::Null = *value {
                                temp_ast.push(AST::Return {
                                    value: Box::new(AST::Identifer(lexer.slice().to_string())),
                                    line,
                                });
                            } else {
                                return Err(("Unexpected identifier after 'return'".to_string(), current_line));
                            }
                        }

                        _ => {
                            temp_ast.push(value);
                            temp_ast.push(AST::Identifer(lexer.slice().to_string()));
                        }
                    }
                }
    
                Ok(Token::Assign) => {
                    if let Some(AST::LetDeclaration { name, value, line }) = temp_ast.pop() {                    
                        if AST::Null == *value {
                            temp_ast.push(AST::LetDeclaration {
                                name,
                                value: Box::new(AST::Null),
                                line,
                            });
                        } else {
                            return Err(("Unexpected '='".to_string(), current_line));
                        }
                    } else {
                        return Err(("Expected a let declaration before '='".to_string(), current_line));
                    }
                }

                Ok(Token::Comma) => {
                    let value = temp_ast.pop().unwrap_or(AST::Null);

                    match value {
                        AST::Call { name, args, line } => {
                            let new_call = handle_nested_arguments(AST::Call {
                                name,
                                args,
                                line,
                            }, AST::Comma)?;

                            temp_ast.push(new_call);
                        }

                        AST::PropertyCall { object, property, args, line } => {
                            temp_ast.push(AST::PropertyCall {
                                object,
                                property,
                                args,
                                line,
                            });
                        }

                        AST::Function { name, args, body, line } => {
                            temp_ast.push(AST::Function {
                                name,
                                args,
                                body,
                                line,
                            });
                        }

                        AST::LetDeclaration { name, value, line } => {
                            temp_ast.push(AST::LetDeclaration {
                                name,
                                value,
                                line,
                            });
                        }

                        _ => {
                            return Err((format!("Unexpected ',' after {:?}", value), current_line));
                        }
                    }
                }
    
                Ok(Token::LParen) => {
                    let value = temp_ast.pop().unwrap_or(AST::Null);

                    match value {
                        AST::Identifer(name) => {
                            temp_ast.push(AST::Call {
                                name,
                                args: Vec::new(),
                                line: current_line,
                            });
                        }

                        AST::Function { name, args, body, line } => {
                            temp_ast.push(AST::Function {
                                name,
                                args,
                                body,
                                line,
                            });
                        }

                        AST::PropertyAccess { object, property, line } => {
                            temp_ast.push(AST::PropertyCall {
                                object,
                                property,
                                args: Vec::new(),
                                line,
                            });
                        }

                        AST::LetDeclaration { name, value, line } => {
                            if let AST::Identifer(ident_name) = *value {
                                temp_ast.push(AST::LetDeclaration {
                                    name: name,
                                    value: Box::new(AST::Call {
                                        name: ident_name,
                                        args: Vec::new(),
                                        line,
                                    }),
                                    line,
                                });
                            } else if let AST::PropertyAccess { object, property, line } = *value {
                                temp_ast.push(AST::LetDeclaration {
                                    name: name,
                                    value: Box::new(AST::PropertyCall {
                                        object,
                                        property,
                                        args: Vec::new(),
                                        line,
                                    }),
                                    line,
                                });
                            } else if let AST::Call { name: call_name, args: call_args, line: call_line } = *value {
                                let new_call = handle_nested_arguments(AST::Call {
                                    name: call_name,
                                    args: call_args,
                                    line: call_line,
                                }, AST::Lparen)?;

                                temp_ast.push(AST::LetDeclaration {
                                    name,
                                    value: Box::new(new_call),
                                    line,
                                });
                            } else {
                                return Err(("Expected a function call before '()'".to_string(), current_line));
                            }
                        }

                        AST::Call { name, args, line } => {
                            let new_call = handle_nested_arguments(AST::Call {
                                name,
                                args,
                                line,
                            }, AST::Lparen)?;

                            temp_ast.push(new_call);
                        }

                        AST::PropertyCall { object, property, args, line } => {
                            let new_call = handle_nested_arguments(AST::PropertyCall {
                                object,
                                property,
                                args,
                                line,
                            }, AST::Lparen)?;

                            temp_ast.push(new_call);
                        }

                        _ => {
                            return Err((format!("Unexpected '()' after {:?}", value), current_line));
                        }
                    }
                }
    
                Ok(Token::String) => {
                    let value = temp_ast.pop().unwrap_or(AST::Null);
    
                    match value {
                        AST::Import { file, as_, line } => {
                            if file.is_none() {
                                temp_ast.push(AST::Import {
                                    file: Some(lexer.slice().to_string()),
                                    as_,
                                    line,
                                });
                            } else {
                                return Err(("Expected a import state before the file path".to_string(), current_line));
                            }
                        }

                        AST::Call { name, mut args, line } => {
                            let arg = args.pop().unwrap_or(AST::Null);

                            match arg {
                                AST::Addition { left, right: _, line } => {
                                    args.push(AST::Addition {
                                        left,
                                        right: Box::new(AST::String(lexer.slice().to_string())),
                                        line,
                                    });
                                }

                                AST::Subtraction { left, right: _, line } => {
                                    args.push(AST::Subtraction {
                                        left,
                                        right: Box::new(AST::String(lexer.slice().to_string())),
                                        line,
                                    });
                                }

                                AST::Call { name: call_name, args: arg_args, line } => {
                                    let new_call = handle_nested_arguments(AST::Call {
                                        name: call_name,
                                        args: arg_args,
                                        line,
                                    }, AST::String(lexer.slice().to_string()))?;

                                    args.push(new_call);
                                }

                                AST::PropertyCall { object, property, args: arg_args, line } => {
                                    let mut new_args = arg_args.clone();
                                    new_args.push(AST::String(lexer.slice().to_string()));

                                    args.push(AST::PropertyCall {
                                        object,
                                        property,
                                        args: new_args,
                                        line,
                                    });
                                }

                                AST::Null => {
                                    args.push(AST::String(lexer.slice().to_string()));
                                }

                                _ => {
                                    args.push(arg);
                                    args.push(AST::String(lexer.slice().to_string()));
                                }
                            }

                            temp_ast.push(AST::Call {
                                name,
                                args,
                                line,
                            });
                        }

                        AST::PropertyCall { object, property, mut args, line } => {
                            args.push(AST::String(lexer.slice().to_string()));

                            temp_ast.push(AST::PropertyCall {
                                object,
                                property,
                                args,
                                line,
                            });
                        }

                        AST::LetDeclaration { name, value, line } => {
                            let val = *value;

                            match val {
                                AST::Addition { left, right: _, line } => {
                                    temp_ast.push(AST::LetDeclaration {
                                        name,
                                        value: Box::new(AST::Addition {
                                            left,
                                            right: Box::new(AST::String(lexer.slice().to_string())),
                                            line,
                                        }),
                                        line,
                                    });
                                }

                                AST::Call { name: call_name, args, line } => {
                                    let mut new_args = args.clone();
                                    new_args.push(AST::String(lexer.slice().to_string()));

                                    temp_ast.push(AST::LetDeclaration {
                                        name,
                                        value: Box::new(AST::Call {
                                            name: call_name,
                                            args: new_args,
                                            line,
                                        }),
                                        line,
                                    });
                                }

                                AST::PropertyCall { object, property, args, line } => {
                                    let mut new_args = args.clone();
                                    new_args.push(AST::String(lexer.slice().to_string()));

                                    temp_ast.push(AST::LetDeclaration {
                                        name,
                                        value: Box::new(AST::PropertyCall {
                                            object,
                                            property,
                                            args: new_args,
                                            line,
                                        }),
                                        line,
                                    });
                                }
                                

                                _ => {
                                    temp_ast.push(AST::LetDeclaration {
                                        name,
                                        value: Box::new(AST::String(lexer.slice().to_string())),
                                        line,
                                    });
                                }
                            }
                        }

                        AST::IfStatement { condition, body, line } => {
                            if let AST::IsEqual { left, right, line } = condition.as_ref() {
                                if let AST::Null = **right {
                                    temp_ast.push(AST::IfStatement {
                                        condition: Box::new(AST::IsEqual {
                                            left: left.clone(),
                                            right: Box::new(AST::String(lexer.slice().to_string())),
                                            line: *line,
                                        }),
                                        body,
                                        line: *line,
                                    });
                                } else {
                                    return Err(("Expected a value before '=='".to_string(), current_line));
                                }
                            } else {
                                if let AST::IsUnequal { left, right, line } = condition.as_ref() {
                                    if let AST::Null = **right {
                                        temp_ast.push(AST::IfStatement {
                                            condition: Box::new(AST::IsUnequal {
                                                left: left.clone(),
                                                right: Box::new(AST::String(lexer.slice().to_string())),
                                                line: *line,
                                            }),
                                            body,
                                            line: *line,
                                        });
                                    } else {
                                        return Err(("Expected a value before '!='".to_string(), current_line));
                                    }
                                } else {
                                    temp_ast.push(AST::IfStatement {
                                        condition,
                                        body,
                                        line,
                                    });
        
                                    temp_ast.push(AST::String(lexer.slice().to_string()));
                                }
                            }
                        }

                        AST::Return { value, line } => {
                            if let AST::Null = *value {
                                temp_ast.push(AST::Return {
                                    value: Box::new(AST::String(lexer.slice().to_string())),
                                    line,
                                });
                            } else {
                                return Err(("Unexpected string after 'return'".to_string(), current_line));
                            }
                        }

                        _ => {
                            return Err(("Expected a call or let declaration before a string".to_string(), current_line));
                        }
                    }
                }

                Ok(Token::Plus) => {
                    let value = temp_ast.pop().unwrap_or(AST::Null);

                    match value {
                        AST::Number(n) => {
                            temp_ast.push(AST::Addition {
                                left: Box::new(AST::Number(n)),
                                right: Box::new(AST::Null),
                                line: current_line,
                            });
                        }

                        AST::Float(f) => {
                            temp_ast.push(AST::Addition {
                                left: Box::new(AST::Float(f)),
                                right: Box::new(AST::Null),
                                line: current_line,
                            });
                        }

                        AST::String(s) => {
                            temp_ast.push(AST::Addition {
                                left: Box::new(AST::String(s)),
                                right: Box::new(AST::Null),
                                line: current_line,
                            });
                        }

                        AST::LetDeclaration { name, value, line } => {
                            if let AST::Call { name: call_name, args, line } = *value {
                                let mut new_args = args.clone();

                                let last_arg = new_args.pop().unwrap_or(AST::Null);

                                new_args.push(AST::Addition {
                                    left: Box::new(last_arg),
                                    right: Box::new(AST::Null),
                                    line,
                                });

                                temp_ast.push(AST::LetDeclaration {
                                    name,
                                    value: Box::new(AST::Call {
                                        name: call_name,
                                        args: new_args,
                                        line,
                                    }),
                                    line,
                                });
                            } else if let AST::PropertyCall { object, property, args, line } = *value {
                                let mut new_args = args.clone();

                                let last_arg = new_args.pop().unwrap_or(AST::Null);

                                new_args.push(AST::Addition {
                                    left: Box::new(last_arg),
                                    right: Box::new(AST::Null),
                                    line,
                                });

                                temp_ast.push(AST::LetDeclaration {
                                    name,
                                    value: Box::new(AST::PropertyCall {
                                        object,
                                        property,
                                        args: new_args,
                                        line,
                                    }),
                                    line,
                                });
                            } else {
                                temp_ast.push(AST::LetDeclaration {
                                    name,
                                    value: Box::new(AST::Addition {
                                        left: value,
                                        right: Box::new(AST::Null),
                                        line,
                                    }),
                                    line,
                                });
                            }
                        }
                        
                        AST::Call { name, mut args, line } => {
                            let arg = args.pop().unwrap_or(AST::Null);

                            match arg {
                                AST::Call { name: call_name, args: arg_args, line } => {
                                    let mut new_arg_args = arg_args.clone();

                                    let last_arg = new_arg_args.pop().unwrap_or(AST::Null);

                                    new_arg_args.push(AST::Addition {
                                        left: Box::new(last_arg),
                                        right: Box::new(AST::Null),
                                        line,
                                    });

                                    args.push(AST::Call {
                                        name: call_name,
                                        args: new_arg_args,
                                        line,
                                    });

                                    temp_ast.push(AST::Call {
                                        name,
                                        args,
                                        line,
                                    });
                                }

                                AST::PropertyCall { object, property, args: arg_args, line } => {
                                    let mut new_arg_args = arg_args.clone();

                                    let last_arg = new_arg_args.pop().unwrap_or(AST::Null);

                                    new_arg_args.push(AST::Addition {
                                        left: Box::new(last_arg),
                                        right: Box::new(AST::Null),
                                        line,
                                    });

                                    args.push(AST::PropertyCall {
                                        object,
                                        property,
                                        args: new_arg_args,
                                        line,
                                    });

                                    temp_ast.push(AST::Call {
                                        name,
                                        args,
                                        line,
                                    });
                                }

                                _ => {
                                    args.push(AST::Addition {
                                        left: Box::new(arg),
                                        right: Box::new(AST::Null),
                                        line,
                                    });
        
                                    temp_ast.push(AST::Call {
                                        name,
                                        args,
                                        line,
                                    });
                                }
                            }
                        }

                        AST::PropertyCall { object, property, mut args, line } => {
                            let arg = args.pop().unwrap_or(AST::Null);

                            match arg {
                                AST::Call { name: call_name, args: arg_args, line } => {
                                    let mut new_arg_args = arg_args.clone();

                                    let last_arg = new_arg_args.pop().unwrap_or(AST::Null);

                                    new_arg_args.push(AST::Addition {
                                        left: Box::new(last_arg),
                                        right: Box::new(AST::Null),
                                        line,
                                    });

                                    args.push(AST::Call {
                                        name: call_name,
                                        args: new_arg_args,
                                        line,
                                    });

                                    temp_ast.push(AST::PropertyCall {
                                        object,
                                        property,
                                        args,
                                        line,
                                    });
                                }

                                AST::PropertyCall { object, property, mut args, line } => {
                                    let arg = args.pop().unwrap_or(AST::Null);

                                    match arg {
                                        AST::Call { name: call_name, args: arg_args, line } => {
                                            let mut new_arg_args = arg_args.clone();
        
                                            let last_arg = new_arg_args.pop().unwrap_or(AST::Null);
        
                                            new_arg_args.push(AST::Addition {
                                                left: Box::new(last_arg),
                                                right: Box::new(AST::Null),
                                                line,
                                            });
        
                                            args.push(AST::Call {
                                                name: call_name,
                                                args: new_arg_args,
                                                line,
                                            });
        
                                            temp_ast.push(AST::PropertyCall {
                                                object,
                                                property,
                                                args,
                                                line,
                                            });
                                        }
        
                                        _ => {
                                            args.push(AST::Addition {
                                                left: Box::new(arg),
                                                right: Box::new(AST::Null),
                                                line,
                                            });
                    
                                            temp_ast.push(AST::PropertyCall {
                                                object,
                                                property,
                                                args,
                                                line,
                                            });
                                        }
                                    }
                                }

                                _ => {
                                    args.push(AST::Addition {
                                        left: Box::new(arg),
                                        right: Box::new(AST::Null),
                                        line,
                                    });
        
                                    temp_ast.push(AST::PropertyCall {
                                        object,
                                        property,
                                        args,
                                        line,
                                    });
                                }
                            }
                        }

                        AST::IfStatement { mut condition, body, line } => {
                            if let AST::IsEqual { left, right, line } = condition.as_ref() {
                                condition = Box::new(AST::IsEqual {
                                    left: left.clone(),
                                    right: Box::new(AST::Addition { left: right.clone(), right: Box::new(AST::Null), line: *line }),
                                    line: *line,
                                });
                            } else {
                                if let AST::IsUnequal { left, right, line } = condition.as_ref() {
                                    condition = Box::new(AST::IsUnequal {
                                        left: left.clone(),
                                        right: Box::new(AST::Addition { left: right.clone(), right: Box::new(AST::Null), line: *line }),
                                        line: *line,
                                    });
                                } else {
                                    return Err(("Expected a value before '+'".to_string(), current_line));
                                }
                            }

                            temp_ast.push(AST::IfStatement {
                                condition,
                                body,
                                line,
                            });
                        }

                        AST::Addition { left, right: _, line } => {
                            temp_ast.push(AST::Addition {
                                left,
                                right: Box::new(AST::Null),
                                line,
                            });
                        }

                        AST::Subtraction { left, right: _, line } => {
                            temp_ast.push(AST::Subtraction {
                                left,
                                right: Box::new(AST::Null),
                                line,
                            });
                        }

                        AST::Return { value, line } => {
                            if let AST::Null = *value {
                                return Err(("Unexpected '+' after 'return'".to_string(), current_line));
                            } else {
                                temp_ast.push(AST::Return {
                                    value: Box::new(AST::Addition {
                                        left: value,
                                        right: Box::new(AST::Null),
                                        line,
                                    }),
                                    line,
                                });
                            }
                        }

                        _ => {
                            return Err((format!("Expected a number, float, string, or let declaration before '+', got {:?}", value), current_line));
                        }
                    }
                }    

                Ok(Token::Minus) => {
                    let value = temp_ast.pop().unwrap_or(AST::Null);

                    match value {
                        AST::Number(n) => {
                            temp_ast.push(AST::Subtraction {
                                left: Box::new(AST::Number(n)),
                                right: Box::new(AST::Null),
                                line: current_line,
                            });
                        }

                        AST::Float(f) => {
                            temp_ast.push(AST::Subtraction {
                                left: Box::new(AST::Float(f)),
                                right: Box::new(AST::Null),
                                line: current_line,
                            });
                        }

                        AST::LetDeclaration { name, value, line } => {
                            match *value {
                                AST::Call { name: call_name, mut args, line } => {
                                    let last_arg = args.pop().unwrap_or(AST::Null);

                                    args.push(AST::Subtraction {
                                        left: Box::new(last_arg),
                                        right: Box::new(AST::Null),
                                        line,
                                    });

                                    temp_ast.push(AST::LetDeclaration {
                                        name,
                                        value: Box::new(AST::Call {
                                            name: call_name,
                                            args,
                                            line,
                                        }),
                                        line,
                                    });
                                }

                                AST::PropertyCall { object, property, mut args, line } => {
                                    let last_arg = args.pop().unwrap_or(AST::Null);

                                    args.push(AST::Subtraction {
                                        left: Box::new(last_arg),
                                        right: Box::new(AST::Null),
                                        line,
                                    });

                                    temp_ast.push(AST::LetDeclaration {
                                        name,
                                        value: Box::new(AST::PropertyCall {
                                            object,
                                            property,
                                            args,
                                            line,
                                        }),
                                        line,
                                    });
                                }

                                _ => {
                                    temp_ast.push(AST::LetDeclaration {
                                        name,
                                        value: Box::new(AST::Subtraction {
                                            left: value,
                                            right: Box::new(AST::Null),
                                            line,
                                        }),
                                        line,
                                    });
                                }
                            }
                        }

                        AST::Call { name, args, line } => {
                            let new_call = handle_nested_arguments(AST::Call {
                                name,
                                args,
                                line,
                            }, AST::Minus)?;

                            temp_ast.push(new_call);
                        }

                        AST::PropertyCall { object, property, args, line } => {
                            let new_call = handle_nested_arguments(AST::PropertyCall {
                                object,
                                property,
                                args,
                                line,
                            }, AST::Minus)?;

                            temp_ast.push(new_call);
                        }

                        AST::IfStatement { mut condition, body, line } => {
                            if let AST::IsEqual { left, right, line } = condition.as_ref() {
                                condition = Box::new(AST::IsEqual {
                                    left: left.clone(),
                                    right: Box::new(AST::Subtraction { left: right.clone(), right: Box::new(AST::Null), line: *line }),
                                    line: *line,
                                });
                            } else {
                                if let AST::IsUnequal { left, right, line } = condition.as_ref() {
                                    condition = Box::new(AST::IsUnequal {
                                        left: left.clone(),
                                        right: Box::new(AST::Subtraction { left: right.clone(), right: Box::new(AST::Null), line: *line }),
                                        line: *line,
                                    });
                                } else {
                                    return Err(("Expected a value before '-'".to_string(), current_line));
                                }
                            }

                            temp_ast.push(AST::IfStatement {
                                condition,
                                body,
                                line,
                            });
                        }

                        AST::Identifer(name) => {
                            temp_ast.push(AST::Subtraction {
                                left: Box::new(AST::Identifer(name)),
                                right: Box::new(AST::Null),
                                line: current_line,
                            });
                        }

                        AST::Subtraction { left, right: _, line } => {
                            temp_ast.push(AST::Subtraction {
                                left,
                                right: Box::new(AST::Null),
                                line,
                            });
                        }

                        AST::Addition { left, right: _, line } => {
                            temp_ast.push(AST::Addition {
                                left,
                                right: Box::new(AST::Null),
                                line,
                            });
                        }

                        AST::Return { value, line } => {
                            temp_ast.push(AST::Return {
                                value: Box::new(AST::Subtraction {
                                    left: value,
                                    right: Box::new(AST::Null),
                                    line,
                                }),
                                line,
                            });
                        }

                        _ => {
                            return Err((format!("Expected a number, float, let or variable declaration before '-', got {:?}", value), current_line));
                        }
                    }
                }

                Ok(Token::Number(n)) => {
                    let value = temp_ast.pop().unwrap_or(AST::Null);

                    match value {
                        AST::Call { name, args, line } => {
                            let new_call = handle_nested_arguments(AST::Call {
                                name,
                                args,
                                line,
                            }, AST::Number(n))?;

                            temp_ast.push(new_call);
                        }

                        AST::PropertyCall { object, property, mut args, line } => {
                            let new_call = handle_nested_arguments(AST::PropertyCall {
                                object,
                                property,
                                args,
                                line,
                            }, AST::Number(n))?;

                            temp_ast.push(new_call);
                        }

                        AST::LetDeclaration { name, value, line } => {
                            if let AST::Addition { left, right: _, line } = *value {
                                temp_ast.push(AST::LetDeclaration {
                                    name,
                                    value: Box::new(AST::Addition {
                                        left,
                                        right: Box::new(AST::Number(n)),
                                        line,
                                    }),
                                    line,
                                });
                            } else if let AST::Subtraction { left, right: _, line } = *value {
                                temp_ast.push(
                                    AST::LetDeclaration {
                                        name,
                                        value: Box::new(AST::Subtraction {
                                            left,
                                            right: Box::new(AST::Number(n)),
                                            line,
                                        }),
                                        line,
                                    }
                                );
                            } else if let AST::Call { name: call_name, args, line } = *value {
                                let new_call = handle_nested_arguments(AST::Call {
                                    name: call_name,
                                    args,
                                    line,
                                }, AST::Number(n))?;

                                temp_ast.push(AST::LetDeclaration {
                                    name,
                                    value: Box::new(new_call),
                                    line,
                                });
                            } else if let AST::PropertyCall { object, property, mut args, line } = *value {
                                let new_call = handle_nested_arguments(AST::PropertyCall {
                                    object,
                                    property,
                                    args,
                                    line,
                                }, AST::Number(n))?;

                                temp_ast.push(AST::LetDeclaration {
                                    name,
                                    value: Box::new(new_call),
                                    line,
                                });
                            } else {
                                temp_ast.push(AST::LetDeclaration {
                                    name,
                                    value: Box::new(AST::Number(n)),
                                    line,
                                });
                            }
                        }

                        AST::Addition { left, right, line } => {
                            if let AST::Null = *right {
                                temp_ast.push(AST::Addition {
                                    left,
                                    right: Box::new(AST::Number(n)),
                                    line,
                                });
                            } else {
                                return Err(("Expected a value before '+'".to_string(), current_line));
                            }
                        }

                        AST::Subtraction { left, right: _, line } => {
                            temp_ast.push(AST::Subtraction {
                                left,
                                right: Box::new(AST::Number(n)),
                                line,
                            });
                        }

                        AST::IfStatement { condition, body, line } => {
                            if let AST::IsEqual { left, right, line } = condition.as_ref() {
                                if let AST::Null = **right {
                                    temp_ast.push(AST::IfStatement {
                                        condition: Box::new(AST::IsEqual {
                                            left: left.clone(),
                                            right: Box::new(AST::Number(n)),
                                            line: *line,
                                        }),
                                        body,
                                        line: *line,
                                    });
                                } else {
                                    if let AST::Addition { left: addition_left, right: _, line } = right.as_ref() {
                                        temp_ast.push(AST::IfStatement {
                                            condition: Box::new(AST::IsEqual {
                                                left: left.clone(),
                                                right: Box::new(AST::Addition {
                                                    left: addition_left.clone(),
                                                    right: Box::new(AST::Number(n)),
                                                    line: *line,
                                                }),
                                                line: *line,
                                            }),
                                            body,
                                            line: *line,
                                        });
                                    } else if let AST::Subtraction { left: subtraction_left, right: _, line } = right.as_ref() {
                                        temp_ast.push(AST::IfStatement {
                                            condition: Box::new(AST::IsEqual {
                                                left: left.clone(),
                                                right: Box::new(AST::Subtraction {
                                                    left: subtraction_left.clone(),
                                                    right: Box::new(AST::Number(n)),
                                                    line: *line,
                                                }),
                                                line: *line,
                                            }),
                                            body,
                                            line: *line,
                                        });
                                    } else {
                                        return Err(("Expected a value before '=='".to_string(), current_line));
                                    }
                                }
                            } else {
                                if let AST::IsUnequal { left, right, line } = condition.as_ref() {
                                    if let AST::Null = **right {
                                        temp_ast.push(AST::IfStatement {
                                            condition: Box::new(AST::IsUnequal {
                                                left: left.clone(),
                                                right: Box::new(AST::Number(n)),
                                                line: *line,
                                            }),
                                            body,
                                            line: *line,
                                        });
                                    } else {
                                        if let AST::Addition { left: addition_left, right: _, line } = right.as_ref() {
                                            temp_ast.push(AST::IfStatement {
                                                condition: Box::new(AST::IsUnequal {
                                                    left: left.clone(),
                                                    right: Box::new(AST::Addition {
                                                        left: addition_left.clone(),
                                                        right: Box::new(AST::Number(n)),
                                                        line: *line,
                                                    }),
                                                    line: *line,
                                                }),
                                                body,
                                                line: *line,
                                            });
                                        } else if let AST::Subtraction { left: subtraction_left, right: _, line } = right.as_ref() {
                                            temp_ast.push(AST::IfStatement {
                                                condition: Box::new(AST::IsUnequal  {
                                                    left: left.clone(),
                                                    right: Box::new(AST::Subtraction {
                                                        left: subtraction_left.clone(),
                                                        right: Box::new(AST::Number(n)),
                                                        line: *line,
                                                    }),
                                                    line: *line,
                                                }),
                                                body,
                                                line: *line,
                                            });
                                        } else {
                                            return Err(("Expected a value before '!='".to_string(), current_line));
                                        }
                                    }
                                } else {
                                    temp_ast.push(AST::IfStatement {
                                        condition,
                                        body,
                                        line,
                                    });
        
                                    temp_ast.push(AST::Number(n));
                                }
                            }
                        }

                        AST::Return { value, line } => {
                            if let AST::Null = *value {
                                temp_ast.push(AST::Return {
                                    value: Box::new(AST::Number(n)),
                                    line,
                                });
                            } else if let AST::Addition { left, right: _, line } = *value {
                                temp_ast.push(AST::Return {
                                    value: Box::new(AST::Addition {
                                        left,
                                        right: Box::new(AST::Number(n)),
                                        line,
                                    }),
                                    line,
                                });
                            } else if let AST::Subtraction { left, right: _, line } = *value {
                                temp_ast.push(AST::Return {
                                    value: Box::new(AST::Subtraction {
                                        left,
                                        right: Box::new(AST::Number(n)),
                                        line,
                                    }),
                                    line,
                                });
                            } else {
                                return Err(("Unexpected number after 'return'".to_string(), current_line));
                            }
                        }

                        _ => {
                            return Err((format!("Expected a call or let declaration before a number, got {:?}", value), current_line));
                        }
                    }
                }
    
                Ok(Token::Boolean) => {
                    let value = temp_ast.pop().unwrap_or(AST::Null);

                    match value {
                        AST::Call { name, mut args, line } => {
                            let last_arg = args.pop().unwrap_or(AST::Null);

                            match last_arg {
                                AST::Call { name: call_name, args: arg_args, line } => {
                                    let mut new_arg_args = arg_args.clone();
                                    new_arg_args.push(AST::Boolean(lexer.slice() == "true"));

                                    args.push(AST::Call {
                                        name: call_name,
                                        args: new_arg_args,
                                        line,
                                    });

                                    temp_ast.push(AST::Call {
                                        name,
                                        args,
                                        line,
                                    });
                                }

                                _ => {
                                    args.push(AST::Boolean(lexer.slice() == "true"));

                                    temp_ast.push(AST::Call {
                                        name,
                                        args,
                                        line,
                                    });
                                }
                            }
                        }

                        AST::PropertyCall { object, property, args, line } => {
                            let mut args = args.clone();

                            args.push(AST::Boolean(lexer.slice() == "true"));

                            temp_ast.push(AST::PropertyCall {
                                object,
                                property,
                                args,
                                line,
                            });
                        }

                        AST::LetDeclaration { name, value: _, line } => {
                            temp_ast.push(AST::LetDeclaration {
                                name,
                                value: Box::new(AST::Boolean(lexer.slice() == "true")),
                                line,
                            });
                        }

                        AST::IfStatement { condition, body, line } => {
                            if let AST::IsEqual { left, right, line } = condition.as_ref() {
                                if let AST::Null = **right {
                                    temp_ast.push(AST::IfStatement {
                                        condition: Box::new(AST::IsEqual {
                                            left: left.clone(),
                                            right: Box::new(AST::Boolean(lexer.slice() == "true")),
                                            line: *line,
                                        }),
                                        body,
                                        line: *line,
                                    });
                                } else {
                                    return Err(("Expected a value before '=='".to_string(), current_line));
                                }
                            } else {
                                if let AST::IsUnequal { left, right, line } = condition.as_ref() {
                                    if let AST::Null = **right {
                                        temp_ast.push(AST::IfStatement {
                                            condition: Box::new(AST::IsUnequal {
                                                left: left.clone(),
                                                right: Box::new(AST::Boolean(lexer.slice() == "true")),
                                                line: *line,
                                            }),
                                            body,
                                            line: *line,
                                        });
                                    } else {
                                        return Err(("Expected a value before '!='".to_string(), current_line));
                                    }
                                } else {
                                    temp_ast.push(AST::IfStatement {
                                        condition,
                                        body,
                                        line,
                                    });
        
                                    temp_ast.push(AST::Boolean(lexer.slice() == "true"));
                                }
                            }
                        }

                        AST::Return { value, line } => {
                            if let AST::Null = *value {
                                temp_ast.push(AST::Return {
                                    value: Box::new(AST::Boolean(lexer.slice() == "true")),
                                    line,
                                });
                            } else {
                                return Err(("Unexpected boolean after 'return'".to_string(), current_line));
                            }
                        }

                        _ => {
                            return Err(("Expected a call or let declaration before a boolean".to_string(), current_line));
                        }
                    }
                }

                Ok(Token::Return) => {
                    if bodies_deep == 0 {
                        return Err(("Unexpected return statement".to_string(), current_line));
                    }

                    temp_ast.push(AST::Return {
                        value: Box::new(AST::Null),
                        line: current_line,
                    });
                }
    
                Ok(Token::Float) => {
                    let value = temp_ast.pop().unwrap_or(AST::Null);
    
                    match value {
                        AST::Call { name, args, line } => {
                            let new_call = handle_nested_arguments(AST::Call {
                                name,
                                args,
                                line,
                            }, AST::Float(lexer.slice().parse().unwrap()))?;

                            temp_ast.push(new_call);
                        }

                        AST::PropertyCall { object, property, args, line } => {
                            let new_call = handle_nested_arguments(AST::PropertyCall {
                                object,
                                property,
                                args,
                                line,
                            }, AST::Float(lexer.slice().parse().unwrap()))?;

                            temp_ast.push(new_call);
                        }

                        AST::LetDeclaration { name, value, line } => {
                            if let AST::Addition { left, right: _, line } = *value {
                                temp_ast.push(AST::LetDeclaration {
                                    name,
                                    value: Box::new(AST::Addition {
                                        left,
                                        right: Box::new(AST::Float(lexer.slice().parse().unwrap())),
                                        line,
                                    }),
                                    line,
                                });
                            } else if let AST::Subtraction { left, right: _, line } = *value {
                                temp_ast.push(
                                    AST::LetDeclaration {
                                        name,
                                        value: Box::new(AST::Subtraction {
                                            left,
                                            right: Box::new(AST::Float(lexer.slice().parse().unwrap())),
                                            line,
                                        }),
                                        line,
                                    }
                                );
                            } else if let AST::Call { name: call_name, mut args, line } = *value {
                                let arg = args.pop().unwrap_or(AST::Null);

                                match arg.clone() {
                                    AST::Addition { left, right, line } => {
                                        if AST::Null == *right {
                                            args.push(AST::Addition {
                                                left,
                                                right: Box::new(AST::Float(lexer.slice().parse().unwrap())),
                                                line,
                                            });
                                        } else {
                                            args.push(arg);
                                            args.push(AST::Float(lexer.slice().parse().unwrap()));
                                        }
                                    }

                                    AST::Subtraction { left, right, line } => {
                                        if AST::Null == *right {
                                            args.push(AST::Subtraction {
                                                left,
                                                right: Box::new(AST::Float(lexer.slice().parse().unwrap())),
                                                line,
                                            });
                                        } else {
                                            args.push(arg);
                                            args.push(AST::Float(lexer.slice().parse().unwrap()));
                                        }
                                    }

                                    AST::Call { name: call_name, args: arg_args, line } => {
                                        let mut new_arg_args = arg_args.clone();
                                        new_arg_args.push(AST::Float(lexer.slice().parse().unwrap()));

                                        args.push(AST::Call {
                                            name: call_name,
                                            args: new_arg_args,
                                            line,
                                        });
                                    }

                                    AST::Null => {
                                        args.push(AST::Float(lexer.slice().parse().unwrap()));
                                    }

                                    _ => {
                                        args.push(arg);
                                        args.push(AST::Float(lexer.slice().parse().unwrap()));
                                    }
                                }

                                temp_ast.push(AST::LetDeclaration {
                                    name,
                                    value: Box::new(AST::Call {
                                        name: call_name,
                                        args,
                                        line,
                                    }),
                                    line,
                                });
                            } else if let AST::PropertyCall { object, property, mut args, line } = *value {
                                let arg = args.pop().unwrap_or(AST::Null);

                                match arg.clone() {
                                    AST::Addition { left, right, line } => {
                                        if AST::Null == *right {
                                            args.push(AST::Addition {
                                                left,
                                                right: Box::new(AST::Float(lexer.slice().parse().unwrap())),
                                                line,
                                            });
                                        } else {
                                            args.push(arg);
                                            args.push(AST::Float(lexer.slice().parse().unwrap()));
                                        }
                                    }

                                    AST::Subtraction { left, right, line } => {
                                        if AST::Null == *right {
                                            args.push(AST::Subtraction {
                                                left,
                                                right: Box::new(AST::Float(lexer.slice().parse().unwrap())),
                                                line,
                                            });
                                        } else {
                                            args.push(arg);
                                            args.push(AST::Float(lexer.slice().parse().unwrap()));
                                        }
                                    }

                                    AST::Call { name: call_name, args: arg_args, line } => {
                                        let mut new_arg_args = arg_args.clone();
                                        new_arg_args.push(AST::Float(lexer.slice().parse().unwrap()));

                                        args.push(AST::Call {
                                            name: call_name,
                                            args: new_arg_args,
                                            line,
                                        });
                                    }

                                    _ => {
                                        args.push(AST::Float(lexer.slice().parse().unwrap()));
                                    }
                                }

                                temp_ast.push(AST::LetDeclaration {
                                    name: name,
                                    value: Box::new(AST::PropertyCall {
                                        object,
                                        property,
                                        args,
                                        line,
                                    }),
                                    line,
                                });
                            } else {
                                temp_ast.push(AST::LetDeclaration {
                                    name,
                                    value: Box::new(AST::Float(lexer.slice().parse().unwrap())),
                                    line,
                                });
                            }
                        }

                        AST::IfStatement { condition, body, line } => {
                            temp_ast.push(AST::IfStatement {
                                condition,
                                body,
                                line,
                            });

                            temp_ast.push(AST::Float(lexer.slice().parse().unwrap()));
                        }

                        AST::Return { value, line } => {
                            if let AST::Null = *value {
                                temp_ast.push(AST::Return {
                                    value: Box::new(AST::Float(lexer.slice().parse().unwrap())),
                                    line,
                                });
                            } else {
                                return Err(("Unexpected float after 'return'".to_string(), current_line));
                            }
                        }

                        AST::Addition { left, right, line } => {
                            if let AST::Null = *right {
                                temp_ast.push(AST::Addition {
                                    left,
                                    right: Box::new(AST::Float(lexer.slice().parse().unwrap())),
                                    line,
                                });
                            } else {
                                return Err(("Expected a value before '+'".to_string(), current_line));
                            }
                        }

                        AST::Subtraction { left, right: _, line } => {
                            temp_ast.push(AST::Subtraction {
                                left,
                                right: Box::new(AST::Float(lexer.slice().parse().unwrap())),
                                line,
                            });
                        }

                        _ => {
                            return Err(("Expected a call or let declaration before a float".to_string(), current_line));
                        }
                    }
                }
    
                Ok(Token::RParen) => {
                    let value = temp_ast.pop().unwrap_or(AST::Null);

                    match value {
                        AST::Call { name, args, line } => {
                            let new_call = handle_nested_arguments(
                                AST::Call { name, args, line }, AST::Rparen
                            )?;

                            temp_ast.push(new_call);
                        }

                        AST::PropertyCall { object, property, args, line } => {
                            let new_call = handle_nested_arguments(
                                AST::PropertyCall { object, property, args, line }, AST::Rparen
                            )?;

                            temp_ast.push(new_call);
                        }

                        AST::Function { name, args, body, line } => {
                            temp_ast.push(AST::Function {
                                name,
                                args,
                                body,
                                line,
                            });
                        }

                        AST::LetDeclaration { name, value, line } => {
                            temp_ast.push(AST::LetDeclaration {
                                name,
                                value,
                                line,
                            });
                        }

                        _ => {
                            return Err((format!("Expected a call or property call before ')', got {:?}", value), current_line));
                        }
                    }
                }
    
                Ok(Token::Semicolon) => {
                    temp_ast.push(AST::Semicolon);
                }

                Ok(Token::LBracket) => {
                    match temp_ast.pop().unwrap_or(AST::Null) {
                        AST::Function { name, args, body, line } => {
                            temp_ast.push(AST::Function {
                                name,
                                args,
                                body,
                                line,
                            });

                            bodies_deep += 1;
                            body_starts = true;
                        }

                        AST::IfStatement { condition, body, line } => {
                            temp_ast.push(AST::IfStatement {
                                condition,
                                body,
                                line,
                            });

                            bodies_deep += 1;
                            body_starts = true;
                        }

                        AST::Identifer(name) => {
                            match temp_ast.pop() {
                                Some(AST::IfStatement { condition: _, body, line }) => {
                                    temp_ast.push(AST::IfStatement {
                                        condition: Box::new(AST::Exists {
                                            value: Box::new(AST::Identifer(name)),
                                            line,
                                        }),
                                        body,
                                        line,
                                    });

                                    bodies_deep += 1;
                                    body_starts = true;
                                }

                                Some(_) => {
                                    return Err(("Expected an if statement before '{'".to_string(), current_line));
                                }

                                None => {
                                    return Err(("Expected an if statement before '{'".to_string(), current_line));
                                }
                            }
                        }

                        _ => {
                            return Err(("Expected a function or if statement before '{'".to_string(), current_line));
                        }
                    }
                }

                Ok(Token::RBracket) => {
                    let value = ast.pop().unwrap_or(AST::Null);

                    match value {
                        AST::Function { name, args, body, line } => {
                            ast.push(AST::Function {
                                name,
                                args,
                                body,
                                line,
                            });

                            bodies_deep -= 1;
                        }

                        AST::IfStatement { condition, body, line } => {
                            ast.push(AST::IfStatement {
                                condition,
                                body,
                                line,
                            });

                            bodies_deep -= 1;
                        }

                        _ => {
                            return Err(("Expected a function or if statement before '}'".to_string(), current_line));
                        }
                    }
                }
    
                Err(err) => {
                    match err {
                        LexingError::UnexpectedToken => {
                            return Err((format!("Unexpected token: {:?}", lexer.slice()), current_line));
    
                        }

                        LexingError::InvalidInteger(str) => {
                            return Err((format!("Could not parse integer: {:?}", str), current_line));
                        }
                    }
                }
    
                _ => {}
            }

            line_map.insert(current_line, ast.clone());
        }
    
        if verbose {
            dbg!(&temp_ast);
        }

        if bodies_deep > 0 && (!body_starts || bodies_deep > 1) {
            ast = handle_nested_ast(ast, temp_ast, current_line)?;
        } else {
            ast.append(&mut temp_ast);
        }

        if verbose {
            dbg!(&ast);
        }
    }

    if verbose {
        dbg!(&ast);
    }

    for item in ast.clone() {
        match item {
            AST::Import { file, as_, line } => {
                let result = eval(AST::Import { file, as_, line }, context);

                if result.is_err() {
                    return Err((result.err().unwrap(), line));
                }
            }

            AST::LetDeclaration { name, value, line } => {
                let cleaned_obj = clean_args(AST::LetDeclaration { name, value, line });

                let result = eval(cleaned_obj, context);

                if result.is_err() {
                    return Err((result.err().unwrap(), line));
                }
            }

            AST::Call { name, args, line } => {
                let cleaned_call = clean_args(AST::Call { name, args, line });

                let result = eval(cleaned_call, context);

                if result.is_err() {
                    return Err((result.err().unwrap(), line));
                }
            }

            AST::PropertyCall { object, property, args, line } => {
                let cleaned_call = clean_args(AST::PropertyCall { object, property, args, line });

                let result = eval(cleaned_call, context);

                if result.is_err() {
                    return Err((result.err().unwrap(), line));
                }
            }

            AST::Function { name, args, body, line } => {
                let cleaned_obj = clean_args(AST::Function { name, args, body, line });

                let result = eval(cleaned_obj, context);

                if result.is_err() {
                    return Err((result.err().unwrap(), line));
                }
            }

            AST::IfStatement { condition, body, line } => {
                let cleaned_obj = clean_args(AST::IfStatement { condition, body, line });

                let result = eval(cleaned_obj, context);

                if result.is_err() {
                    return Err((result.err().unwrap(), line));
                }
            }

            AST::Semicolon => {}
            AST::Null => {}

            _ => {
                if verbose {
                    println!("I'm not sure what to do with a {:?}", item);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn let_str() {
        let mut context = crate::utils::create_context();
        parse("let x = \"test\"", &mut context).unwrap();

        assert_eq!(context.get("x"), Some(&AST::String("test".to_string())));
    }


    #[test]
    fn let_number() {
        let mut context = crate::utils::create_context();
        let result = parse("let x = 10", &mut context);

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn let_boolean() {
        let mut context = crate::utils::create_context();
        let result = parse("let x = true", &mut context);

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn let_float() {
        let mut context = crate::utils::create_context();
        let result = parse("let x = 1.123", &mut context);

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn let_unknown_var() {
        let mut context = crate::utils::create_context();
        let result = parse("let x = y", &mut context);

        assert_eq!(result, Err(("Variable y not found".to_string(), 1)));
    }

    #[test]
    fn print_str() {
        let mut context = crate::utils::create_context();
        let result = parse("print(\"Hello, world!\")", &mut context);

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn print_number() {
        let mut context = crate::utils::create_context();
        let result = parse("print(10)", &mut context);

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn print_float() {
        let mut context = crate::utils::create_context();
        let result = parse("print(1.123)", &mut context);

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn print_boolean() {
        let mut context = crate::utils::create_context();
        let result = parse("print(true)", &mut context);

        assert_eq!(result, Ok(()));

    }

    #[test]
    fn print_var() {
        let mut context = crate::utils::create_context();
        parse("let x = 10", &mut context).unwrap();

        let result = parse("print(x)", &mut context);

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn print_unknown_var() {
        let mut context = crate::utils::create_context();
        let result = parse("print(x)", &mut context);

        assert_eq!(result, Ok(())); // cause prints Null
    }

    #[test]
    fn addition_in_if_statement() {
        let mut context = crate::utils::create_context();
        let left_side = parse("if 1 + 1 == 2 {\n print(\"Hello, world!\") \n}", &mut context);
        let right_side = parse("if 3 == 1 + 2 {\n print(\"Hello, world!\") \n}", &mut context);

        assert_eq!(left_side, Ok(()));
        assert_eq!(right_side, Ok(()));
    }

    #[test]
    fn subtraction_in_if_statement() {
        let mut context = crate::utils::create_context();
        let left_side = parse("if 2 - 1 == 1 {\n print(\"Hello, world!\") \n}", &mut context);
        let right_side = parse("if 3 == 2 - 1 {\n print(\"Hello, world!\") \n}", &mut context);

        assert_eq!(left_side, Ok(()));
        assert_eq!(right_side, Ok(()));
    }

    #[test]
    fn print_multipile_args_seperated_by_comma() {
        let mut context = crate::utils::create_context();
        let result = parse("print(\"Hello, world!\", 10, 1.123, true)", &mut context);

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn print_multipile_args_seperated_by_plus() {
        let mut context = crate::utils::create_context();
        let result = parse("print(\"Hello, world!\" + 10 + 1.123 + true)", &mut context);

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn read_file() {
        let mut context = crate::utils::create_context();
        let result = parse("import \"file\" as f\nprint(f.read(\"input.modu\"))", &mut context);

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn read_file_to_var() {
        let mut context = crate::utils::create_context();
        let result = parse("import \"file\" as f\nlet x = f.read(\"input.modu\")\nprint(x)", &mut context);

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn read_file_import_with_asterisk() {
        let mut context = crate::utils::create_context();
        let result = parse("import \"file\" as *\nprint(read(\"input.modu\"))", &mut context);

        assert_eq!(result, Ok(()));
    }
}