//! Parsing negative numbers from the command line. `bpaf` won't try to do anything special here,
//! user will have to escape minus sign either with `=` or --
use bpaf::*;

fn main() {
    let age = long("age").argument::<i64>("AGE");
    let msg = "\
To pass a value that starts with a dash requres one one of two special syntaxes:

This will pass '-1' to '--age' handler and leave remaining arguments as is
    --age=-1
This will transform everything after '--' into non flags, '--age' will handle '-1'
and positional handlers will be able to handle the rest.
    --age -- -1";
    let num = age.to_options().descr(msg).run();
    println!("age: {}", num);
}
