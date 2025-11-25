use bpaf_crux::*;

fn main() {
    let a = short('a').req_flag('a');

    let parser = a.to_options();
    let mut args = std::env::args_os().collect::<Vec<_>>();
    args.remove(0);
    let r = parser.run_inner(args.as_slice());
    todo!("{r:?}");
}
