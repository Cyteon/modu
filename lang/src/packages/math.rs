use std::collections::HashMap;
use rand;

use crate::ast::AST;
use crate::eval::eval;
use crate::packages::string::rep as str_rep;

pub fn div(args: Vec<AST>, context: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match (eval(args[0].clone(), context), eval(args[1].clone(), context)) {
        (Ok(AST::Number(a)), Ok(AST::Number(b))) => {
            if b == 0 {
                return Err("cannot divide by zero".to_string());
            }

            let result = a as f64 / b as f64;

            if result.fract() == 0.0 {
                return Ok((AST::Number(result as i64), AST::Null));
            }

            return Ok((AST::Float(result), AST::Null));
        }

        (Ok(AST::Float(a)), Ok(AST::Float(b))) => {
            if b == 0.0 {
                return Err("cannot divide by zero".to_string());
            }

            return Ok((AST::Float(a / b), AST::Null));
        }

        (Ok(AST::Float(a)), Ok(AST::Number(b))) => {
            if b == 0 {
                return Err("cannot divide by zero".to_string());
            }

            return Ok((AST::Float(a / b as f64), AST::Null));
        }

        (Ok(AST::Number(a)), Ok(AST::Float(b))) => {
            if b == 0.0 {
                return Err("cannot divide by zero".to_string());
            }

            return Ok((AST::Float(a as f64 / b), AST::Null));
        }

        _ => {
            return Err(format!("div requires 2 numbers, got {} and {}", args[0], args[1]));
        }
    }
}

pub fn mul(args: Vec<AST>, context: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match (eval(args[0].clone(), context), eval(args[1].clone(), context)) {
        (Ok(AST::Number(a)), Ok(AST::Number(b))) => {
            return Ok((AST::Number(a * b), AST::Null));
        }

        (Ok(AST::Float(a)), Ok(AST::Float(b))) => {
            return Ok((AST::Float(a * b), AST::Null));
        }

        (Ok(AST::Float(a)), Ok(AST::Number(b))) => {
            return Ok((AST::Float(a * b as f64), AST::Null));
        }

        (Ok(AST::Number(a)), Ok(AST::Float(b))) => {
            return Ok((AST::Float(a as f64 * b), AST::Null));
        }

        (Ok(AST::String(s)), Ok(AST::Number(n))) => {
            return Ok(str_rep(vec![AST::String(s), AST::Number(n)], context)?);
        }

        (Ok(AST::Number(n)), Ok(AST::String(s))) => {
            return Ok(str_rep(vec![AST::String(s), AST::Number(n)], context)?);
        }

        _ => {
            return Err("mul requires 2 numbers".to_string());
        }
    }
}

pub fn abs(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => {
            return Ok((AST::Number(a.abs()), AST::Null));
        }

        Ok(AST::Float(a)) => {
            return Ok((AST::Float(a.abs()), AST::Null));
        }

        _ => {
            return Err("abs requires a number".to_string());
        }
    }
}

pub fn sqrt(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => {
            if a < 0 {
                return Err("sqrt requires a positive number".to_string());
            }

            return Ok((AST::Float((a as f64).sqrt()), AST::Null));
        }

        Ok(AST::Float(a)) => {
            if a < 0.0 {
                return Err("sqrt requires a positive number".to_string());
            }

            return Ok((AST::Float(a.sqrt()), AST::Null));
        }

        _ => {
            return Err("sqrt requires a number".to_string());
        }
    }
}

pub fn pow(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => {
            match eval(args[1].clone(), &mut HashMap::new()) {
                Ok(AST::Number(b)) => {
                    if b < 0 {
                        return Err("pow requires a positive number".to_string());
                    }

                    return Ok((AST::Number(a.pow(b as u32)), AST::Null));
                }

                Ok(AST::Float(b)) => {
                    if b < 0.0 {
                        return Err("pow requires a positive number".to_string());
                    }

                    return Ok((AST::Float((a as f64).powf(b)), AST::Null));
                }

                _ => {
                    return Err("pow requires a number".to_string());
                }
            }
        }

        Ok(AST::Float(a)) => {
            match eval(args[1].clone(), &mut HashMap::new()) {
                Ok(AST::Number(b)) => {
                    return Ok((AST::Float(a.powf(b as f64)), AST::Null));
                }

                Ok(AST::Float(b)) => {
                    return Ok((AST::Float(a.powf(b)), AST::Null));
                }

                _ => {
                    return Err("pow requires a number".to_string());
                }
            }
        }

        _ => {
            return Err("pow requires a number".to_string());
        }
    }
}

