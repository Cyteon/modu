use std::collections::HashMap;
use std::process::Command;
use crate::ast::AST;
use crate::eval::eval;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

fn clean_command(cmd: String) -> String {
	let clean = cmd.trim()
		.trim_matches('"')
		.trim_matches('\'')
		.to_string();

	return clean;
}

pub fn exec(args: Vec<AST>, context: &mut HashMap<String, AST>) -> Result<(AST, AST), String> {
	if args.len() != 1 {
		return Err("os.exec requires exactly one argument".to_string());
	}

	let command = match eval(args[0].clone(), context) {
		Ok(AST::String(value))=> value,

		Ok(_) => return Err("os.exec argument must be a string".to_string()),

		Err(e) => return Err(e),
	};

	let cleaned = clean_command(command);

	let output = {
		#[cfg(windows)] {
			Command::new("C:\\Windows\\System32\\cmd.exe")
				.arg("/C")
				.arg(&cleaned)
				.creation_flags(0x08000000)
				.output()
		}

		#[cfg(not(windows))] {
			Command::new("sh")
				.args(["-c", &cleaned])
				.output()
		}
	};

	match output {
		Ok(output) => {
			let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
			if output.status.success() {
				Ok((AST::String(stdout), AST::Null))
			} else {
				let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
				Err(stderr)
			}
		},
		Err(e) => Err(format!("Command execution failed: {}", e))
	}
}

pub fn get_object() -> HashMap<String, AST> {
	let mut object = HashMap::new();

	object.insert(
		"exec".to_string(),
		AST::InternalFunction {
			name: "exec".to_string(),
			args: vec!["command".to_string()],
			call_fn: exec,
		}
	);

	let os_name = match std::env::consts::OS {
		"linux" => "linux",
		"macos" => "macos",
		"windows" => "windows",
		_ => "unknown"
	};

	object.insert(
		"name".to_string(),
		AST::String(os_name.to_string())
	);

	object
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_exec_echo() {
		let args = vec![AST::String("echo hello".to_string())];
		let result = exec(args, &mut HashMap::new()).unwrap();
		match result.0 {
			AST::String(value) => {
				assert!(value.contains("hello"));
			},
			_ => panic!("Expected string output")
		}
	}
}