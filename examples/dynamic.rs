//! You can construct parser at runtime without having a concrete type too

use bpaf::*;

#[derive(Debug, Clone)]
enum Value {
    Bool(bool),
    Number(usize),
    String(String),
}

fn number(name: &'static str) -> impl Parser<(String, Value)> {
    let label = name.to_string();
    long(name)
        .argument::<usize>("NUM")
        .map(move |n| (label.clone(), Value::Number(n)))
}

fn bool(name: &'static str) -> impl Parser<(String, Value)> {
    let label = name.to_string();
    long(name)
        .switch()
        .map(move |n| (label.clone(), Value::Bool(n)))
}

fn string(name: &'static str) -> impl Parser<(String, Value)> {
    let label = name.to_string();
    long(name)
        .help("this can use a help message")
        .argument::<String>("NUM")
        .map(move |n| (label.clone(), Value::String(n)))
}

fn cons<T>(acc: Box<dyn Parser<Vec<T>>>, cur: Box<dyn Parser<T>>) -> Box<dyn Parser<Vec<T>>>
where
    T: 'static,
{
    construct!(acc, cur)
        .map(|(mut acc, cur)| {
            acc.push(cur);
            acc
        })
        .boxed()
}

enum Ty {
    Bool,
    Number,
    String,
}

fn main() {
    let items = &[
        ("banana", Ty::Bool),
        ("width", Ty::Number),
        ("name", Ty::String),
    ];

    let mut parser = pure(Vec::<(String, Value)>::new()).boxed();
    for (name, ty) in items {
        parser = cons(
            parser,
            match ty {
                Ty::Bool => bool(name).boxed(),
                Ty::Number => number(name).boxed(),
                Ty::String => string(name).boxed(),
            },
        )
    }

    let options = parser.run();
    println!("{:?}", options);
}
