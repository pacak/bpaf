#![allow(clippy::ptr_arg)]
use bpaf::*;

#[test]
fn mixing_shell_and_positional_1_flag_or_pos() {
    let arg = || short('b').help("Option b").req_flag(10);
    let pos = || {
        positional::<String>("DISTANCE")
            .complete_shell(ShellComp::File { mask: None })
            .guard(|s| !s.is_empty(), "unreachable")
            .parse(|s| s.parse::<usize>())
    };

    let r = construct!([arg(), pos()])
        .to_options()
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-b\t-b\t\tOption b\n\nFile { mask: None }\n");
}

#[test]
fn mixing_shell_and_positional_2_arg_or_pos() {
    let arg = || short('b').help("Option b").argument::<usize>("HELLO");
    let pos = || positional::<usize>("DISTANCE").complete_shell(ShellComp::File { mask: None });

    let r = construct!([arg(), pos()])
        .to_options()
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-b\t-b=HELLO\t\tOption b\n\nFile { mask: None }\n");
}

#[test]
fn mixing_shell_and_positional_3_flag_and_pos() {
    let arg = || short('b').help("Option b").req_flag(10);
    let pos = || positional::<usize>("DISTANCE").complete_shell(ShellComp::File { mask: None });

    let r1 = construct!(arg(), pos())
        .to_options()
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r1, "-b\t-b\t\tOption b\n\nFile { mask: None }\n");
}

#[test]
fn mixing_shell_and_positional_4_arg_and_pos() {
    let arg = || short('b').help("Option b").argument::<usize>("HELLO");
    let pos = || positional::<usize>("DISTANCE").complete_shell(ShellComp::File { mask: None });

    let r = construct!(arg(), pos())
        .to_options()
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-b\t-b=HELLO\t\tOption b\n\nFile { mask: None }\n");
}

