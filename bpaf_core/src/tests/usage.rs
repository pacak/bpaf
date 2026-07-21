use crate::{
    Parser, Visited, console_writer::Styled, construct, long, positional, pure, short,
    visitors::usage::Usage,
};

fn usage(visited: &impl Visited) -> String {
    let mut u = Usage::default();
    visited.vi(&mut u);
    let mut out = String::new();
    u.render_to(&mut out);

    let mut s = Styled {
        raw: out,
        tab: usize::MAX,
    }
    .mono();

    s.pop();
    s
}

#[test]
fn usage_product() {
    let a = short('a').switch();
    let b = short('b').req_flag(());
    let c = long("cat").argument::<usize>("KET").many();
    let d = positional::<usize>("LEN").optional();
    let parser = construct!(a, b, c, d).to_options();

    let r = usage(&parser);
    assert_eq!(r, "[-a] -b [--cat=KET]... [LEN]");
}

#[test]
fn many_and_arg() {
    let parser = short('M')
        .argument::<u32>("ARG")
        .help("with help")
        .many()
        .to_options();
    let r = usage(&parser);
    assert_eq!(r, "[-M=ARG]...");
}

#[test]
fn many_and_pos() {
    let parser = positional::<u32>("Ket")
        .help("with help")
        .many()
        .to_options();
    let r = usage(&parser);
    assert_eq!(r, "[<Ket>]...");
}

#[test]
fn some_arg() {
    let parser = short('a').argument::<String>("A").some("ARG").to_options();
    assert_eq!(usage(&parser), "-a=A...");
}

#[test]
fn many_switch() {
    let parser = short('a').switch().many().to_options();
    assert_eq!(usage(&parser), "[-a]...");
}

#[test]
fn some_switch() {
    let parser = short('a').switch().some("ARG").to_options();
    assert_eq!(usage(&parser), "[-a]...");
}

#[test]
fn some_and_req() {
    let parser = ra().some("some").to_options();
    assert_eq!(usage(&parser), "-a...");
}

#[test]
fn a_or_b() {
    let a = short('a').long("aaa").argument::<String>("A");
    let b = short('b').long("bbb").argument::<String>("B");
    let parser = construct!([a, b]).to_options();
    assert_eq!(usage(&parser), "(-a=A | -b=B)");
}

#[test]
fn a_or_b_and_c() {
    let a = short('a').long("aaa").argument::<String>("A");
    let b = short('b').long("bbb").argument::<String>("B");
    let ab = construct!([a, b]);
    let c = positional::<String>("C");
    let parser = construct!(ab, c).to_options();
    assert_eq!(usage(&parser), "(-a=A | -b=B) C");
}

#[test]
fn a_or_b_opt() {
    let a = short('a').long("aaa").argument::<String>("A");
    let b = short('b').long("bbb").argument::<String>("B");
    let parser = construct!([a, b]).optional().to_options();
    assert_eq!(usage(&parser), "[-a=A | -b=B]");
}

#[test]
fn a_or_b_opt_and_c() {
    let a = short('a').long("aaa").argument::<String>("A");
    let b = short('b').long("bbb").argument::<String>("B");
    let ab = construct!([a, b]).optional();
    let c = positional::<String>("C");
    let parser = construct!(ab, c).to_options();
    assert_eq!(usage(&parser), "[-a=A | -b=B] C");
}

#[test]
fn usage_choice_req() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let parser = construct!([a, b]).to_options();
    let r = usage(&parser);
    assert_eq!(r, "(-a | -b)");
}

fn ra() -> impl Parser<Output = bool> {
    short('a').req_flag(true)
}

fn oa() -> impl Parser<Output = bool> {
    short('a').switch()
}

fn rb() -> impl Parser<Output = bool> {
    short('b').req_flag(true)
}

fn ob() -> impl Parser<Output = bool> {
    short('b').switch()
}

fn ca() -> impl Parser<Output = bool> {
    pure(true).to_options().command("a")
}

fn cb() -> impl Parser<Output = bool> {
    pure(true).to_options().command("b")
}

#[test]
fn optional_and_sum_1() {
    let parser = construct!([oa(), ob()]).to_options();
    assert_eq!(usage(&parser), "[-a | -b]");
}

#[test]
fn optional_and_sum_2() {
    let parser = construct!([ra(), ob()]).to_options();
    assert_eq!(usage(&parser), "[-a | -b]");
}

#[test]
fn optional_and_sum_3() {
    let parser = construct!([ra(), rb()]).to_options();
    assert_eq!(usage(&parser), "(-a | -b)");
}

#[test]
fn optional_and_sum_4() {
    let parser = construct!([ra(), ob()]).optional().to_options();
    assert_eq!(usage(&parser), "[-a | -b]");
}

