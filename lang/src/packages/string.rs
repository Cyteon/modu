use std::collections::HashMap;

use crate::ast::AST;
use crate::eval::eval;

// TODO: extend this package in a future commit/version

pub fn get_object() -> HashMap<String, AST> {
    let mut objects = HashMap::new();

    objects.insert(
        "charat".to_string(),
        AST::InternalFunction {
            name: "charat".to_string(),
            args: vec!["s".to_string(), "i".to_string()],
            call_fn: charat
        }
    );

    objects.insert(
        "codepointat".to_string(),
        AST::InternalFunction {
            name: "codepointat".to_string(),
            args: vec!["s".to_string(), "i".to_string()],
            call_fn: codepointat
        }
    );

    objects.insert(
        "concat".to_string(),
        AST::InternalFunction {
            name: "concat".to_string(),
            args: vec!["a".to_string(), "b".to_string()],
            call_fn: concat
        }
    );

    objects.insert(
        "length".to_string(),
        AST::InternalFunction {
            name: "length".to_string(),
            args: vec!["s".to_string()],
            call_fn: length
        }
    );

    objects.insert(
        "endswith".to_string(),
        AST::InternalFunction {
            name: "endswith".to_string(),
            args: vec!["s".to_string(), "suf".to_string()],
            call_fn: endswith
        }
    );

    objects.insert(
        "includes".to_string(),
        AST::InternalFunction {
            name: "includes".to_string(),
            args: vec!["s".to_string(), "sub".to_string()],
            call_fn: includes
        }
    );

    objects.insert(
        "indexof".to_string(),
        AST::InternalFunction {
            name: "indexof".to_string(),
            args: vec!["s".to_string(), "sub".to_string()],
            call_fn: indexof
        }
    );

    objects.insert(
        "rep".to_string(),
        AST::InternalFunction {
            name: "rep".to_string(),
            args: vec!["s".to_string(), "i".to_string()],
            call_fn: rep
        }
    );

    objects.insert(
        "capitalize".to_string(),
        AST::InternalFunction {
            name: "capitalize".to_string(),
            args: vec!["s".to_string()],
            call_fn: capitalize
        }
    );

    objects.insert(
        "lower".to_string(),
        AST::InternalFunction {
            name: "lower".to_string(),
            args: vec!["s".to_string()],
            call_fn: lower
        }
    );

    objects.insert(
        "empty".to_string(),
        AST::InternalFunction {
            name: "empty".to_string(),
            args: vec![],
            call_fn: empty
        }
    );

    objects.insert(
        "isempty".to_string(),
        AST::InternalFunction {
            name: "isempty".to_string(),
            args: vec!["s".to_string()],
            call_fn: isempty
        }
    );

    objects.insert(
        "is".to_string(),
        AST::InternalFunction {
            name: "is".to_string(),
            args: vec!["s".to_string(), "op".to_string()],
            call_fn: is
        }
    );

    objects
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_object_test() {
        let object = get_object();

        assert_eq!(object.len(), 13);
    }
}

// Functions

pub fn charat(args: Vec<AST>, context: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match (eval(args[0].clone(), context), eval(args[1].clone(), context)) {
        (Ok(AST::String(s)), Ok(AST::Number(i))) => {
            match s.len() {
                l if l as i64 >= i || i < 0 => Err("index out of bounds".to_string()),
                _ => Ok((AST::String(s.chars().nth(i as usize).unwrap().to_string()), AST::Null))
            }
        }

        _ => Err(format!("string.charat requires a string and an integer, got {} and {}", args[0], args[1]).to_string())
    }
}

pub fn codepointat(args: Vec<AST>, context: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match (eval(args[0].clone(), context), eval(args[1].clone(), context)) {
        (Ok(AST::String(s)), Ok(AST::Number(i))) => {
            match s.len() {
                l if l as i64 >= i || i < 0 => Err("index out of bounds".to_string()),
                _ => Ok((AST::Number(s.chars().nth(i as usize).unwrap() as i64), AST::Null))
            }
        }

        _ => Err(format!("string.codepointat requires a string and an integer, got {} and {}", args[0], args[1]).to_string())
    }
}

pub fn concat(args: Vec<AST>, context: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match (eval(args[0].clone(), context), eval(args[1].clone(), context)) {
        (Ok(AST::String(a)), Ok(AST::String(b))) => Ok((AST::String(a.clone() + &b.clone()), AST::Null)),

        _ => Err(format!("string.concat requires a string and a string, got {} and {}", args[0], args[1]).to_string())
    }
}

pub fn length(args: Vec<AST>, context: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), context) {
        Ok(AST::String(s)) => Ok((AST::Number(s.len() as i64), AST::Null)),

        _ => Err(format!("string.length requires a string, got {}", args[0]))
    }
}

pub fn endswith(args: Vec<AST>, context: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match (eval(args[0].clone(), context), eval(args[1].clone(), context)) {
        (Ok(AST::String(s)), Ok(AST::String(suf))) => Ok((AST::Boolean(s.ends_with(&suf.clone())), AST::Null)),

        _ => Err(format!("string.endswith requires two strings, got {} and {}", args[0], args[1]))
    }
}

