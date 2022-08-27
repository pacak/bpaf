fn complete_calculator(input: Option<&String>) -> Vec<(&'static str, Option<&'static str>)> {
    let items = ["alpha", "beta", "banana", "cat", "durian"];
    items
        .iter()
        .filter(|item| input.map_or(true, |input| item.starts_with(input)))
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
        .comp(complete_calculator);
    let parser = construct!(a, b, bb, c).to_options();

    println!("{:?}", parser.run());
}
