use bpaf_cauwugo::opts::{cauwugo_opts, Cauwugo};
use std::process::Command;

fn main() -> std::io::Result<()> {
    if let Some(first) = std::env::args_os().nth(1) {
        let supported = first == "--help"
            || first == "add"
            || first == "build"
            || first == "check"
            || first == "clean"
            || first == "run"
            || first == "test"
            || first.to_str().map_or(false, |s| s.starts_with("--bpaf"));

        if !supported {
            let mut cmd = Command::new("cargo");
            cmd.args(std::env::args_os().skip(1));

            let mut child = cmd.spawn()?;
            if let Some(code) = child.wait()?.code() {
                std::process::exit(code)
            } else {
                return Ok(());
            }
        }
    }

    let opts = cauwugo_opts().run();
    let mut cmd = Command::new("cargo");
    match opts.cauwugo {
        Cauwugo::Add(add) => add.pass_to_cmd(&mut cmd),
        Cauwugo::Build(build) => build.pass_to_cmd(&mut cmd),
        Cauwugo::Check(check) => check.pass_to_cmd(&mut cmd),
        Cauwugo::Clean(clean) => clean.pass_to_cmd(&mut cmd),
        Cauwugo::Run(run) => run.pass_to_cmd(&mut cmd),
        Cauwugo::Test(test) => test.pass_to_cmd(&mut cmd),
        // bench       Run the benchmarks ?
    }

    if opts.bpaf_verbose {
        println!("{:?}", cmd);
    }

    if opts.bpaf_dry {
        return Ok(());
    }

    let mut child = cmd.spawn()?;
    if let Some(code) = child.wait()?.code() {
        std::process::exit(code)
    }

    Ok(())
}