#[test]
fn optional_and_sum_5() {
    let parser = construct!([ra(), rb()]).optional().to_options();
    assert_eq!(usage(&parser), "[-a | -b]");
}

#[test]
fn optional_and_prod_1() {
    let parser = construct!(oa(), ob()).to_options();
    assert_eq!(usage(&parser), "[-a] [-b]");
}

#[test]
fn optional_and_prod_2() {
    let parser = construct!(ra(), ob()).to_options();
    assert_eq!(usage(&parser), "-a [-b]");
}

#[test]
fn optional_and_prod_3() {
    let parser = construct!(ra(), rb()).to_options();
    assert_eq!(usage(&parser), "-a -b");
}

#[test]
fn optional_and_prod_4() {
    let parser = construct!(ra(), ob()).optional().to_options();
    assert_eq!(usage(&parser), "[-a [-b]]");
}

#[test]
fn optional_and_prod_5() {
    let parser = construct!(ra(), rb()).optional().to_options();
    assert_eq!(usage(&parser), "[-a -b]");
}

#[test]
fn flatten_prod_left() {
    let ab = construct!(ra(), rb());
    let parser = construct!(ab, ra()).to_options();
    assert_eq!(usage(&parser), "-a -b -a");
}

#[test]
fn flatten_prod_mid() {
    let ab = construct!(ra(), rb());
    let parser = construct!(ra(), ab, rb()).to_options();
    assert_eq!(usage(&parser), "-a -a -b -b");
}

#[test]
fn flatten_prod_right() {
    let ab = construct!(ra(), rb());
    let parser = construct!(ra(), ab).to_options();
    assert_eq!(usage(&parser), "-a -a -b");
}

#[test]
fn dedup_commands_sum_1() {
    let parser = construct!([ca(), cb()]).to_options();
    assert_eq!(usage(&parser), "COMMAND ...");
}

#[test]
fn dedup_commands_sum_2() {
    let parser = construct!([oa(), ca(), cb()]).to_options();
    assert_eq!(usage(&parser), "[-a | COMMAND ...]");
}

#[test]
fn dedup_commands_sum_3() {
    let parser = construct!([ca(), oa(), cb()]).to_options();
    assert_eq!(usage(&parser), "[COMMAND ... | -a]");
}

#[test]
fn dedup_commands_sum_4() {
    let parser = construct!([ca(), cb(), oa()]).to_options();
    assert_eq!(usage(&parser), "[COMMAND ... | -a]");
}

#[test]
fn dedup_commands_prod_1() {
    let parser = construct!(ca(), cb()).to_options();
    assert_eq!(usage(&parser), "COMMAND ... COMMAND ...");
}

#[test]
fn flatten_prod() {
    let a = short('a').switch();
    let b = short('b').req_flag(());
    let c = short('c').switch();
    let ab = construct!(a, b);
    let parser = construct!(ab, c).many().to_options();
    assert_eq!(usage(&parser), "[[-a] -b [-c]]...");
}

#[test]
fn flatten_sum_of_prods_1() {
    let a = construct!(ra(),);
    let b = construct!(rb(),);
    let parser = construct!([a, b]).to_options();
    assert_eq!(usage(&parser), "(-a | -b)");
}

#[test]
fn required_or_and() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c = short('c').req_flag(());
    let d = short('d').req_flag(());
    let ab = construct!(a, b);
    let cd = construct!(c, d);
    let parser = construct!([ab, cd]).to_options();
    assert_eq!(usage(&parser), "(-a -b | -c -d)");
}

#[test]
fn flatten_sum_of_prods_3() {
    let a = construct!(ra(), oa()).map(|_| true);
    let parser = construct!([a, rb()]).to_options();
    assert_eq!(usage(&parser), "(-a [-a] | -b)");
}

#[test]
fn flatten_prod_of_prods() {
    let a = construct!(ra(), oa());
    let b = construct!(rb(), ob());
    let parser = construct!(a, b).to_options();
    assert_eq!(usage(&parser), "-a [-a] -b [-b]");
}

#[test]
fn required_one_many() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let parser = construct!(a, b).many().to_options();
    assert_eq!(usage(&parser), "[-a -b]...");
}

#[test]
fn optional_one_many() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let parser = construct!(a, b).optional().many().to_options();
    assert_eq!(usage(&parser), "[-a -b]...");
}

#[test]
fn sensors_many() {
    let a = short('a').argument::<String>("A");
    let b = short('b').argument::<String>("B");
    let parser = construct!(a, b).many().to_options();
    assert_eq!(usage(&parser), "[-a=A -b=B]...");
}

