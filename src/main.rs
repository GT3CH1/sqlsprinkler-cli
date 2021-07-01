// Copyright 2021 Gavin Pease
use std::env;
fn main() {
    println!("Eat short");
    let args: Vec<String> = env::args().collect();
    let my_arg = &args[1];
    println!("{}",myArg);
}