pub fn includes(args: Vec<AST>, context: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match (eval(args[0].clone(), context), eval(args[1].clone(), context)) {
        (Ok(AST::String(s)), Ok(AST::String(suf))) => Ok((AST::Boolean(s.contains(&suf.clone())), AST::Null)),

        _ => Err(format!("string.endswith requires two strings, got {} and {}", args[0], args[1]))
    }
}

pub fn indexof(args: Vec<AST>, context: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match (eval(args[0].clone(), context), eval(args[1].clone(), context)) {
        (Ok(AST::String(s)), Ok(AST::String(suf))) => {
            match s.find(&suf.clone()) {
                None => Ok((AST::Number(-1), AST::Null)),
                Some(i) => Ok((AST::Number(i as i64), AST::Null))
            }
        },

        _ => Err(format!("string.endswith requires two strings, got {} and {}", args[0], args[1]))
    }
}

pub fn rep(args: Vec<AST>, context: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match (eval(args[0].clone(), context), eval(args[1].clone(), context)) {
        (Ok(AST::String(s)), Ok(AST::Number(i))) => {
            if i < 0 {
                return Err("string.rep requires a non-negative integer as the second argument".to_string());
            }
            let mut res = "".to_string();
            if i > 0 {
                for _ in 0..i {
                    res += &s.clone();
                }
            }
            Ok((AST::String(res), AST::Null))
        },

        _ => Err(format!("string.rep requires a string and an integer, got {} and {}", args[0], args[1]))
    }
}

pub fn capitalize(args: Vec<AST>, context: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), context) {
        Ok(AST::String(s)) => Ok((AST::String(s.to_uppercase()), AST::Null)),

        _ => Err(format!("string.capitalize requires a string, got {}", args[0]))
    }
}

pub fn lower(args: Vec<AST>, context: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), context) {
        Ok(AST::String(s)) => Ok((AST::String(s.to_lowercase()), AST::Null)),

        _ => Err(format!("string.capitalize requires a string, got {}", args[0]))
    }
}

pub fn empty(_: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    Ok((AST::String("".to_string()), AST::Null))
}

pub fn isempty(args: Vec<AST>, context: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), context) {
        Ok(AST::String(s)) => Ok((AST::Boolean(s == ""), AST::Null)),

        _ => Err(format!("string.isempty requires a string, got {}", args[0]))
    }
}

pub fn is(args: Vec<AST>, context: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match (eval(args[0].clone(), context), eval(args[1].clone(), context)) {
        (Ok(AST::String(s)), Ok(AST::String(op))) => {
            match op {
                o if o == "alpha" => Ok((AST::Boolean(s.clone().chars().all(|ch| ch.is_alphabetic())), AST::Null)),
                o if o == "alnum" => Ok((AST::Boolean(s.clone().chars().all(|ch| ch.is_alphanumeric())), AST::Null)),
                o if o == "ascii" => Ok((AST::Boolean(s.clone().chars().all(|ch| ch.is_ascii())), AST::Null)),
                o if o == "control" => Ok((AST::Boolean(s.clone().chars().all(|ch| ch.is_control())), AST::Null)),
                o if o == "lower" => Ok((AST::Boolean(s.clone().chars().all(|ch| ch.is_lowercase())), AST::Null)),
                o if o == "numeric" => Ok((AST::Boolean(s.clone().chars().all(|ch| ch.is_numeric())), AST::Null)),
                o if o == "upper" => Ok((AST::Boolean(s.clone().chars().all(|ch| ch.is_uppercase())), AST::Null)),
                o if o == "wspace" => Ok((AST::Boolean(s.clone().chars().all(|ch| ch.is_whitespace())), AST::Null)),
                o if o == "aalpha" => Ok((AST::Boolean(s.clone().chars().all(|ch| ch.is_ascii_alphabetic())), AST::Null)),
                o if o == "aalnum" => Ok((AST::Boolean(s.clone().chars().all(|ch| ch.is_ascii_alphanumeric())), AST::Null)),
                o if o == "acontrol" => Ok((AST::Boolean(s.clone().chars().all(|ch| ch.is_ascii_control())), AST::Null)),
                o if o == "adigit" => Ok((AST::Boolean(s.clone().chars().all(|ch| ch.is_ascii_digit())), AST::Null)),
                o if o == "agraphic" => Ok((AST::Boolean(s.clone().chars().all(|ch| ch.is_ascii_graphic())), AST::Null)),
                o if o == "ahdigit" => Ok((AST::Boolean(s.clone().chars().all(|ch| ch.is_ascii_hexdigit())), AST::Null)),
                o if o == "alower" => Ok((AST::Boolean(s.clone().chars().all(|ch| ch.is_ascii_lowercase())), AST::Null)),
                o if o == "aodigit" => Ok((AST::Boolean(s.clone().chars().all(|ch| ch.is_ascii_octdigit())), AST::Null)),
                o if o == "apunc" => Ok((AST::Boolean(s.clone().chars().all(|ch| ch.is_ascii_punctuation())), AST::Null)),
                o if o == "aupper" => Ok((AST::Boolean(s.clone().chars().all(|ch| ch.is_ascii_uppercase())), AST::Null)),
                o if o == "awspace" => Ok((AST::Boolean(s.clone().chars().all(|ch| ch.is_ascii_whitespace())), AST::Null)),
                _ => Err("invalid operation".to_string())
            }
        },

        _ => Err(format!("string.is requires a string and an operation (represented by a string), got {}, {}", args[0], args[1]))
    }
}