//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short, long)]
    /// Opt in for premium serivces
    pub premium: bool,
    #[bpaf(external(cmd), many)]
    pub commands: Vec<Cmd>,
}

#[derive(Debug, Clone, Bpaf)]
pub enum Cmd {
    #[bpaf(command, adjacent)]
    /// Performs eating action
    Eat(#[bpaf(positional("FOOD"))] String),
    #[bpaf(command, adjacent)]
    /// Performs drinking action
    Drink {
        /// Are you going to drink coffee?
        coffee: bool,
    },
    #[bpaf(command, adjacent)]
    /// Performs taking a nap action
    Sleep {
        #[bpaf(argument("HOURS"))]
        time: usize,
    },
}