pub fn floor(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => {
            return Ok((AST::Number(a), AST::Null));
        }

        Ok(AST::Float(a)) => {
            return Ok((AST::Number(a.floor() as i64), AST::Null));
        }

        _ => {
            return Err("floor requires a number".to_string());
        }
    }
}

pub fn ceil(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => {
            return Ok((AST::Number(a), AST::Null));
        }

        Ok(AST::Float(a)) => {
            return Ok((AST::Number(a.ceil() as i64), AST::Null));
        }

        _ => {
            return Err("ceil requires a number".to_string());
        }
    }
}

pub fn random(_: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    return Ok((AST::Float(rand::random()), AST::Null));
}

pub fn random_int(_: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    return Ok((AST::Number(rand::random()), AST::Null));
}

pub fn cbrt(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => {
            if a < 0 {
                return Err("cbrt requires a positive number".to_string());
            }

            return Ok((AST::Float((a as f64).cbrt()), AST::Null));
        }

        Ok(AST::Float(a)) => {
            if a < 0.0 {
                return Err("cbrt requires a positive number".to_string());
            }

            return Ok((AST::Float(a.cbrt()), AST::Null));
        }

        _ => {
            return Err("cbrt requires a number".to_string());
        }
    }
}

pub fn acos(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => {
            if a < -1 || a > 1 {
                return Err("acos requires a number in the interval [-1, 1]".to_string());
            }

            return Ok((AST::Float((a as f64).acos()), AST::Null));
        },

        Ok(AST::Float(a)) => {
            if a < -1.0 || a > 1.0 {
                return Err("acos requires a number in the interval [-1, 1]".to_string());
            }

            return Ok((AST::Float(a.acos()), AST::Null));
        },

        _ => Err("acos requires a number".to_string())
    }
}

pub fn acosh(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => {
            if a < 1 {
                return Err("acosh requires a number greater than or equal to 1".to_string());
            }

            return Ok((AST::Float((a as f64).acosh()), AST::Null));
        },

        Ok(AST::Float(a)) => {
            if a < 1.0 {
                return Err("acosh requires a number greater than or equal to 1".to_string());
            }

            return Ok((AST::Float(a.acosh()), AST::Null));
        },

        _ => Err("acosh requires a number".to_string())
    }
}

pub fn asin(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => {
            if a < -1 || a > 1 {
                return Err("asin requires a number in the interval [-1, 1]".to_string());
            }

            return Ok((AST::Float((a as f64).asin()), AST::Null));
        },

        Ok(AST::Float(a)) => {
            if a < -1.0 || a > 1.0 {
                return Err("asin requires a number in the interval [-1, 1]".to_string());
            }

            return Ok((AST::Float(a.asin()), AST::Null));
        },

        _ => Err("asin requires a number".to_string())
    }
}

pub fn asinh(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => Ok((AST::Float((a as f64).asinh()), AST::Null)),
        Ok(AST::Float(a)) => Ok((AST::Float(a.asinh()), AST::Null)),

        _ => Err("asinh requires a number".to_string())
    }
}

pub fn atan(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => Ok((AST::Float((a as f64).atan()), AST::Null)),
        Ok(AST::Float(a)) => Ok((AST::Float(a.atan()), AST::Null)),

        _ => Err("atan requires a number".to_string())
    }
}

pub fn atanh(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => {
            if a < -1 || a > 1 {
                return Err("atanh requires a number in the interval [-1, 1]".to_string());
            }

            return Ok((AST::Float((a as f64).atanh()), AST::Null));
        },

        Ok(AST::Float(a)) => {
            if a < -1.0 || a > 1.0 {
                return Err("atanh requires a number in the interval [-1, 1]".to_string());
            }

            return Ok((AST::Float(a.atanh()), AST::Null));
        },

        _ => Err("atanh requires a number".to_string())
    }
}

pub fn cos(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => {
            if a < -1 || a > 1 {
                return Err("cos requires a number in the interval [-1, 1]".to_string());
            }

            return Ok((AST::Float((a as f64).cos()), AST::Null));
        },

        Ok(AST::Float(a)) => {
            if a < -1.0 || a > 1.0 {
                return Err("cos requires a number in the interval [-1, 1]".to_string());
            }

            return Ok((AST::Float(a.cos()), AST::Null));
        },

        _ => Err("cos requires a number".to_string())
    }
}