#[test]
fn sensors_some() {
    let a = short('a').argument::<String>("A");
    let b = short('b').argument::<String>("B");
    let parser = construct!(a, b).some("want some sensors").to_options();
    assert_eq!(usage(&parser), "(-a=A -b=B)...");
}

#[test]
fn several_commands_squash_1() {
    let a = pure(()).to_options().command("cmd_a");
    let b = pure(()).to_options().command("cmd_b");
    let ab = construct!([a, b]).group_help("Explanation applicable for both A and B:");
    let c = pure(()).to_options().command("cmd_c");

    let parser = construct!([ab, c]);

    let u = usage(&parser);

    assert_eq!(u, "COMMAND ...");
}

#[test]
fn several_commands_squash_2() {
    let a = pure(()).to_options().command("cmd_a");
    let b = pure(()).to_options().command("cmd_b");
    let c = pure(()).to_options().command("cmd_c");
    let parser = construct!([a, b, c]).to_options();

    let u = usage(&parser);

    assert_eq!(u, "COMMAND ...");
}

#[test]
fn several_commands_squash_3() {
    let a = pure(()).to_options().command("cmd_a");
    let b = pure(()).to_options().command("cmd_b");
    let c = pure(()).to_options().command("cmd_c");
    let parser = construct!([a, b, c]).fallback(()).to_options();

    let u = usage(&parser);

    assert_eq!(u, "[COMMAND ...]");
}

#[test]
fn single_optional_req_select() {
    let a = short('a').req_flag(());
    let parser = construct!([a]).optional().to_options();

    assert_eq!(usage(&parser), "[-a]");
}

#[test]
fn fallback_req_select() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let parser = construct!([a, b]).fallback(()).to_options();

    assert_eq!(usage(&parser), "[-a | -b]");
}

#[test]
fn single_fallback_req_select() {
    let a = short('a').req_flag(());

    let parser = construct!([a]).fallback(()).to_options();

    assert_eq!(usage(&parser), "[-a]");
}

#[test]
fn optional_argument_select() {
    let a = short('a').argument::<String>("A");
    let b = short('b').argument::<String>("B");
    let parser = construct!([a, b]).optional().to_options();

    assert_eq!(usage(&parser), "[-a=A | -b=B]");
}

#[test]
fn required_or_many() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c = short('c').req_flag(());
    let d = short('d').req_flag(());
    let ab = construct!(a, b);
    let cd = construct!(c, d);
    let e = pure(((), ()));
    let f = pure(((), ()));
    let ef = construct!([e, f]);
    let parser = construct!([ab, cd, ef]).many().to_options();
    assert_eq!(usage(&parser), "[-a -b | -c -d]...");
}

#[test]
fn no_actual_arguments_also_works() {
    let parser = pure(true).to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "Usage: app\n\nAvailable options:\n    -h, --help  Prints help information\n"
    );
}

#[test]
fn hidden_fallback_branch() {
    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    struct Fallback {
        name: String,
    }

    let name = positional::<String>("COMMAND");
    let fallback = construct!(Fallback { name }).hide().map(Commands::Fallback);

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    enum Commands {
        Build {},
        Fallback(Fallback),
    }

    let build = pure(Commands::Build {}).to_options().command("build");
    let parser = construct!([fallback, build]).to_options();

    assert_eq!(usage(&parser), "COMMAND ...");
}

#[test]
fn positionals_in_branches_are_okay() {
    let a = short('a').argument::<String>("A");
    let b = short('b').argument::<String>("B");
    let c = positional::<String>("C");
    let d = positional::<String>("D");

    let ac = construct!(a, c);
    let bd = construct!(b, d);
    let parser = construct!([ac, bd]).to_options();
    assert_eq!(usage(&parser), "(-a=A C | -b=B D)");
}

#[test]
fn optional_last_of_many() {
    let a = short('a').req_flag(true);
    let b = short('b').req_flag(false);
    let ab = construct!([a, b]).last().fallback(false);
    let parser = ab.to_options();

    assert_eq!(usage(&parser), "[-a | -b]...");
}

#[test]
fn optional_last_of_req() {
    let a = short('a').req_flag(true);
    let b = short('b').req_flag(false);
    let ab = construct!([a, b]).last();
    let parser = ab.to_options();

    assert_eq!(usage(&parser), "(-a | -b)...");
}

#[test]
fn double_strict() {
    let a = short('a').switch();
    let b = positional::<usize>("B").strict();
    let c = positional::<usize>("C").strict();
    let parser = construct!(a, b, c).to_options();
    assert_eq!(usage(&parser), "[-a] -- B C");
}

