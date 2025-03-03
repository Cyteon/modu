mod math;
mod file;
mod time;
mod os;
mod ffi;
pub mod json;
pub mod string;

use crate::ast::AST;

pub fn get_package(name: &str) -> Option<AST> {
	match name {
		"math" => {
			Some(AST::Object {
				properties: math::get_object(),
				line: 0,
			})
		}

		"time" => {
			Some(AST::Object {
				properties: time::get_object(),
				line: 0,
			})
		}

		"file" => {
			let args = std::env::args().collect::<Vec<String>>();

			if args.len() > 1 && args[1] == "server" { // fallback incase the stop in eval.rs explodes for some reason
				return None;
			}

			Some(AST::Object {
				properties: file::get_object(),
				line: 0,
			})
		}

		"os" => Some(AST::Object {
			properties: os::get_object(),
			line: 0
		}),

		"ffi" => Some(AST::Object {
			properties: ffi::get_object(),
			line: 0
		}),

		"json" => Some(AST::Object {
			properties: json::get_object(),
			line: 0
		}),

		"string" => Some(AST::Object {
			properties: string::get_object(),
			line: 0
		}),

		_ => None
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_math_package() {
		let math = get_package("math").unwrap();
		match math {
			AST::Object { properties, line: _ } => {
				assert_eq!(properties.len(), 10);
				assert_eq!(properties.contains_key("div"), true);
			}
			_ => panic!("Expected AST::Object")
		}
	}

	#[test]
	fn get_file_package() {
		let file = get_package("file").unwrap();

		match file {
			AST::Object { properties, line: _ } => {
				assert_eq!(properties.len(), 3);
				assert_eq!(properties.contains_key("write_append"), true);
			}

			_ => panic!("Expected AST::Object")
		}
	}

	#[test]
	fn get_unknown_package() {
		let unknown = get_package("unknown");
		assert_eq!(unknown, None);
	}

	#[test]
	fn get_os_package() {
		let os = get_package("os").unwrap();
		match os {
			AST::Object { properties, line: _ } => {
				assert_eq!(properties.len(), 2);
				assert_eq!(properties.contains_key("exec"), true);
				assert_eq!(properties.contains_key("name"), true);
			}
			_ => panic!("Expected AST::Object")
		}
	}

}