//! You can do potentially confusing things if you really-really have to :)

// I want something like this ./binary --arg1 "stuff" subcommand --arg2 "stuff" I want arg1
// to be at "top level" because it will be used by many sub commands and be mandatory almost always
// unless arg2 is passed to the subcommand.

// In practice it would be ./binary --token "token" create --org "org" and ./binary create
// --example where the last case wouldn't need the token because we aren't doing any api calls.

use bpaf::*;

/// this datatype is intended for program consumption, usize field in complex commands
/// is a shared top level argument
#[derive(Debug, Clone)]
enum Command {
    Simple,
    Complex1(String, i32),
    Complex2(String, i16),
}

/// this datatype is just an intermediate representation,
/// it exists only to ensure that the final type (Command) can be used without unwraps
#[derive(Debug, Clone)]
enum PreCommand {
    Simple,
    Complex1(i32),
    Complex2(i16),
}

fn main() {
    let token = long("token")
        .help("Token used for complex commands")
        .argument::<String>("TOKEN")
        .optional();

    // start with defining 3 commands: simple, complex1 and complex2
    let simple_parser = pure(PreCommand::Simple).to_options();
    let simple = simple_parser.command("simple");

    let complex1_parser = positional::<i32>("ARG");
    let complex1 = construct!(PreCommand::Complex1(complex1_parser))
        .to_options()
        .descr("This is complex command 1")
        .command("complex1");

    let complex2_parser = positional::<i16>("ARG");

    let complex2 = construct!(PreCommand::Complex2(complex2_parser))
        .to_options()
        .descr("This is complex command 2")
        .command("complex2");

    // compose then to accept any of those
    let preparser = construct!([simple, complex1, complex2]);

    // make a parser that accepts optional token and one of incomplete commands
    // then create complete command or fail
    let parser = construct!(token, preparser).parse(|(token, cmd)| match cmd {
        PreCommand::Simple => Ok(Command::Simple),
        PreCommand::Complex1(a) => match token {
            Some(token) => Ok(Command::Complex1(token, a)),
            None => Err("You must specify token to use with --token"),
        },
        PreCommand::Complex2(a) => match token {
            Some(token) => Ok(Command::Complex2(token, a)),
            None => Err("You must specify token to use with --token"),
        },
    });

    let cmd = parser.to_options().run();
    println!("{:?}", cmd);
}