#[test]
fn optional_strict() {
    let parser = positional::<usize>("A").strict().optional().to_options();
    assert_eq!(usage(&parser), "[-- A]");
}

#[test]
fn commands_and_adjacent() {
    let eat = pure(()).to_options().command("eat").lazy();

    let sleep = pure(()).to_options().command("sleep").lazy();

    let cmds = construct!([eat, sleep]);
    let switch = short('s').switch();

    let parser = construct!(switch, cmds).to_options();

    let expected = "[-s] COMMAND ...";
    assert_eq!(usage(&parser), expected);
}

#[test]
fn nest_usage() {
    let num = short('n')
        .long("num")
        .argument::<u32>("N")
        .help("Number to add");
    let add = long("add").short('a').nest(num);

    let doctor = long("doctor").help("Run diag").req_flag(0);
    let check = long("check").help("Perform the check").nest(pure(42));
    let parser = construct!([add, doctor, check]).to_options();

    let expected = "(-a {-n=N} | --doctor | --check)";
    assert_eq!(usage(&parser), expected);
}

#[test]
fn nest_usage_preserves_inner_sum() {
    // Top level of the nested parser is a product, but inner sums should stay
    let a = short('a').req_flag(0);
    let b = short('b').req_flag(1);
    let c = short('c').req_flag(2);
    let bc = construct!([b, c]);
    let inner = construct!(a, bc);
    let add = long("add").short('d').nest(inner);
    let parser = add.to_options();

    let expected = "-d {-a (-b | -c)}";
    assert_eq!(usage(&parser), expected);
}

#[test]
fn or_else_required_visible() {
    let a = short('a').switch();
    let b = short('b').req_flag(true);
    let parser = a.or_else(b).to_options();

    let expected = "[-a | -b]";
    assert_eq!(usage(&parser), expected);
}

#[test]
fn or_else_required_hidden_usage() {
    let a = short('a').switch();
    let b = short('b').req_flag(true).hide_usage();
    let parser = a.or_else(b).to_options();

    let expected = "[-a]";
    assert_eq!(usage(&parser), expected);
}

#[test]
fn or_else_required_hidden_help() {
    let a = short('a').switch();
    let b = short('b').req_flag(true).hide();
    let parser = a.or_else(b).to_options();

    let expected = "[-a]";
    assert_eq!(usage(&parser), expected);
}

#[test]
fn required_or_optional_with_hidden_usage() {
    let a = short('a').req_flag('a');
    let c = short('c').flag('c', 'C');
    let v = short('v').req_flag('v').hide_usage();
    let p = a.or_else(c).into_box().or_else(v);
    let parser = p.to_options();
    let expected = "[-a | -c]";
    assert_eq!(usage(&parser), expected);
}

#[test]
fn optional_sum_in_product() {
    let a = short('a').req_flag('a');
    let c = short('c').flag('c', 'C');
    let inner = a.or_else(c);
    let b = short('b').req_flag('b');
    let parser = construct!(inner, b).to_options();
    let expected = "[-a | -c] -b";
    assert_eq!(usage(&parser), expected);
}

#[test]
fn global_optional_sum_in_sum() {
    let a = short('a').req_flag('a');
    let c = short('c').flag('c', 'C');
    let g = a.or_else(c).global();
    let d = short('d').req_flag('d');
    let parser = construct!([g, d]).to_options();
    let expected = "[-a | -c | -d]";
    assert_eq!(usage(&parser), expected);
}



// #[test]
// fn many_strict() {
//     let a = short('a').switch();
//     let b = positional::<usize>("B").strict().many();
//     let parser = construct!(a, b).to_options();
//     assert_eq!(usage(&parser), "[-a] [-- B...]");
// }

// #[test]
// fn quadruple_strict() {
//     let a = short('a').switch();
//     let b = positional::<usize>("B").strict();
//     let c = positional::<usize>("C").strict();
//     let abc = construct!(a, b, c);
//
//     let d = short('d').switch();
//     let e = positional::<usize>("E").strict();
//     let f = positional::<usize>("F").strict();
//     let def = construct!(d, e, f);
//
//     let parser = construct!([abc, def]).to_options();
//     assert_eq!(usage(&parser), "([-a] -- B C | [-d] -- E F)");
// }
//
// #[test]
// fn quadruple_strict_many() {
//     let a = short('a').switch();
//     let b = positional::<usize>("B").strict().many();
//     let ab = construct!(a, b);
//
//     let d = short('d').switch();
//     let f = positional::<usize>("F").strict().many();
//     let df = construct!(d, f);
//
//     let parser = construct!([ab, df]).to_options();
//     assert_eq!(usage(&parser), "([-a] -- [B]... | [-d] -- [F]...)");
// }
