use crate::ast::AST;

use std::{collections::HashMap, path::PathBuf};
use crate::utils;
use crate::packages::get_package;

static DISABLED_ON_SERVER: [&str; 3] = ["file", "os", "ffi"];

pub fn eval(expr: AST, context: &mut HashMap<String, AST>) -> Result<AST, String> {
    match expr {
        AST::Call { name, args, line: _ } => {
            match name.as_str() {
                _ => {
                    match context.get(&name) {
                        Some(value) => {
                            match value {
                                AST::Function { name: _, args: f_args, body, line: _ } => {
                                    if args.len() == f_args.len() || f_args.last().unwrap() == "__args__" {
                                        let mut new_context = context.clone();

                                        for (i, arg) in f_args.iter().enumerate() {
                                            new_context.insert(arg.clone(), eval(args[i].clone(), &mut new_context.clone())?);
                                        }

                                        let mut depth = 0;

                                        for expr in body {
                                            if depth > 100 {
                                                return Err("Maximum recursion depth exceeded".to_string());
                                            }

                                            if let AST::Return { value, line: _ } = expr {
                                                return eval(*value.clone(), &mut new_context);
                                            }

                                            depth += 1;

                                            let ast: AST = eval(expr.clone(), &mut new_context)?;

                                            if let AST::Return { value, line: _ } = ast {
                                                return Ok(*value);
                                            }
                                        }
                                    } else {                                        
                                        return Err(format!("{} takes {} argument(s)", name, f_args.len()));
                                    }
                                }

                                AST::InternalFunction { name: _, args: f_args, call_fn } => {
                                    if args.len() == f_args.len() || f_args.last().unwrap() == "__args__" {
                                        return Ok(call_fn(args, context)?.0);
                                    } else {
                                        return Err(format!("{} takes {} argument(s)", name, f_args.len()));
                                    }
                                }

                                _ => {
                                    return Err(format!("{} is not a function", name));
                                }
                            }
                        }

                        None => {
                            return Err(format!("Function {} not found", name));
                        }
                    }
                }
            }
        }

        AST::LetDeclaration { name, value, line: _ } => {
            if utils::is_reserved(name.as_ref().unwrap_or(&"".to_string())) {
                return Err(format!("{} is a reserved keyword", name.as_ref().unwrap()));
            }

            if let Some(name) = name {
                match *value {
                    AST::Identifer(i_name) => {
                        match context.get(&i_name) {
                            Some(value) => {
                                let val = eval(value.clone(), context)?;
                                context.insert(name, val);
                            }

                            None => {
                                return Err(format!("Variable {} not found", i_name));
                            }
                        }
                    }

                    _ => {
                        let val = eval(*value, context)?;
                        context.insert(name, val);
                    }
                }
            }
        }

        AST::Function { name, args, body, line: _ } => {
            context.insert(name.clone(), AST::Function { name, args, body, line: 0 });
        }

        AST::Semicolon => {
            return Ok(AST::Null);
        }

        AST::Import { file, as_, line } => {
            let args = std::env::args().collect::<Vec<String>>();
            let file = file.unwrap().replace("\"", "");

            let path: PathBuf;

            if args.len() > 2 {
                path = std::path::Path::new(&args[2]).parent().unwrap().join(&file);
            } else {
                path = file.clone().into();
            }

            if file.ends_with(".modu") {
                match std::fs::read_to_string(&path) {
                    Ok(file) => {
                        let mut new_context = context.clone();
    
                        match crate::parser::parse(&file, &mut new_context) {
                            Ok(_) => {
                                let insert_as = as_.unwrap();

                                if insert_as == "*" {
                                    for (name, value) in new_context {
                                        context.insert(name, value);
                                    }
    
                                    return Ok(AST::Null);
                                } else {
                                    context.insert(insert_as, AST::Object { properties: new_context, line });
                                }
                            }
    
                            Err(e) => {
                                return Err(e.0);
                            }
                        }
                    }
    
                    Err(e) => {
                        dbg!(path);
    
                        return Err(e.to_string());
                    }
                }
            } else {
                let args = std::env::args().collect::<Vec<String>>();

                if args.len() > 1 && args[1] == "server" {
                    if DISABLED_ON_SERVER.contains(&file.as_str()) {
                        return Err(format!("{} is disabled on the server", file));
                    }
                }

                let package = get_package(&file);

                if let Some(package) = package {
                    if let AST::Object { properties, line } = package {
                        let insert_as = as_.unwrap();

                        if insert_as == "*" {
                            for (name, value) in properties {
                                context.insert(name, value);
                            }
                        } else {
                            context.insert(insert_as, AST::Object { properties, line });
                        }
                    }
                } else {
                    if std::fs::exists(format!(".modu/packages/{}", file)).unwrap() {
                        let mut new_context = context.clone();

                        let content = std::fs::read_to_string(format!(".modu/packages/{}/lib.modu", file)).unwrap();

                        match crate::parser::parse(&content, &mut new_context) {
                            Ok(_) => {
                                let insert_as = as_.unwrap();

                                if insert_as == "*" {
                                    for (name, value) in new_context {
                                        context.insert(name, value);
                                    }

                                    return Ok(AST::Null);
                                } else {
                                    context.insert(insert_as, AST::Object { properties: new_context, line });
                                }
                            }

                            _ => {
                                return Err(format!("Failed to parse package {}", file));
                            }
                        }
                    } else {
                        return Err(format!("Package {} not found", file));
                    }
                }
            }
        }

        AST::PropertyCall { object, property, args, line: _ } => {
            match object.clone() {
                Some(name) => {
                    match context.get(&name) {
                        Some(value) => {
                            match value {
                                AST::Object { properties, line: _ } => {
                                    match properties.get(property.as_ref().unwrap()) {
                                        Some(value) => {
                                            match value {
                                                AST::Function { name, args: f_args, body, line: _ } => {
                                                    if args.len() == f_args.len() || f_args.last().unwrap() == "__args__" {
                                                        let mut new_context = context.clone();

                                                        for (i, arg) in f_args.iter().enumerate() {
                                                            new_context.insert(arg.clone(), eval(args[i].clone(), &mut new_context.clone())?);
                                                        }

                                                        new_context.remove(name);
                                                        
                                                        for prop in properties {
                                                            new_context.insert(prop.0.clone(), prop.1.clone());
                                                        }

                                                        for expr in body {
                                                            if let AST::Return { value, line: _ } = expr {
                                                                return eval(*value.clone(), &mut new_context);
                                                            }

                                                            let ast = eval(expr.clone(), &mut new_context)?;

                                                            if let AST::Return { value, line: _ } = ast {
                                                                return Ok(*value);
                                                            }
                                                        }
                                                    } else {
                                                        return Err(format!("{} takes {} argument(s)", name, f_args.len()));
                                                    }
                                                }

                                                AST::InternalFunction { name, args: f_args, call_fn } => {
                                                    if args.len() == f_args.len() || f_args.last().unwrap() == "__args__" {
                                                        return Ok(call_fn(args, context)?.0);
                                                    } else if f_args[0] == "self" && args.len() == f_args.len() - 1 || f_args.last().unwrap() == "__args__" {
                                                            let mut new_args = vec![AST::Object { properties: properties.clone(), line: 0 }];

                                                            for arg in args {
                                                                new_args.push(arg);
                                                            }

                                                            let result = call_fn(new_args, context)?;

                                                            match result.1.clone() {
                                                                AST::Object { properties, line: _ } => {
                                                                    context.insert(object.unwrap(), AST::Object { properties, line: 0 });
                                                                }

                                                                _ => {}
                                                            }

                                                            return Ok(result.0);
                                                    } else if f_args[0] == "self" {
                                                        return Err(format!("{} takes {} argument(s)", name, f_args.len() - 1));
                                                    } else {
                                                        return Err(format!("{} takes {} argument(s)", name, f_args.len()));
                                                    }
                                                }

                                                _ => {
                                                    return Err(format!("{} on object {} is not a function", property.as_ref().unwrap(), name));
                                                }
                                            }
                                        }

                                        None => {
                                            return Err(format!("Property {} not found in object {}", property.as_ref().unwrap(), name));
                                        }
                                    }
                                }

                                _ => {
                                    return Err(format!("{} is not an object", name));
                                }
                            }
                        }

                        None => {
                            return Err(format!("Object {} not found", name));
                        }
                    }
                }

                None => {
                    return Err("Object appears to be null".to_string());
                }
            }
        }

        AST::IsEqual { left, right, line: _ } => {
            match (eval(*left, context)?, eval(*right, context)?) {
                (AST::Number(l), AST::Number(r)) => {
                    return Ok(AST::Boolean(l == r));
                }

                (AST::Float(l), AST::Float(r)) => {
                    return Ok(AST::Boolean(l == r));
                }

                (AST::String(l), AST::String(r)) => {
                    return Ok(AST::Boolean(l == r));
                }

                (AST::Boolean(l), AST::Boolean(r)) => {
                    return Ok(AST::Boolean(l == r));
                }

                _ => {
                    return Ok(AST::Boolean(false));
                }
            }
        }

        AST::IsUnequal { left, right, line: _ } => {
            match (eval(*left, context)?, eval(*right, context)?) {
                (AST::Number(l), AST::Number(r)) => {
                    return Ok(AST::Boolean(l != r));
                }

                (AST::Float(l), AST::Float(r)) => {
                    return Ok(AST::Boolean(l != r));
                }

                (AST::String(l), AST::String(r)) => {
                    return Ok(AST::Boolean(l != r));
                }

                (AST::Boolean(l), AST::Boolean(r)) => {
                    return Ok(AST::Boolean(l != r));
                }

                _ => {
                    return Ok(AST::Boolean(true));
                }
            }
        }

        AST::LessThan { left, right, line: _ } => {
            match (eval(*left, context)?, eval(*right, context)?) {
                (AST::Number(l), AST::Number(r)) => {
                    return Ok(AST::Boolean(l < r));
                }

                (AST::Float(l), AST::Float(r)) => {
                    return Ok(AST::Boolean(l < r));
                }

                (val, val2) => {
                    return Err(format!("Cannot compare {:?} and {:?}", val, val2));
                }
            }
        }

        AST::GreaterThan { left, right, line: _ } => {
            match (eval(*left, context)?, eval(*right, context)?) {
                (AST::Number(l), AST::Number(r)) => {
                    return Ok(AST::Boolean(l > r));
                }

                (AST::Float(l), AST::Float(r)) => {
                    return Ok(AST::Boolean(l > r));
                }

                (val, val2) => {
                    return Err(format!("Cannot compare {:?} and {:?}", val, val2));
                }
            }
        }

        AST::LessThanOrEqual { left, right, line: _ } => {
            match (eval(*left, context)?, eval(*right, context)?) {
                (AST::Number(l), AST::Number(r)) => {
                    return Ok(AST::Boolean(l <= r));
                }

                (AST::Float(l), AST::Float(r)) => {
                    return Ok(AST::Boolean(l <= r));
                }

                (val, val2) => {
                    return Err(format!("Cannot compare {:?} and {:?}", val, val2));
                }
            }
        }

        AST::GreaterThanOrEqual { left, right, line: _ } => {
            match (eval(*left, context)?, eval(*right, context)?) {
                (AST::Number(l), AST::Number(r)) => {
                    return Ok(AST::Boolean(l >= r));
                }

                (AST::Float(l), AST::Float(r)) => {
                    return Ok(AST::Boolean(l >= r));
                }

                (val, val2) => {
                    return Err(format!("Cannot compare {:?} and {:?}", val, val2));
                }
            }
        }

        AST::Exists { value, line: _ } => {
            match eval(*value, context)? {
                AST::Null => {
                    return Ok(AST::Boolean(false));
                }

                AST::Boolean(false) => {
                    return Ok(AST::Boolean(false));
                }

                _ => {
                    return Ok(AST::Boolean(true));
                }
            }
        }

        AST::IfStatement { condition, body, line: _ } => {
            match eval(*condition, context)? {
                AST::Boolean(b) => {
                    if b {
                        for expr in body {
                            if let AST::Return { value, line } = expr {
                                return Ok(AST::Return { value, line });
                            }

                            eval(expr, context)?;
                        }
                    }
                }

                _ => {
                    return Err("If statement condition must return a boolean".to_string());
                }
            }
        }

        AST::Number(_) | AST::Boolean(_) | AST::Float(_) | AST::Object { .. } | AST::Null => {
            return Ok(expr);
        }

        AST::String(value) => {
            return Ok(AST::String(escape_sequence_helper(value)));
        }

        AST::Addition { left, right, line: _ } => {
            match (eval(*left.clone(), context)?, eval(*right.clone(), context)?) {
                (AST::Number(l), AST::Number(r)) => {
                    return Ok(AST::Number(l + r));
                }

                (AST::Float(l), AST::Float(r)) => {
                    return Ok(AST::Float(l + r));
                }

                (AST::Number(l), AST::Float(r)) => {
                    return Ok(AST::Float(l as f64 + r));
                }

                (AST::Float(l), AST::Number(r)) => {
                    return Ok(AST::Float(l + r as f64));
                }

                (AST::String(l), AST::String(r)) => {
                    return Ok(AST::String(format!("{}{}", l, r)));
                }

                _ => {
                    return Err(format!("Cannot add {:?} and {:?}", eval(*left, context)?, eval(*right, context)?));
                }
            }
        }

        AST::Subtraction { left, right, line: _ } => {
            match (eval(*left.clone(), context)?, eval(*right.clone(), context)?) {
                (AST::Number(l), AST::Number(r)) => {
                    return Ok(AST::Number(l - r));
                }

                (AST::Float(l), AST::Float(r)) => {
                    return Ok(AST::Float(l - r));
                }

                (AST::Number(l), AST::Float(r)) => {
                    return Ok(AST::Float(l as f64 - r));
                }

                (AST::Float(l), AST::Number(r)) => {
                    return Ok(AST::Float(l - r as f64));
                }

                (AST::Null, AST::Number(r)) => {
                    return Ok(AST::Number(-r));
                }

                (AST::Null, AST::Float(r)) => {
                    return Ok(AST::Float(-r));
                }

                (AST::Number(l), AST::Null) => {
                    return Ok(AST::Number(l));
                }

                (AST::Float(l), AST::Null) => {
                    return Ok(AST::Float(l));
                }

                _ => {
                    return Err(format!("Cannot subtract {:?} and {:?}", eval(*left, context)?, eval(*right, context)?));
                }
            }
        }

        AST::Identifer(name) => {
            match context.get(&name) {
                Some(value) => {
                    return Ok(value.clone());
                }

                None => {
                    return Ok(AST::Null);
                }
            }
        }

        AST::PropertyAccess { object, property, line: _ } => {
            match object {
                Some(name) => {
                    match context.get(&name) {
                        Some(value) => {
                            match value {
                                AST::Object { properties, line: _ } => {
                                    match properties.get(property.as_ref().unwrap()) {
                                        Some(value) => {
                                            return Ok(value.clone());
                                        }

                                        None => {
                                            return Err(format!("Property {:?} not found", property));
                                        }
                                    }
                                }

                                _ => {
                                    return Err(format!("{} is not an object", name));
                                }
                            }
                        }

                        None => {
                            return Err(format!("Variable {} not found", name));
                        }
                    }
                }

                None => {
                    return Err("Object not found".to_string());
                }
            }
        }

        AST::Return { value, line: _ } => {
            return Ok(*value);
        }

        _ => {
            return Err(format!("Unknown expression, got {:?}", expr));
        }
    }

    Ok(AST::Null)
}

