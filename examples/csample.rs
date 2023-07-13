//! This example shows dynamic shell completion features

fn complete_calculator(input: &String) -> Vec<(&'static str, Option<&'static str>)> {
    let items = ["alpha", "beta", "banana", "cat", "durian"];
    items
        .iter()
        .filter(|item| item.starts_with(input))
        .map(|item| (*item, None))
        .collect::<Vec<_>>()
}

fn main() {
    use bpaf::*;

    let a = short('a').long("avocado").help("Use avocado").switch();
    let b = short('b').long("banana").help("Use banana").switch();
    let bb = long("bananananana").help("I'm Batman").switch();
    let c = long("calculator")
        .help("calculator expression")
        .argument::<String>("EXPR")
        .complete(complete_calculator);
    let parser = construct!(a, b, bb, c)
        .to_options()
        .descr("Dynamic autocomplete example")
        .footer(
            "\
    bpaf supports dynamic autocompletion for a few shells, make sure your binary is in $PATH
     and try using one of those this output should go into a file that depends on your shell:
    $ csample --bpaf-complete-style-bash
    $ csample --bpaf-complete-style-zsh
    $ csample --bpaf-complete-style-fish
    $ csample --bpaf-complete-style-elvish",
        );

    println!("{:?}", parser.fallback_to_usage().run());
}
