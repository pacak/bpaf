use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
struct Foo {
    //  #[bpaf(hide, pure(Default::default()), optional)]
    #[bpaf(hide, pure(None), optional)]
    files: Option<Vec<u32>>,
}

fn main() {
    todo!("{:?}", foo().run());
}
