//! This example constructs dynamic command tree from a structure

use bpaf::*;

#[derive(Debug, Clone)]
enum Cog {
    Command {
        help: &'static str,
        name: &'static str,
        operation: &'static str,
    },
    Group {
        name: &'static str,
        help: &'static str,
        nested: Vec<Cog>,
    },
}

fn config() -> Cog {
    let echo1 = Cog::Command {
        help: "First echo command",
        name: "echoCommand",
        operation: "echo 'some text'",
    };

    let echo2 = Cog::Command {
        help: "Second echo command",
        name: "anotherEchoCmd",
        operation: "echo 'another text'",
    };
    let sleep = Cog::Command {
        name: "sleepCommand",
        help: "sleep for a bit",
        operation: "sleep 5",
    };
    let group1 = Cog::Group {
        name: "commandGroup",
        help: "contains a single sleep",
        nested: vec![sleep],
    };
    Cog::Group {
        name: "commands",
        help: "contains all the commands, can be flattened with choose but effort :)",
        nested: vec![echo1, echo2, group1],
    }
}

fn choose<T>(xs: Vec<Box<dyn Parser<T> + 'static>>) -> Box<dyn Parser<T>>
where
    T: 'static,
{
    let mut items = xs.into_iter();

    let mut res = items.next().unwrap();
    for next in items {
        res = construct!([res, next]).boxed()
    }
    res
}

fn make_parser(item: &Cog) -> Box<dyn Parser<&'static str>> {
    match item {
        Cog::Command {
            help,
            name,
            operation,
        } => Box::new(pure(*operation).to_options().descr(*help).command(name)),
        Cog::Group { name, help, nested } => {
            let nested = nested.iter().map(make_parser).collect::<Vec<_>>();
            let inner = choose(nested);
            inner.to_options().descr(*help).command(name).boxed()
        }
    }
}

fn main() {
    let cfg = config();
    let command = make_parser(&cfg).to_options().run();
    println!("{:?}", command);
}
