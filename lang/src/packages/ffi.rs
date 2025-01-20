use std::collections::HashMap;
use std::result;
use crate::ast::AST;
use crate::eval::eval;

pub fn call(mut args: Vec<AST>, context: &mut HashMap<String, AST>) -> Result<AST, String> {
    // (path_to_lib, function_name, arg1, arg2, ...)

    if args.len() < 2 {
        return Err("ffi.call requires at least 2 arguments".to_string());
    }

    let path = match eval(args[0].clone(), context) {
        Ok(AST::String(v)) => v,

        _ => return Err("ffi.call first argument must be a string".to_string()),
    };

    let name = match eval(args[1].clone(), context) {
        Ok(AST::String(v)) => v,

        _ => return Err("ffi.call second argument must be a string".to_string()),
    };

    unsafe {
        let lib = match libloading::Library::new(path) {
            Ok(lib) => lib,
            Err(e) => return Err(format!("Failed to load library: {}", e)),
        };

        let func: libloading::Symbol<unsafe extern "C" fn(
            argc: std::ffi::c_int,
            argv: *const std::ffi::c_int
        ) -> *mut std::ffi::c_void> 
            = match lib.get(name.as_bytes()) {
                Ok(func) => func,
                Err(e) => return Err(format!("Failed to load function: {}", e)),
            };

        let mut args_ptr: Vec<std::ffi::c_int> = Vec::new();

        args.remove(0);
        args.remove(0);

        for arg in args {
            match arg {
                AST::Number(v) => {
                    args_ptr.push(v as std::ffi::c_int);
                }

                _ => return Err("ffi.call arguments must be numbers for now".to_string()),
            };
        }

        let result_ptr = func(
            args_ptr.len() as std::ffi::c_int,
            args_ptr.as_ptr()
        );

        if result_ptr.is_null() {
            return Ok(AST::Null);
        };

        // like result_ptr as i64, but it can also be str
        if (result_ptr as i64) <= i32::MAX as i64 && (result_ptr as i64) >= i32::MIN as i64 {
            return Ok(AST::Number(result_ptr as i64));
        } else {
            let str = std::ffi::CStr::from_ptr(result_ptr as *const _);
            return Ok(AST::String(str.to_string_lossy().into_owned()))
        }
    }
}

pub fn get_object() -> HashMap<String, AST> {
	let mut object = HashMap::new();

	object.insert(
        "call".to_string(),
        AST::InternalFunction {
            name: "call".to_string(),
            args: vec!["__args__".to_string()],
            call_fn: call,
        }
    );

	object
}