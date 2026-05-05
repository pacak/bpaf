#![allow(dead_code)]

use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
enum LetterSum {
    /// Alpha: α
    #[bpaf(long, short)]
    Alpha,
    /// Beta: β
    #[bpaf(long, short)]
    Beta,
    /// Gamma: γ
    #[bpaf(long, short)]
    Gamma,
    /// Zetta: ζ
    #[bpaf(short)]
    Zetta,
}

#[derive(Debug, Clone, Bpaf)]
struct LetterProd {
    /// Alpha: α
    #[bpaf(long, short)]
    alpha: bool,
    /// Beta: β
    #[bpaf(long, short)]
    beta: bool,
    /// Gamma: γ
    #[bpaf(long, short)]
    gamma: bool,
    /// Zetta: ζ
    #[bpaf(short)]
    zetta: bool,
}

#[derive(Debug, Clone, Bpaf)]
enum Command {
    /// Alpha: α
    #[bpaf(command, short('a'))]
    Alpha,
    /// Beta: β
    #[bpaf(command, short('b'))]
    Beta,

    /// Bak Kut Teh: 肉骨茶
    #[bpaf(command)]
    BakKutTeh,

    /// Gamma: γ
    #[bpaf(command, short('g'))]
    Gamma,
}

fn comp_names(prefix: &str) -> Vec<(String, Option<String>)> {
    let mut names = Vec::new();
    let mut push = |name: &str, help: &str| {
        if name.starts_with(prefix) {
            names.push((name.to_owned(), Some(help.to_owned())));
        }
    };
    push("Alice", "Sends a message");
    push("Bob", "Receives a message");
    push("Carol", "Unrelated third party");
    push("Carlos", "A different unrelated third party");
    push("Grace", "Government representative");
    names
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
#[allow(dead_code)]
enum Opts {
    /// A set of conflicting short and long flags
    #[bpaf(command)]
    FlagSum {
        #[bpaf(external)]
        letter_sum: LetterSum,
    },

    /// A set of non-conflicting short and long flags
    #[bpaf(command)]
    FlagProd {
        #[bpaf(external)]
        letter_prod: LetterProd,
    },

    /// A set of conflicting commands
    #[bpaf(command)]
    Commands {
        #[bpaf(external)]
        command: Command,
    },

    /// Positional value completion
    #[bpaf(command)]
    PosComp {
        #[bpaf(positional::<String>("VAL"), complete(comp_names))]
        value: String,
    },

    /// Argument value completion
    #[bpaf(command)]
    ArgComp {
        #[bpaf(argument::<String>("VAL"), complete(comp_names))]
        value: String,
    },
}

fn main() {
    let opts = opts().fallback_to_usage().run();
    println!("{opts:?}");
}
