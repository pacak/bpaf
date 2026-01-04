use crate::{
    Parser, Visited, console_writer::apply_style, construct, long, positional, pure, short,
    visitors::usage::Usage,
};

fn usage(visited: &impl Visited) -> String {
    let mut u = Usage::default();
    visited.visit(&mut u);
    let mut out = String::new();
    u.render_to(&mut out);

    let mut s = apply_style(&out, 100, None);
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
fn some_and_req() {
    let parser = ra().some("some").to_options();
    assert_eq!(usage(&parser), "-a...");
}

#[test]
fn usage_choice_req() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let parser = construct!([a, b]).to_options();
    let r = usage(&parser);
    assert_eq!(r, "(-a | -b)");
}

fn ra() -> impl Parser<bool> {
    short('a').req_flag(true)
}

fn oa() -> impl Parser<bool> {
    short('a').switch()
}

fn rb() -> impl Parser<bool> {
    short('b').req_flag(true)
}

fn ob() -> impl Parser<bool> {
    short('b').switch()
}

fn ca() -> impl Parser<bool> {
    pure(true).to_options().command("a")
}

fn cb() -> impl Parser<bool> {
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