pub fn cosh(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => Ok((AST::Float((a as f64).cosh()), AST::Null)),
        Ok(AST::Float(a)) => Ok((AST::Float(a.cosh()), AST::Null)),

        _ => Err("cosh requires a number".to_string())
    }
}

pub fn exp(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => Ok((AST::Float((a as f64).exp()), AST::Null)),
        Ok(AST::Float(a)) => Ok((AST::Float(a.exp()), AST::Null)),

        _ => Err("exp requires a number".to_string())
    }
}

pub fn exp2(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => Ok((AST::Float((a as f64).exp2()), AST::Null)),
        Ok(AST::Float(a)) => Ok((AST::Float(a.exp2()), AST::Null)),

        _ => Err("exp2 requires a number".to_string())
    }
}

pub fn expm1(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => Ok((AST::Float((a as f64).exp_m1()), AST::Null)),
        Ok(AST::Float(a)) => Ok((AST::Float(a.exp_m1()), AST::Null)),

        _ => Err("expm1 requires a number".to_string())
    }
}

pub fn fract(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => Ok((AST::Float((a as f64).fract()), AST::Null)),
        Ok(AST::Float(a)) => Ok((AST::Float(a.fract()), AST::Null)),

        _ => Err("fract requires a number".to_string())
    }
}

pub fn ln(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => {
            if a <= 0 {
                return Err("ln requires a number greater than 0".to_string());
            }

            return Ok((AST::Float((a as f64).ln()), AST::Null));
        },

        Ok(AST::Float(a)) => {
            if a <= 0.0 {
                return Err("ln requires a number greater than 0".to_string());
            }

            return Ok((AST::Float(a.ln()), AST::Null));
        },

        _ => Err("ln requires a number".to_string())
    }
}

pub fn ln1p(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => {
            if a <= -1 {
                return Err("ln1p requires a number greater than -1".to_string());
            }

            return Ok((AST::Float((a as f64).ln_1p()), AST::Null));
        }

        Ok(AST::Float(a)) => {
            if a <= -1.0 {
                return Err("ln1p requires a number greater than -1".to_string());
            }

            return Ok((AST::Float(a.ln_1p()), AST::Null));
        }

        _ => Err("ln1p requires a number".to_string())
    }
}

pub fn log10(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => {
            if a <= 0 {
                return Err("log10 requires a number greater than 0".to_string());
            }

            return Ok((AST::Float((a as f64).log10()), AST::Null));
        },

        Ok(AST::Float(a)) => {
            if a <= 0.0 {
                return Err("log10 requires a number greater than 0".to_string());
            }

            return Ok((AST::Float(a.log10()), AST::Null));
        },

        _ => Err("log10 requires a number".to_string())
    }
}

pub fn log2(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => {
            if a <= 0 {
                return Err("log2 requires a number greater than 0".to_string());
            }

            return Ok((AST::Float((a as f64).log2()), AST::Null));
        },

        Ok(AST::Float(a)) => {
            if a <= 0.0 {
                return Err("log2 requires a number greater than 0".to_string());
            }

            return Ok((AST::Float(a.log2()), AST::Null));
        },

        _ => Err("log2 requires a number".to_string())
    }
}

pub fn sin(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => Ok((AST::Float((a as f64).sin()), AST::Null)),
        Ok(AST::Float(a)) => Ok((AST::Float(a.sin()), AST::Null)),

        _ => Err("sin requires a number".to_string())
    }
}

pub fn sinh(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => Ok((AST::Float((a as f64).sinh()), AST::Null)),
        Ok(AST::Float(a)) => Ok((AST::Float(a.sinh()), AST::Null)),

        _ => Err("sinh requires a number".to_string())
    }
}

pub fn tan(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => Ok((AST::Float((a as f64).tan()), AST::Null)),
        Ok(AST::Float(a)) => {
            let b = a.tan();
            if b.is_nan() {
                return Err("tan requires a real number that is not an odd multiple of pi/2".to_string());
            }
            Ok((AST::Float(b), AST::Null))
        },

        _ => Err("tan requires a number".to_string())
    }
}

pub fn tanh(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => Ok((AST::Float((a as f64).tanh()), AST::Null)),
        Ok(AST::Float(a)) => Ok((AST::Float(a.tanh()), AST::Null)),

        _ => Err("tanh requires a number".to_string())
    }
}