pub fn escape_sequence_helper(s: String) -> String {
    let mut iter = s.chars().peekable();
    let mut s2 = Vec::<char>::new();
    while let Some(ch) = iter.next() {
        match ch {
            '\\' => {
                let x = iter.next_if_eq(&'x');
                if let Some(_) = x {
                    let d1ornone = iter.next_if(|&c| c.is_ascii_hexdigit());
                    if let Some(d1) = d1ornone {
                        let d2ornone = iter.next_if(|&c| c.is_ascii_hexdigit());
                        if let Some(d2) = d2ornone {
                            let n1 = match d1.to_ascii_lowercase() {
                                '0' => 0,
                                '1' => 1,
                                '2' => 2,
                                '3' => 3,
                                '4' => 4,
                                '5' => 5,
                                '6' => 6,
                                '7' => 7,
                                '8' => 8,
                                '9' => 9,
                                'a' => 10,
                                'b' => 11,
                                'c' => 12,
                                'd' => 13,
                                'e' => 14,
                                'f' => 15,
                                _ => 0,
                            };
                            let n2 = match d2.to_ascii_lowercase() {
                                '0' => 0,
                                '1' => 1,
                                '2' => 2,
                                '3' => 3,
                                '4' => 4,
                                '5' => 5,
                                '6' => 6,
                                '7' => 7,
                                '8' => 8,
                                '9' => 9,
                                'a' => 10,
                                'b' => 11,
                                'c' => 12,
                                'd' => 13,
                                'e' => 14,
                                'f' => 15,
                                _ => 0,
                            };
                            s2.push(char::from_u32(((n1 * 16) + n2) as u32).unwrap_or('0'));
                            continue;
                        }
                        s2.push('\\');
                        s2.push('x');
                        s2.push(d1);
                        continue;
                    }
                    s2.push('\\');
                    s2.push('x');
                    continue;
                }
                s2.push('\\');
            }
            _ => s2.push(ch),
        }
    }
    let s3 = s2.iter().collect::<String>()
        .replace("\\0", "\0")
        .replace("\\\"", "\"")
        .replace("\\\'", "\'")
        .replace("\\\\", "\\")
        .replace("\\a", "\x07")
        .replace("\\b", "\x08")
        .replace("\\f", "\x0c")
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\t", "\t")
        .replace("\\v", "\x0b");
    s3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_variable() {
        let mut context = crate::utils::create_context();

        let expr = AST::Identifer("unknown".to_string());

        assert_eq!(eval(expr, &mut context).unwrap(), AST::Null);
    }

    #[test]
    fn unknown_function() {
        let mut context = crate::utils::create_context();

        let expr = AST::Call { name: "cookie".to_string(), args: vec![], line: 0 };

        match eval(expr, &mut context) {
            Ok(_) => {
                assert!(false);
            }

            Err(e) => {
                assert_eq!(e, "Function cookie not found");
            }
        }
    }

    #[test]
    fn addition() {
        let mut context = crate::utils::create_context();

        let expr = AST::Addition { left: Box::new(AST::Number(1)), right: Box::new(AST::Number(2)), line: 0 };

        assert_eq!(eval(expr, &mut context).unwrap(), AST::Number(3));
    }

    #[test]
    fn subtraction() {
        let mut context = crate::utils::create_context();

        let expr = AST::Subtraction { left: Box::new(AST::Number(1)), right: Box::new(AST::Number(2)), line: 0 };

        assert_eq!(eval(expr, &mut context).unwrap(), AST::Number(-1));
    }

    #[test]
    fn negative_num() {
        let mut context = crate::utils::create_context();

        let expr = AST::Subtraction { left: Box::new(AST::Null), right: Box::new(AST::Number(2)), line: 0 };

        assert_eq!(eval(expr, &mut context).unwrap(), AST::Number(-2));
    }

    #[test]
    fn join_strings() {
        let mut context = crate::utils::create_context();

        let expr = AST::Addition { left: Box::new(AST::String("Hello,".to_string())), right: Box::new(AST::String(" World!".to_string())), line: 0 };

        assert_eq!(eval(expr, &mut context).unwrap(), AST::String("Hello, World!".to_string()));
    }

    #[test]
    fn add_floats() {
        let mut context = crate::utils::create_context();

        let expr = AST::Addition { left: Box::new(AST::Float(1.0)), right: Box::new(AST::Float(2.0)), line: 0 };

        assert_eq!(eval(expr, &mut context).unwrap(), AST::Float(3.0));
    }

    #[test]
    fn add_float_and_int() {
        let mut context = crate::utils::create_context();

        let expr = AST::Addition { left: Box::new(AST::Float(1.0)), right: Box::new(AST::Number(2)), line: 0 };

        assert_eq!(eval(expr, &mut context).unwrap(), AST::Float(3.0));
    }

    #[test]
    fn add_int_and_string() {
        let mut context = crate::utils::create_context();

        let expr = AST::Addition { left: Box::new(AST::Number(1)), right: Box::new(AST::String(" cookie".to_string())), line: 0 };

        match eval(expr, &mut context) {
            Ok(_) => {
                assert!(false);
            }

            Err(e) => {
                assert_eq!(e, "Cannot add Number(1) and String(\" cookie\")");
            }
        }
    }
}