#[test]
fn static_complete_test_1() {
    let a = short('a').long("avocado").help("Use avocado").switch();
    let b = short('b').long("banana").help("Use banana").switch();
    let bb = long("bananananana").help("I'm Batman").switch();
    let c = long("calculator")
        .help("calculator expression")
        .argument::<String>("EXPR");

    let parser = construct!(a, b, bb, c).to_options();

    let r = parser
        .run_inner(Args::from(&["-b"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--banana");

    let r = parser
        .run_inner(Args::from(&["--"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
--avocado\t--avocado\t\tUse avocado
--banana\t--banana\t\tUse banana
--bananananana\t--bananananana\t\tI'm Batman
--calculator\t--calculator=EXPR\t\tcalculator expression\n\n"
    );

    // this used to be disambiguation, not anymore

    let r = parser
        .run_inner(Args::from(&["-vvvv"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-vvvv\n");

    let r = parser
        .run_inner(Args::from(&["-v"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-v\n");

    let r = parser
        .run_inner(Args::from(&["--b"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
--banana\t--banana\t\tUse banana
--bananananana\t--bananananana\t\tI'm Batman\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["--a"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--avocado");

    let r = parser
        .run_inner(Args::from(&["--banana"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
--banana\t--banana\t\tUse banana
--bananananana\t--bananananana\t\tI'm Batman\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["--bananan"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--bananananana");
}

#[test]
fn long_and_short_arguments() {
    let parser = short('p')
        .long("potato")
        .argument::<String>("POTATO")
        .to_options();

    let r = parser
        .run_inner(Args::from(&["-p"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--potato");

    let r = parser
        .run_inner(Args::from(&["-p", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tPOTATO\t\t\n\n");

    let r = parser
        .run_inner(Args::from(&["-p", "x"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tPOTATO\t\t\n\n");
}

#[test]
fn short_command_alias() {
    let a = long("potato")
        .argument::<String>("A")
        .to_options()
        .command("cmd_a")
        .short('a');

    let b = long("potato")
        .argument::<String>("A")
        .to_options()
        .command("cmd_b")
        .short('b');
    let parser = construct!([a, b]).to_options();

    let r = parser
        .run_inner(Args::from(&["a"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "cmd_a");

    let r = parser
        .run_inner(Args::from(&["cmd_a"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "cmd_a");

    let r = parser
        .run_inner(Args::from(&["b", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--potato");
}

#[test]
fn single_command_completes_to_full() {
    let parser = short('a').switch().to_options().command("cmd").to_options();

    let r = parser
        .run_inner(Args::from(&["c"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "cmd");

    let r = parser
        .run_inner(Args::from(&["cmd"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "cmd");
}

#[test]
fn static_complete_test_2() {
    let a = long("potato")
        .argument::<String>("SHAPE")
        .to_options()
        .command("check")
        .short('C')
        .help("check packages");

    let b = long("megapotato")
        .argument::<String>("MEGA")
        .to_options()
        .command("clean")
        .help("clean target dir");

    let c = long("makan")
        .argument::<String>("BKT")
        .to_options()
        .command("build")
        .short('b')
        .help("build project");

    let parser = construct!([a, b, c]).to_options();

    let r = parser
        .run_inner(Args::from(&["c"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
check\tcheck\t\tcheck packages
clean\tclean\t\tclean target dir\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["check", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--potato");

    let r = parser
        .run_inner(Args::from(&["check"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "check");

    let r = parser
        .run_inner(Args::from(&["C", "--p"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--potato");

    let r = parser
        .run_inner(Args::from(&["x"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "");

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
check\tcheck\t\tcheck packages
clean\tclean\t\tclean target dir
build\tbuild\t\tbuild project\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["ch"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "check");
}

#[test]
fn static_complete_test_3() {
    let a = long("potato").help("po").argument::<String>("P");
    let b = long("banana").help("ba").argument::<String>("B");
    let ab = construct!(a, b);
    let c = long("durian").argument::<String>("D");
    let parser = construct!(ab, c).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();

    assert_eq!(
        r,
        "\
--potato\t--potato=P\t\tpo
--banana\t--banana=B\t\tba
--durian\t--durian=D\t\t\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["-"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();

    assert_eq!(
        r,
        "\
--potato\t--potato=P\t\tpo
--banana\t--banana=B\t\tba
--durian\t--durian=D\t\t\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["--"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
--potato\t--potato=P\t\tpo
--banana\t--banana=B\t\tba
--durian\t--durian=D\t\t\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["--d"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--durian");
}

#[test]
fn static_complete_test_4() {
    let a = short('a').argument::<String>("A");
    let b = short('b').argument::<String>("B");
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&["-a", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tA\t\t\n\n");

    let r = parser
        .run_inner(Args::from(&["-a"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n");

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
-a\t-a=A\t\t
-b\t-b=B\t\t\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["-"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
-a\t-a=A\t\t
-b\t-b=B\t\t\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["--"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--\n");
}

#[test]
fn static_complete_test_5() {
    let a = short('a').argument::<String>("A");
    let b = short('b').argument::<String>("B");
    let c = short('c').argument::<String>("C");
    let d = short('d').argument::<String>("D");
    let ab = construct!(a, b);
    let cd = construct!(c, d);
    let parser = construct!(ab, cd).to_options();

    let r = parser
        .run_inner(Args::from(&["-a", "x", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
-b\t-b=B\t\t
-c\t-c=C\t\t
-d\t-d=D\t\t\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["-a", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tA\t\t\n\n");

    let r = parser
        .run_inner(Args::from(&["-a"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n");

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "-a\t-a=A\t\t
-b\t-b=B\t\t
-c\t-c=C\t\t
-d\t-d=D\t\t\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["-"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();

    assert_eq!(
        r,
        "\
-a\t-a=A\t\t
-b\t-b=B\t\t
-c\t-c=C\t\t
-d\t-d=D\t\t\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["--"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--\n");
}

#[test]
fn static_complete_test_6() {
    let a = short('a').argument::<String>("A").optional();
    let b = short('b').argument::<String>("B").many();
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&["-b", "x", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
-a\t-a=A\t\t
-b\t-b=B\t\t\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["-a", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tA\t\t\n\n");

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
-a\t-a=A\t\t
-b\t-b=B\t\t\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["-a", "x", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-b");

    let r = parser
        .run_inner(Args::from(&["-a"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n");

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
-a\t-a=A\t\t
-b\t-b=B\t\t\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["-b", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tB\t\t\n\n");
}

#[test]
fn static_complete_test_7() {
    let a = short('a').help("switch").switch();
    let b = positional::<String>("FILE").help("File to use");
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
-a\t-a\t\tswitch
\tFILE\t\tFile to use\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["-a", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tFILE\t\tFile to use\n\n");

    let r = parser
        .run_inner(Args::from(&["-a", "x"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tFILE\t\tFile to use\n\n");
}

#[test]
fn static_complete_test_8() {
    let parser = short('a')
        .long("durian")
        .switch()
        .to_options()
        .command("nom")
        .to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "nom");

    let r = parser
        .run_inner(Args::from(&["nom", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--durian");

    let r = parser
        .run_inner(Args::from(&["nom", "-a"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--durian");

    let r = parser
        .run_inner(Args::from(&["nom", "-a", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\n");
}

#[test]
fn just_positional() {
    let parser = positional::<String>("FILE")
        .help("File to use")
        .to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tFILE\t\tFile to use\n\n");

    let r = parser
        .run_inner(Args::from(&["xxx"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tFILE\t\tFile to use\n\n");
}

fn test_completer(input: &String) -> Vec<(&'static str, Option<&'static str>)> {
    let mut vec = test_completer_descr(input);
    vec.iter_mut().for_each(|i| i.1 = None);
    vec
}

fn test_completer2(input: &String) -> Vec<(&'static str, Option<&'static str>)> {
    let items = ["auto", "mala"];
    items
        .iter()
        .filter(|item| item.starts_with(input))
        .map(|item| (*item, None))
        .collect::<Vec<_>>()
}

fn test_completer_descr(input: &String) -> Vec<(&'static str, Option<&'static str>)> {
    let items = ["alpha", "beta", "banana", "cat", "durian"];
    items
        .iter()
        .filter(|item| item.starts_with(input))
        .map(|item| (*item, Some(*item)))
        .collect::<Vec<_>>()
}

#[test]
fn dynamic_complete_test_1() {
    let parser = short('a')
        .argument::<String>("ARG")
        .complete(test_completer)
        .to_options();

    let r = parser
        .run_inner(Args::from(&["-a", "b"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();

    assert_eq!(
        r,
        "\
\tARG\t\t
beta\tbeta\t\t
banana\tbanana\t\t\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["-a", "be"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "beta");

    let r = parser
        .run_inner(Args::from(&["-a", "beta"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "beta");

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a");

    let r = parser
        .run_inner(Args::from(&["-a", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
\tARG\t\t
alpha\talpha\t\t
beta\tbeta\t\t
banana\tbanana\t\t
cat\tcat\t\t
durian\tdurian\t\t\n\n"
    );
}

#[test]
fn dynamic_complete_test_2() {
    let parser = short('a').argument::<String>("ARG").to_options();

    let r = parser
        .run_inner(Args::from(&["-a", "b"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tARG\t\t\n\n");
}

#[test]
fn dynamic_complete_test_3() {
    let a = short('a').long("avocado").help("Use avocado").switch();
    let b = short('b').long("banana").help("Use banana").switch();
    let bb = long("bananananana").help("I'm Batman").switch();
    let c = long("calculator")
        .help("calculator expression")
        .argument::<String>("EXPR")
        .complete(test_completer);
    let parser = construct!(a, b, bb, c).to_options();

    let r = parser
        .run_inner(Args::from(&["--calculator", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
\tEXPR\t\tcalculator expression
alpha\talpha\t\t
beta\tbeta\t\t
banana\tbanana\t\t
cat\tcat\t\t
durian\tdurian\t\t\n\n"
    );
}

#[test]
fn dynamic_complete_test_4() {
    let parser = long("name")
        .argument::<String>("NAME")
        .complete(test_completer_descr)
        .to_options();

    let r = parser
        .run_inner(Args::from(&["--name", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
\tNAME\t\t
alpha\talpha\t\talpha
beta\tbeta\t\tbeta
banana\tbanana\t\tbanana
cat\tcat\t\tcat
durian\tdurian\t\tdurian\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["--name", "a"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "alpha");
}

#[test]
fn static_with_hide() {
    let a = short('a').switch();
    let b = short('b').switch().hide();
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a");
}

#[test]
fn static_with_fallback_and_hide() {
    let a = short('a').switch();
    let b = short('b').switch().hide();
    let parser = construct!(a, b).fallback((false, false)).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a");
}

#[test]
fn csample_mystery() {
    let a = short('a').long("avocado").help("Use avocado").switch();
    let b = short('b').long("banana").help("Use banana").switch();
    let bb = long("bananananana").help("I'm Batman").switch();
    let c = long("calculator")
        .help("calculator expression")
        .argument::<String>("EXPR")
        .complete(test_completer);
    let parser = construct!(a, b, bb, c)
        .to_options()
        .descr("Dynamic autocomplete example")
        .footer("footer");

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
--avocado\t--avocado\t\tUse avocado
--banana\t--banana\t\tUse banana
--bananananana\t--bananananana\t\tI'm Batman
--calculator\t--calculator=EXPR\t\tcalculator expression\n\n"
    );
}

#[test]
fn only_positionals_after_double_dash() {
    let a = short('a').switch();
    let b = short('b').switch();
    let c = short('c').switch();
    let d = positional::<String>("D");
    let parser = construct!(a, b, c, d).to_options();

    let r = parser
        .run_inner(Args::from(&["-a", "--"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--\n");

    let r = parser
        .run_inner(Args::from(&["-a"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a");

    let r = parser
        .run_inner(Args::from(&["-a", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
-b\t-b\t\t
-c\t-c\t\t
\tD\t\t\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["--", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tD\t\t\n\n");
}

#[test]
fn many_does_not_duplicate_metadata() {
    let parser = positional::<String>("D").many().to_options();
    let r = parser
        .run_inner(Args::from(&["xxx"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tD\t\t\n\n");
}

#[test]
fn some_does_not_duplicate_metadata() {
    let parser = positional::<String>("D").some("").to_options();
    let r = parser
        .run_inner(Args::from(&["xxx"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tD\t\t\n\n");
}

#[test]
fn only_positionals_after_positionals() {
    let a = short('a').switch();
    let d = positional::<String>("D").many();
    let parser = construct!(a, d).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
-a\t-a\t\t
\tD\t\t\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["xxx"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tD\t\t\n\n");

    let r = parser
        .run_inner(Args::from(&["xxx", "yyy"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tD\t\t\n\n");

    let r = parser
        .run_inner(Args::from(&["xxx", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\t-a\t\t\n\tD\t\t\n\n");
}

fn complete_alpha(input: &String) -> Vec<(&'static str, Option<&'static str>)> {
    if "alpha".starts_with(input) {
        vec![("alpha", Some("alpha description"))]
    } else {
        Vec::new()
    }
}

fn complete_beta(input: &String) -> Vec<(&'static str, Option<&'static str>)> {
    if "beta".starts_with(input) {
        vec![("beta", Some("beta description"))]
    } else {
        Vec::new()
    }
}

#[test]
fn positionals_complete_in_order() {
    let a = positional::<String>("A").complete(complete_alpha);
    let b = positional::<String>("B").complete(complete_beta);
    let parser = construct!(a, b).to_options();

    // there's no valid completions that start with x - don't render anything
    let r = parser
        .run_inner(Args::from(&["x"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "");

    // same as before - no valid arguments start with y - don't render anything
    let r = parser
        .run_inner(Args::from(&["x", "y"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "");

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "alpha");

    let r = parser
        .run_inner(Args::from(&["a"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "alpha");

    let r = parser
        .run_inner(Args::from(&["xxx", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "beta");

    let r = parser
        .run_inner(Args::from(&["xxx", "b"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "beta");
}

#[test]
fn should_be_able_to_suggest_positional_along_with_non_positionals_flags() {
    let a = short('a').argument::<String>("A").complete(complete_alpha);
    let b = positional::<String>("B").complete(complete_beta);
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
-a\t-a=A\t\t
beta\tbeta\t\tbeta description\n\n"
    );
}

#[test]
fn should_be_able_to_suggest_double_dash() {
    fn c_b(_input: &String) -> Vec<(String, Option<String>)> {
        vec![("--".to_string(), None)]
    }
    let a = long("arg")
        .argument::<String>("ARG")
        .complete(c_b)
        .optional();

    let parser = construct!(a).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--arg");

    let r = parser
        .run_inner(Args::from(&["--arg", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--");
}

#[test]
fn suggest_double_dash_automatically_for_strictly_positional() {
    let a = short('a').switch();
    let b = positional::<String>("B").strict();
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();

    assert_eq!(
        r,
        "\
-a\t-a\t\t
--\t--\t\tPositional only items after this token\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["-"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();

    assert_eq!(
        r,
        "\
-a\t-a\t\t
--\t--\t\tPositional only items after this token\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["--"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--\n");

    let r = parser
        .run_inner(Args::from(&["--", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tB\t\t\n\n");
}

#[test]
fn multiple_adjacent_single_letter_vals() {
    let a = short('a').switch();
    let b = short('b').switch();
    let c = short('c').switch();
    let parser = construct!(a, b, c).to_options();

    // with no input the right behavior is to suggest all the switches
    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\t-a\t\t\n-b\t-b\t\t\n-c\t-c\t\t\n\n");

    // with a single item present separately we should suggest the remaining two
    let r = parser
        .run_inner(Args::from(&["-a", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-b\t-b\t\t\n-c\t-c\t\t\n\n");

    // with a single item present in the same item we still should suggest the remaining two values
    let r = parser
        .run_inner(Args::from(&["-a"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-b\t-b\t\t\n-c\t-c\t\t\n\n");

    // should I change the representation for the completions? Knowing that somethnig is a
    // flag with a short or long name is no longer necessary
    // now I need to be able to represent adding a -v to -f to get -fv, so pretty printed value is
    // -v, substitution is `-fv`...
    //
    //
    // FWIW bash doesn't complete short flags in combination... should we behave the same way?
}

#[test]
fn ambiguity_no_resolve() {
    let a0 = short('a').switch().count();
    let a1 = short('a').argument::<usize>("AAAAAA");
    let parser = construct!([a0, a1]).to_options();

    // -a is a valid argument, proper behavior here is to finalize it
    let r = parser
        .run_inner(Args::from(&["-a"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a");

    let r = parser
        .run_inner(Args::from(&["-aaa"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "");
}

#[test]
fn ambiguity_to_flags() {
    let parser = short('a').switch().many().to_options();

    let r = parser
        .run_inner(Args::from(&["-aaa"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();

    assert_eq!(r, "-aaa\n");
}

#[test]
fn short_argument_variants() {
    let parser = short('a').argument::<String>("META").to_options();
    let r = parser
        .run_inner(Args::from(&["-a=aa"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tMETA\t\t\n\n");

    let r = parser
        .run_inner(Args::from(&["-a", "aa"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tMETA\t\t\n\n");

    let r = parser
        .run_inner(Args::from(&["-aaa"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tMETA\t\t\n\n");
}

#[test]
fn long_argument_variants() {
    let parser = long("alpha")
        .argument::<String>("META")
        .complete(complete_beta)
        .to_options();

    let r = parser
        .run_inner(Args::from(&["--alpha=beta"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--alpha=beta");

    let r = parser
        .run_inner(Args::from(&["--alpha", "Regina"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tMETA\t\t\n\n");
}

#[test]
fn zsh_style_completion_visible() {
    let a = short('a')
        .long("argument")
        .help("this is an argument")
        .argument::<String>("ARG");
    let b = short('b').argument::<String>("BANANA");
    let parser = construct!(a, b).group_help("items").to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
--argument\t--argument=ARG\titems\tthis is an argument
-b\t-b=BANANA\titems\t\n\n"
    );
}

#[test]
fn zsh_many_positionals() {
    let parser = positional::<String>("POS").many().to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tPOS\t\t\n\n");

    let r = parser
        .run_inner(Args::from(&["p"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tPOS\t\t\n\n");
}

#[test]
fn zsh_help_single_line_only() {
    let a = short('a').help("hello world").argument::<String>("X");
    let b = short('b').help("hello from switch").switch();
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();

    assert_eq!(
        r,
        "\
-a\t-a=X\t\thello world
-b\t-b\t\thello from switch\n\n"
    );
}

#[test]
fn shell_help_single_line_only() {
    let a = short('a').help("hello 1\n\nworld").argument::<String>("X");
    let b = short('b').help("hello 2\n\nworld").argument::<String>("Y");
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();

    assert_eq!(
        r,
        "\
-a\t-a=X\t\thello 1
-b\t-b=Y\t\thello 2\n\n"
    );
}

#[test]
fn derive_decorations() {
    #[derive(Debug, Clone, Bpaf)]
    #[allow(dead_code)]
    /// == Cargo options
    struct CargoOpts {
        /// optimize
        release: bool,
        /// pick target
        target: String,
    }

    #[derive(Debug, Clone, Bpaf)]
    #[allow(dead_code)]
    /// == Application options
    struct AppOpts {
        /// pick focus
        focus: String,
        /// inline rust
        inline: bool,
    }

    #[derive(Debug, Clone, Bpaf)]
    #[allow(dead_code)]
    #[bpaf(options)]
    struct Opts {
        #[bpaf(external)]
        cargo_opts: CargoOpts,
        #[bpaf(external)]
        app_opts: AppOpts,
    }

    let parser = opts();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
--release\t--release\t== Cargo options\toptimize
--target\t--target=ARG\t== Cargo options\tpick target
--focus\t--focus=ARG\t== Application options\tpick focus
--inline\t--inline\t== Application options\tinline rust\n\n"
    );
}

#[test]
fn zsh_complete_info() {
    fn foo(_input: &String) -> Vec<(&'static str, Option<&'static str>)> {
        vec![("hello", Some("word")), ("sample", None)]
    }
    let parser = short('a')
        .argument::<String>("X")
        .complete(foo)
        .to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a");

    let r = parser
        .run_inner(Args::from(&["-"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a");

    let r = parser
        .run_inner(Args::from(&["-a", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
\tX\t\t
hello\thello\t\tword
sample\tsample\t\t\n\n"
    );
}

#[test]
fn pair_of_positionals() {
    // with positional items only current item should make suggestions, not both...
    let a = positional::<String>("A").complete(test_completer);
    let b = positional::<String>("B").complete(test_completer2);
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&["a"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "alpha");

    let r = parser
        .run_inner(Args::from(&["alpha", "a"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "auto");
}

#[test]
fn double_dash_as_positional() {
    let parser = positional::<String>("P")
        .help("Help")
        .complete(test_completer)
        .to_options();

    let r = parser
        .run_inner(Args::from(&["--", "a"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "alpha");

    let r = parser
        .run_inner(Args::from(&["a"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "alpha");

    let r = parser
        .run_inner(Args::from(&["-"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-\n");
    //
    let r = parser
        .run_inner(Args::from(&["--"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--\n");

    let r = parser
        .run_inner(Args::from(&["--"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--\n");

    let r = parser
        .run_inner(Args::from(&["x"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "x\n");
    //    assert_eq!(r, "\tP\t\tHelp\n\n");
}

#[test]
fn strict_positional_completion() {
    let a = long("arg").switch();
    let p = positional::<String>("S")
        .strict()
        .complete(|_| vec![("--hello".to_owned(), None)]);
    let parser = construct!(a, p).to_options();

    let r = parser
        .run_inner(Args::from(&["--"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
--arg\t--arg\t\t
--hello\t--hello\t\t\n\n"
    );

    let r = parser
        .run_inner(Args::from(&["--a"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--arg");

    let r = parser
        .run_inner(Args::from(&["--", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--hello");

    let r = parser
        .run_inner(Args::from(&["--", "--h"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--hello");
}

#[test]
fn avoid_inserting_metavars() {
    let parser = short('a').argument::<String>("A").to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a");

    let r = parser
        .run_inner(Args::from(&["-a", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\tA\t\t\n\n");
}

#[test]
fn shell_dir_completion() {
    let parser = short('a')
        .argument::<String>("FILE")
        .complete_shell(ShellComp::Dir { mask: None })
        .to_options();

    let r = parser
        .run_inner(Args::from(&["-a", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "\nDir { mask: None }\n");
}
#[test]
fn generate_unparseable_items() {
    let one = pure(()).to_options().command("cone");
    let two = pure(()).to_options().command("ctwo");
    let e = short('e').switch();

    let one_e = construct!(e, one).map(|x| x.1);
    let parser = construct!([one_e, two]).to_options();

    // passing -e restricts branch with cmd_two
    let r = parser
        .run_inner(Args::from(&["-e", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "cone");

    // passing -e restricts branch with cmd_two
    let r = parser
        .run_inner(Args::from(&["-e", "c"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "cone");

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
-e\t-e\t\t
cone\tcone\t\t
ctwo\tctwo\t\t\n\n"
    );
}

#[test]
fn complete_with_fallback() {
    let parser = long("name")
        .argument::<String>("NAME")
        .complete(test_completer_descr)
        .parse(|x| x.parse::<u16>())
        .fallback(10)
        .to_options();

    let r = parser
        .run_inner(Args::from(&["--name", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
\tNAME\t\t
alpha\talpha\t\talpha
beta\tbeta\t\tbeta
banana\tbanana\t\tbanana
cat\tcat\t\tcat
durian\tdurian\t\tdurian\n\n"
    );
}

#[test]
fn mix_of_options_and_positional_completions() {
    let a = short('a')
        .long("arg")
        .help("Alhpa argument")
        .argument("ALPHA")
        .complete(complete_alpha);
    let b = positional("BETA")
        .help("Beta argument")
        .complete(complete_beta);
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();

    assert_eq!(
        r,
        "\
--arg\t--arg=ALPHA\t\tAlhpa argument
beta\tbeta\t\tbeta description\n\n"
    );
}

#[test]
fn positionals_with_no_completions_are_not_duplicated() {
    let a = short('a')
        .long("arg")
        .help("Alhpa argument")
        .argument::<String>("ALPHA");
    let b = positional::<String>("BETA").help("Beta argument");

    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();

    assert_eq!(
        r,
        "\
--arg\t--arg=ALPHA\t\tAlhpa argument
\tBETA\t\tBeta argument\n\n"
    );
}
