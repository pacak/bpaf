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
        .argument("EXPR")
        .complete(complete_calculator);
    let parser = construct!(a, b, bb, c)
        .to_options()
        .descr("Dynamic autocomplete example")
        .footer(
            "\
    Currently bpaf supports bash and zsh
    To use it in bash have this binary compiled and in PATH and run

    $ source <(csample --bpaf-complete-style-bash)

    To use it in zsh you need to place output of this command in ~/.zsh/_csample
    $ csample --bpaf-complete-style-zsh
    ",
        );

    println!("{:?}", parser.run());
}
