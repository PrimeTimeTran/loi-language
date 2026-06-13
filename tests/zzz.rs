mod harness;

use inkwell::context::Context;

use crate::harness::helpers::compile_and_lower;

#[test]
fn p01_main_fn_initialized_success() {
    let context = Context::create();

    let llvm = compile_and_lower(&context, "Hello Loi").expect("compile failed");

    println!("{}", llvm.ir());

    llvm.verify().expect("LLVM verify failed");
}
#[test]
fn p02_identifies_recognized() {
    let context = Context::create();

    let llvm = compile_and_lower(&context, "true").expect("compile failed");

    println!("{}", llvm.ir());

    llvm.verify().expect("LLVM verify failed");
}
#[test]
fn p03_str_number() {
    let context = Context::create();

    let llvm = compile_and_lower(&context, "5").expect("compile failed");

    println!("{}", llvm.ir());

    llvm.verify().expect("LLVM verify failed");
}

#[test]
fn p04_print() {
    let context = Context::create();

    let llvm = compile_and_lower(&context, "print(5)").expect("compile failed");

    println!("{}", llvm.ir());

    llvm.verify().expect("LLVM verify failed");
}
