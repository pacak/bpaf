use std::process::ExitCode;

use bpaf_cauwugo::opts::{cargo_command, options, Options, CARGO_PARAMS};

fn main() {
    let opts = options().run();

    match opts {
        Options::Run {
            cargo_params,
            args,
            exec,
        } => {
            let mut cmd = cargo_command("run", &cargo_params, exec.as_ref());
            let mut child = cmd.arg("--").args(&args).spawn().unwrap();
            match child.wait().unwrap().code() {
                Some(code) => std::process::exit(code),
                None => {}
            }
        }
    }

    //    println!("{:?}", *EXECS);
}