pub fn trunc(args: Vec<AST>, _: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
    match eval(args[0].clone(), &mut HashMap::new()) {
        Ok(AST::Number(a)) => Ok((AST::Float((a as f64).trunc()), AST::Null)),
        Ok(AST::Float(a)) => Ok((AST::Float(a.trunc()), AST::Null)),

        _ => Err("trunc requires a number".to_string())
    }
}

pub fn get_object() -> HashMap<String, AST> {
    let mut objects = HashMap::new();

    objects.insert(
        "div".to_string(),
        AST::InternalFunction {
            name: "div".to_string(),
            args: vec!["a".to_string(), "b".to_string()],
            call_fn: div,
        }
    );

    objects.insert(
        "mul".to_string(),
        AST::InternalFunction {
            name: "mul".to_string(),
            args: vec!["a".to_string(), "b".to_string()],
            call_fn: mul,
        }
    );

    objects.insert(
        "abs".to_string(),
        AST::InternalFunction {
            name: "abs".to_string(),
            args: vec!["a".to_string()],
            call_fn: abs,
        }
    );

    objects.insert(
        "sqrt".to_string(),
        AST::InternalFunction {
            name: "sqrt".to_string(),
            args: vec!["a".to_string()],
            call_fn: sqrt,
        }
    );

    objects.insert(
        "pow".to_string(),
        AST::InternalFunction {
            name: "pow".to_string(),
            args: vec!["a".to_string(), "b".to_string()],
            call_fn: pow,
        }
    );

    objects.insert(
        "floor".to_string(),
        AST::InternalFunction {
            name: "floor".to_string(),
            args: vec!["a".to_string()],
            call_fn: floor,
        }
    );

    objects.insert(
        "ceil".to_string(),
        AST::InternalFunction {
            name: "ceil".to_string(),
            args: vec!["a".to_string()],
            call_fn: ceil,
        }
    );

    objects.insert(
        "random".to_string(),
        AST::InternalFunction {
            name: "random".to_string(),
            args: vec![],
            call_fn: random,
        }
    );

    objects.insert(
        "random_int".to_string(),
        AST::InternalFunction {
            name: "random_int".to_string(),
            args: vec![],
            call_fn: random_int,
        }
    );

    objects.insert(
        "cbrt".to_string(),
        AST::InternalFunction {
            name: "cbrt".to_string(),
            args: vec!["a".to_string()],
            call_fn: cbrt,
        }
    );

    objects.insert(
        "acos".to_string(),
        AST::InternalFunction {
            name: "acos".to_string(),
            args: vec!["a".to_string()],
            call_fn: acos,
        }
    );

    objects.insert(
        "acosh".to_string(),
        AST::InternalFunction {
            name: "acosh".to_string(),
            args: vec!["a".to_string()],
            call_fn: acosh,
        }
    );

    objects.insert(
        "asin".to_string(),
        AST::InternalFunction {
            name: "asin".to_string(),
            args: vec!["a".to_string()],
            call_fn: asin,
        }
    );

    objects.insert(
        "asinh".to_string(),
        AST::InternalFunction {
            name: "asinh".to_string(),
            args: vec!["a".to_string()],
            call_fn: asinh,
        }
    );

    objects.insert(
        "atan".to_string(),
        AST::InternalFunction {
            name: "atan".to_string(),
            args: vec!["a".to_string()],
            call_fn: atan,
        }
    );

    objects.insert(
        "atanh".to_string(),
        AST::InternalFunction {
            name: "atanh".to_string(),
            args: vec!["a".to_string()],
            call_fn: atanh,
        }
    );

    objects.insert(
        "cos".to_string(),
        AST::InternalFunction {
            name: "cos".to_string(),
            args: vec!["a".to_string()],
            call_fn: cos,
        }
    );

    objects.insert(
        "cosh".to_string(),
        AST::InternalFunction {
            name: "cosh".to_string(),
            args: vec!["a".to_string()],
            call_fn: cosh,
        }
    );

    objects.insert(
        "exp".to_string(),
        AST::InternalFunction {
            name: "exp".to_string(),
            args: vec!["a".to_string()],
            call_fn: exp,
        }
    );

    objects.insert(
        "exp2".to_string(),
        AST::InternalFunction {
            name: "exp2".to_string(),
            args: vec!["a".to_string()],
            call_fn: exp2,
        }
    );

    objects.insert(
        "expm1".to_string(),
        AST::InternalFunction {
            name: "expm1".to_string(),
            args: vec!["a".to_string()],
            call_fn: expm1,
        }
    );

    objects.insert(
        "fract".to_string(),
        AST::InternalFunction {
            name: "fract".to_string(),
            args: vec!["a".to_string()],
            call_fn: fract,
        }
    );

    objects.insert(
        "ln".to_string(),
        AST::InternalFunction {
            name: "ln".to_string(),
            args: vec!["a".to_string()],
            call_fn: ln,
        }
    );

    objects.insert(
        "ln1p".to_string(),
        AST::InternalFunction {
            name: "ln1p".to_string(),
            args: vec!["a".to_string()],
            call_fn: ln1p,
        }
    );

    objects.insert(
        "log10".to_string(),
        AST::InternalFunction {
            name: "log10".to_string(),
            args: vec!["a".to_string()],
            call_fn: log10,
        }
    );

    objects.insert(
        "log2".to_string(),
        AST::InternalFunction {
            name: "log2".to_string(),
            args: vec!["a".to_string()],
            call_fn: log2,
        }
    );

    objects.insert(
        "sin".to_string(),
        AST::InternalFunction {
            name: "sin".to_string(),
            args: vec!["a".to_string()],
            call_fn: sin,
        }
    );

    objects.insert(
        "sinh".to_string(),
        AST::InternalFunction {
            name: "sinh".to_string(),
            args: vec!["a".to_string()],
            call_fn: sinh,
        }
    );

    objects.insert(
        "tan".to_string(),
        AST::InternalFunction {
            name: "tan".to_string(),
            args: vec!["a".to_string()],
            call_fn: tan,
        }
    );

    objects.insert(
        "tanh".to_string(),
        AST::InternalFunction {
            name: "tanh".to_string(),
            args: vec!["a".to_string()],
            call_fn: tanh,
        }
    );

    objects.insert(
        "trunc".to_string(),
        AST::InternalFunction {
            name: "trunc".to_string(),
            args: vec!["a".to_string()],
            call_fn: trunc,
        }
    );

    objects.insert(
        "CBRT_2".to_string(),
        AST::Float(1.25992104989487316476721060727822835_f64)
    );

    objects.insert(
        "CBRT_3".to_string(),
        AST::Float(1.44224957030740838232163831078010958_f64)
    );

    objects.insert(
        "E".to_string(),
        AST::Float(2.71828182845904523536028747135266250_f64)
    );

    objects.insert(
        "EGAMMA".to_string(),
        AST::Float(0.57721566490153286060651209008240243_f64)
    );

    objects.insert(
        "FRAC_1_PI".to_string(),
        AST::Float(0.31830988618379067153776752674502872_f64)
    );

    objects.insert(
        "FRAC_1_SQRT_2".to_string(),
        AST::Float(0.70710678118654752440084436210484903_f64)
    );

    objects.insert(
        "FRAC_1_SQRT_2PI".to_string(),
        AST::Float(0.39894228040143267793994605993438186_f64)
    );

    objects.insert(
        "FRAC_1_SQRT_3".to_string(),
        AST::Float(0.57735026918962576450914878050195745_f64)
    );

    objects.insert(
        "FRAC_1_SQRT_PI".to_string(),
        AST::Float(0.56418958354775628694807945156077258_f64)
    );

    objects.insert(
        "FRAC_2_PI".to_string(),
        AST::Float(0.63661977236758134307553505349005744_f64)
    );

    objects.insert(
        "FRAC_2_SQRT_PI".to_string(),
        AST::Float(1.12837916709551257389615890312154517_f64)
    );

    objects.insert(
        "FRAC_PI_2".to_string(),
        AST::Float(1.57079632679489661923132169163975144_f64)
    );

    objects.insert(
        "FRAC_PI_3".to_string(),
        AST::Float(1.04719755119659774615421446109316763_f64)
    );

    objects.insert(
        "FRAC_PI_4".to_string(),
        AST::Float(0.78539816339744830961566084581987572_f64)
    );

    objects.insert(
        "FRAC_PI_5".to_string(),
        AST::Float(0.62831853071795864769252867665590057_f64)
    );

    objects.insert(
        "FRAC_PI_6".to_string(),
        AST::Float(0.52359877559829887307710723054658381_f64)
    );

    objects.insert(
        "FRAC_PI_7".to_string(),
        AST::Float(0.44879895051282760549466334046850041_f64)
    );

    objects.insert(
        "FRAC_PI_8".to_string(),
        AST::Float(0.39269908169872415480783042290993786_f64)
    );

    objects.insert(
        "LN_2".to_string(),
        AST::Float(0.69314718055994530941723212145817656_f64)
    );

    objects.insert(
        "LN_10".to_string(),
        AST::Float(2.30258509299404568401799145468436421_f64)
    );

    objects.insert(
        "LOG2_10".to_string(),
        AST::Float(3.32192809488736234787031942948939018_f64)
    );

    objects.insert(
        "LOG2_E".to_string(),
        AST::Float(1.44269504088896340735992468100189214_f64)
    );

    objects.insert(
        "LOG10_2".to_string(),
        AST::Float(0.30102999566398119521373889472449302_f64)
    );

    objects.insert(
        "LOG10_E".to_string(),
        AST::Float(0.43429448190325182765112891891660508_f64)
    );

    objects.insert(
        "PHI".to_string(),
        AST::Float(1.61803398874989484820458683436563811_f64)
    );

    objects.insert(
        "PI".to_string(),
        AST::Float(3.14159265358979323846264338327950288_f64)
    );

    objects.insert(
        "RT12_2".to_string(),
        AST::Float(1.05946309435929526456182529494634170_f64)
    );

    objects.insert(
        "SILVER".to_string(),
        AST::Float(2.41421356237309504880168872420969808_f64)
    );

    objects.insert(
        "SUPERPHI".to_string(),
        AST::Float(1.46557123187676802665673122521993910_f64)
    );

    objects.insert(
        "SQRT_2".to_string(),
        AST::Float(1.41421356237309504880168872420969808_f64)
    );

    objects.insert(
        "SQRT_3".to_string(),
        AST::Float(1.73205080756887729352744634150587236_f64)
    );

    objects.insert(
        "SQRT_5".to_string(),
        AST::Float(2.23606797749978969640917366873127623_f64)
    );

    objects.insert(
        "TAU".to_string(),
        AST::Float(6.28318530717958647692528676655900577_f64)
    );

    objects
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_object_test() {
        let object = get_object();

        assert_eq!(object.len(), 64);
        assert_eq!(object.contains_key("div"), true);
    }

    #[test]
    fn div_test() {
        let mut context = HashMap::new();
        let args = vec![AST::Number(10), AST::Number(2)];
        let result = div(args, &mut context).unwrap().0;

        match result {
            AST::Number(a) => {
                assert_eq!(a, 5);
            }
            _ => panic!("Expected AST::Number")
        }
    }

    #[test]
    fn abs_test() {
        let mut context = HashMap::new();
        let args = vec![AST::Number(-10)];

        match abs(args, &mut context).unwrap().0 {
            AST::Number(a) => {
                assert_eq!(a, 10);
            }
            _ => panic!("Expected AST::Number")
        }
    }

    #[test]
    fn sqrt_test() {
        let mut context = HashMap::new();
        let args = vec![AST::Number(9)];

        match sqrt(args, &mut context).unwrap().0 {
            AST::Float(a) => {
                assert_eq!(a, 3.0);
            }

            _ => panic!("Expected AST::Float")
        }
    }

    #[test]
    fn pow_test() {
        let mut context = HashMap::new();
        let args = vec![AST::Number(2), AST::Number(3)];

        match pow(args, &mut context).unwrap().0 {
            AST::Number(a) => {
                assert_eq!(a, 8);
            }

            _ => panic!("Expected AST::Number")
        }
    }

    #[test]
    fn floor_test() {
        let mut context = HashMap::new();
        let args = vec![AST::Float(3.14)];

        match floor(args, &mut context).unwrap().0 {
            AST::Number(a) => {
                assert_eq!(a, 3);
            }

            _ => panic!("Expected AST::Number")
        }
    }

    #[test]
    fn ceil_test() {
        let mut context = HashMap::new();
        let args = vec![AST::Float(3.14)];

        match ceil(args, &mut context).unwrap().0 {
            AST::Number(a) => {
                assert_eq!(a, 4);
            }

            _ => panic!("Expected AST::Number")
        }
    }

    #[test]
    fn pi_test() {
        let object = get_object();

        match object.get("PI").unwrap() {
            AST::Float(a) => {
                assert_eq!(a, &std::f64::consts::PI);
            }

            _ => panic!("Expected AST::Float")
        }
    }

    #[test]
    fn div_by_zero() {
        let mut context = HashMap::new();
        let args = vec![AST::Number(10), AST::Number(0)];

        match div(args, &mut context) {
            Err(e) => {
                assert_eq!(e, "cannot divide by zero");
            }

            _ => panic!("Expected Err")
        }
    }
}
