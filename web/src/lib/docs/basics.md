# Basics

Note: Semicolons are not required!

## Variables

Variables can be defined, and redefined with 'let'. \
Like this:

```rust
let a = 1;
let b = "a";
let c = false;
```

You can also define variables with other variables like

```rust
let a = 1;
let b = a;
```

In addition, you can define variables with math, see [Math](math).

## Functions

Functions are defined with the 'fn' keyword, then with arguments inside of parentheses. \
There is currently no support for default values, and modu will return an error if you provide the wrong number of arguments.

```rust
fn yap(msg) {
    print(msg);
}

yap("Hello, World!");

// Outputs
//
// Hello, World!
```

Functions defined in a file, can be also be accessed in other files when imported, see [Imports](imports).

## Conditions

Modu currently only supports == and != operators. \
If the condition given returns true, the code inside of the brackets is ran.

```rust
if 1 == 1 {
    print("yes");
}

if 1 != 2 {
    print("duh");
}

// Outputs
//
// yes
// duh
```

You can also use conditions to a check if a value is not null in a simpler, more clean way:
```rust
if a {
    print("a exists");
}
```