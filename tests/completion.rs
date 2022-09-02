#![allow(clippy::ptr_arg)]

use bpaf::*;
#[test]
fn static_complete_test_1() {
    let a = short('a').long("avocado").help("Use avocado").switch();
    let b = short('b').long("banana").help("Use banana").switch();
    let bb = long("bananananana").help("I'm Batman").switch();
    let c = long("calculator")
        .help("calculator expression")
        .argument("EXPR");
    let parser = construct!(a, b, bb, c).to_options();

    let r = parser
        .run_inner(Args::from(&["--"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
--avocado\tUse avocado
--banana\tUse banana
--bananananana\tI'm Batman
--calculator\tcalculator expression
"
    );

    let r = parser
        .run_inner(Args::from(&["-vvvv"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-vvvv\n");

    let r = parser
        .run_inner(Args::from(&["-v"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "");

    let r = parser
        .run_inner(Args::from(&["--b"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
--banana\tUse banana
--bananananana\tI'm Batman
"
    );

    let r = parser
        .run_inner(Args::from(&["--a"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();

    assert_eq!(r, "--avocado\n");

    let r = parser
        .run_inner(Args::from(&["--banana"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
--banana\tUse banana
--bananananana\tI'm Batman
"
    );

    let r = parser
        .run_inner(Args::from(&["--bananan"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--bananananana\n");

    let r = parser
        .run_inner(Args::from(&["-b"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--banana\n");
}

#[test]
fn long_and_short_arguments() {
    let parser = short('p').long("potato").argument("POTATO").to_options();

    let r = parser
        .run_inner(Args::from(&["-p"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--potato\n");

    let r = parser
        .run_inner(Args::from(&["-p", ""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "<POTATO>\n");

    let r = parser
        .run_inner(Args::from(&["-p", "x"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "x\n");
}

#[test]
fn short_command_alias() {
    let a = long("potato")
        .argument("A")
        .to_options()
        .command("cmd_a")
        .short('a');
    let b = long("potato")
        .argument("A")
        .to_options()
        .command("cmd_b")
        .short('b');
    let parser = construct!([a, b]).to_options();

    let r = parser
        .run_inner(Args::from(&["a"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "cmd_a\n");

    let r = parser
        .run_inner(Args::from(&["cmd_a"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "cmd_a\n");

    let r = parser
        .run_inner(Args::from(&["b", ""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--potato\n");
}

#[test]
fn single_command_completes_to_full() {
    let parser = short('a').switch().to_options().command("cmd").to_options();

    let r = parser
        .run_inner(Args::from(&["c"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "cmd\n");

    let r = parser
        .run_inner(Args::from(&["cmd"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "cmd\n");
}

#[test]
fn static_complete_test_2() {
    let a = long("potato")
        .argument("SHAPE")
        .to_options()
        .command("check")
        .short('c')
        .help("check packages");
    let b = long("megapotato")
        .argument("MEGA")
        .to_options()
        .command("clean")
        .help("clean target dir");
    let c = long("makan")
        .argument("BKT")
        .to_options()
        .command("build")
        .short('b')
        .help("build project");

    let parser = construct!([a, b, c]).to_options();

    let r = parser
        .run_inner(Args::from(&["check", ""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--potato\n");

    let r = parser
        .run_inner(Args::from(&["check"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "check\n");

    let r = parser
        .run_inner(Args::from(&["c"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
check\tcheck packages
clean\tclean target dir
"
    );

    let r = parser
        .run_inner(Args::from(&["c", "--p"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--potato\n");

    let r = parser
        .run_inner(Args::from(&["x"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "");

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
check\tcheck packages
clean\tclean target dir
build\tbuild project
"
    );

    let r = parser
        .run_inner(Args::from(&["ch"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "check\n");
}

#[test]
fn static_complete_test_3() {
    let a = long("potato").help("po").argument("P");
    let b = long("banana").help("ba").argument("B");
    let ab = construct!(a, b);
    let c = long("durian").argument("D");
    let parser = construct!(ab, c).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--potato\tpo\n--banana\tba\n--durian\n");

    let r = parser
        .run_inner(Args::from(&["-"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--potato\tpo\n--banana\tba\n--durian\n");

    let r = parser
        .run_inner(Args::from(&["--"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--potato\tpo\n--banana\tba\n--durian\n");

    let r = parser
        .run_inner(Args::from(&["--d"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--durian\n");
}

#[test]
fn static_complete_test_4() {
    let a = short('a').argument("A");
    let b = short('b').argument("B");
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&["-a", ""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "<A>\n");

    let r = parser
        .run_inner(Args::from(&["-a"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n");

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n-b\n");

    let r = parser
        .run_inner(Args::from(&["-"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n-b\n");

    let r = parser
        .run_inner(Args::from(&["--"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "");
}

#[test]
fn static_complete_test_5() {
    let a = short('a').argument("A");
    let b = short('b').argument("B");
    let c = short('c').argument("C");
    let d = short('d').argument("D");
    let ab = construct!(a, b);
    let cd = construct!(c, d);
    let parser = construct!(ab, cd).to_options();

    let r = parser
        .run_inner(Args::from(&["-a", "x", ""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-b\n-c\n-d\n");

    let r = parser
        .run_inner(Args::from(&["-a", ""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "<A>\n");

    let r = parser
        .run_inner(Args::from(&["-a"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n");

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n-b\n-c\n-d\n");

    let r = parser
        .run_inner(Args::from(&["-"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n-b\n-c\n-d\n");

    let r = parser
        .run_inner(Args::from(&["--"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "");
}

#[test]
fn static_complete_test_6() {
    let a = short('a').argument("A").optional();
    let b = short('b').argument("B").many();
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n-b\n");

    let r = parser
        .run_inner(Args::from(&["-a", "x", ""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-b\n");

    let r = parser
        .run_inner(Args::from(&["-a", ""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "<A>\n");

    let r = parser
        .run_inner(Args::from(&["-a"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n");

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n-b\n");

    let r = parser
        .run_inner(Args::from(&["-b", ""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "<B>\n");

    let r = parser
        .run_inner(Args::from(&["-b", "x", ""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n-b\n");
}

#[test]
fn static_complete_test_7() {
    let a = short('a').help("switch").switch();
    let b = positional("FILE").help("File to use");
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\tswitch\n<FILE>\tFile to use\n");

    let r = parser
        .run_inner(Args::from(&["-a", ""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "<FILE>\n");
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
        .run_inner(Args::from(&[""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "nom\n");

    let r = parser
        .run_inner(Args::from(&["nom", ""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--durian\n");

    let r = parser
        .run_inner(Args::from(&["nom", "-a"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--durian\n");

    let r = parser
        .run_inner(Args::from(&["nom", "-a", ""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "");
}

#[test]
fn just_positional() {
    let parser = positional("FILE").help("File to use").to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "<FILE>\n");

    let r = parser
        .run_inner(Args::from(&["xxx"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "xxx\n");
}

#[test]
fn dynamic_complete_test_1() {
    fn completer(input: &String) -> Vec<(&'static str, Option<&'static str>)> {
        let items = ["alpha", "beta", "banana", "cat", "durian"];
        items
            .iter()
            .filter(|item| item.starts_with(input))
            .map(|item| (*item, None))
            .collect::<Vec<_>>()
    }

    let parser = short('a').argument("ARG").complete(completer).to_options();

    let r = parser
        .run_inner(Args::from(&["-a", "b"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "beta\nbanana\n");

    let r = parser
        .run_inner(Args::from(&["-a", "be"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "beta\n");

    let r = parser
        .run_inner(Args::from(&["-a", "beta"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "beta\n");

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n");

    let r = parser
        .run_inner(Args::from(&["-a", ""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "alpha\nbeta\nbanana\ncat\ndurian\n");
}

#[test]
fn dynamic_complete_test_2() {
    let parser = short('a').argument("ARG").to_options();

    // we don't know how to complete "b", compgen in bash returns an empty line, so should we
    let r = parser
        .run_inner(Args::from(&["-a", "b"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "b\n");
}

#[test]
fn dynamic_complete_test_3() {
    fn complete_calculator(input: &String) -> Vec<(&'static str, Option<&'static str>)> {
        let items = ["alpha", "beta", "banana", "cat", "durian"];
        items
            .iter()
            .filter(|item| item.starts_with(input))
            .map(|item| (*item, None))
            .collect::<Vec<_>>()
    }

    let a = short('a').long("avocado").help("Use avocado").switch();
    let b = short('b').long("banana").help("Use banana").switch();
    let bb = long("bananananana").help("I'm Batman").switch();
    let c = long("calculator")
        .help("calculator expression")
        .argument("EXPR")
        .complete(complete_calculator);
    let parser = construct!(a, b, bb, c).to_options();

    let r = parser
        .run_inner(Args::from(&["--calculator", ""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!("alpha\nbeta\nbanana\ncat\ndurian\n", r);
}

#[test]
fn dynamic_complete_test_4() {
    fn complete_calculator(input: &String) -> Vec<(&'static str, Option<&'static str>)> {
        let names = ["Yuri", "Lupusregina", "Solution", "Shizu", "Entoma"];
        names
            .iter()
            .filter(|item| item.starts_with(input))
            .map(|item| (*item, Some(*item)))
            .collect::<Vec<_>>()
    }

    let parser = long("name")
        .argument("NAME")
        .complete(complete_calculator)
        .to_options();

    let r = parser
        .run_inner(Args::from(&["--name", ""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "Yuri\tYuri\nLupusregina\tLupusregina\nSolution\tSolution\nShizu\tShizu\nEntoma\tEntoma\n"
    );

    let r = parser
        .run_inner(Args::from(&["--name", "L"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "Lupusregina\n");
}

#[test]
fn static_with_hide() {
    let a = short('a').switch();
    let b = short('b').switch().hide();
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n");
}

#[test]
fn static_with_fallback_and_hide() {
    let a = short('a').switch();
    let b = short('b').switch().hide();
    let parser = construct!(a, b).fallback((false, false)).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n");
}

#[test]
fn csample_mystery() {
    fn complete_calculator(input: &String) -> Vec<(&'static str, Option<&'static str>)> {
        let items = ["alpha", "beta", "banana", "cat", "durian"];
        items
            .iter()
            .filter(|item| item.starts_with(input))
            .map(|item| (*item, None))
            .collect::<Vec<_>>()
    }

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
        .footer("footer");

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--avocado\tUse avocado\n--banana\tUse banana\n--bananananana\tI'm Batman\n--calculator\tcalculator expression\n");
}

#[test]
fn only_positionals_after_double_dash() {
    let a = short('a').switch();
    let b = short('b').switch();
    let c = short('c').switch();
    let d = positional("D");
    let parser = construct!(a, b, c, d).to_options();

    let r = parser
        .run_inner(Args::from(&["-a"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n");

    let r = parser
        .run_inner(Args::from(&["-a", ""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-b\n-c\n<D>\n");

    let r = parser
        .run_inner(Args::from(&["-a", "--"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--\n");

    let r = parser
        .run_inner(Args::from(&["--", ""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "<D>\n");
}

#[test]
fn many_does_not_duplicate_metadata() {
    let parser = positional("D").many().to_options();
    let r = parser
        .run_inner(Args::from(&["xxx"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "xxx\n");
}

#[test]
fn some_does_not_duplicate_metadata() {
    let parser = positional("D").some("").to_options();
    let r = parser
        .run_inner(Args::from(&["xxx"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "xxx\n");
}

#[test]
fn only_positionals_after_positionals() {
    let a = short('a').switch();
    let d = positional("D").many();
    let parser = construct!(a, d).to_options();

    let r = parser
        .run_inner(Args::from(&["xxx"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "xxx\n");
}

#[test]
fn positionals_complete_in_order() {
    fn c_a(_input: &String) -> Vec<(String, Option<String>)> {
        vec![("a".to_string(), None)]
    }

    fn c_b(_input: &String) -> Vec<(String, Option<String>)> {
        vec![("b".to_string(), None)]
    }

    let a = positional("A").complete(c_a);
    let b = positional("B").complete(c_b);
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "a\n");

    let r = parser
        .run_inner(Args::from(&["xxx", ""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "b\n");
}

#[test]
fn should_be_able_to_suggest_positional_along_with_non_positionals_flags() {
    fn c_a(_input: &String) -> Vec<(String, Option<String>)> {
        vec![("a".to_string(), None)]
    }
    fn c_b(_input: &String) -> Vec<(String, Option<String>)> {
        vec![("b".to_string(), None)]
    }

    let a = short('a').argument("A").complete(c_a);
    let b = positional("B").complete(c_b);
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\nb\n");
}

#[test]
fn should_be_able_to_suggest_double_dash() {
    fn c_b(_input: &String) -> Vec<(String, Option<String>)> {
        vec![("--".to_string(), None)]
    }
    let a = long("arg").argument("ARG").optional();
    let b = positional("B").complete(c_b);
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--arg\n--\n");

    let r = parser
        .run_inner(Args::from(&["--"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--arg\n--\n");
}

#[test]
fn suggest_double_dash_automatically_for_strictly_positional() {
    let a = short('a').switch();
    let b = positional("B").strict();
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n--\t-- Positional only items\n");

    let r = parser
        .run_inner(Args::from(&["-"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n--\t-- Positional only items\n");

    let r = parser
        .run_inner(Args::from(&["--"]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--\n");

    let r = parser
        .run_inner(Args::from(&["--", ""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "<B>\n");
}

#[track_caller]
fn test_zsh_comp<T: std::fmt::Debug>(
    parser: &OptionParser<T>,
    args: &[&str],
    expected: &[[&str; 4]],
) {
    let expected_len = expected.len();
    let mut actual_len = 0;
    let padding = std::iter::repeat(&["\0"; 4]);
    let padded = expected.iter().chain(padding);
    for (ix, (line, expected)) in parser
        .run_inner(Args::from(args).set_comp(2))
        .unwrap_err()
        .unwrap_stdout()
        .lines()
        .zip(padded)
        .enumerate()
    {
        actual_len += 1;
        if expected[0] == "\0" {
            panic!(
                "expected {} items, but output contains at least {}. unexpected: {:?}",
                expected_len, actual_len, line,
            );
        }
        let mut actual_words = ["\0"; 4];
        for (ix, w) in line.split('\0').enumerate() {
            actual_words[ix] = w;
        }
        assert_eq!(&actual_words, expected, "on line {}", ix)
    }

    assert_eq!(actual_len, expected_len);
}

#[test]
fn zsh_style_completion_visible() {
    let a = short('a')
        .long("argument")
        .help("this is an argument")
        .argument("ARG");
    let b = short('b').argument("BANANA");
    let parser = construct!(a, b)
        .complete_style(CompleteDecor::VisibleGroup("items"))
        .to_options();

    test_zsh_comp(
        &parser,
        &[""],
        &[
            [
                "--argument",
                "--argument <ARG>    this is an argument",
                "items",
                "items",
            ],
            ["-b", "-b <BANANA>", "items", "items"],
        ],
    );
}

#[test]
fn zsh_style_completion_hidden() {
    let a = short('a')
        .long("argument")
        .help("this is an argument")
        .argument("ARG");
    let b = short('b').argument("BANANA");
    let parser = construct!(a, b)
        .complete_style(CompleteDecor::HiddenGroup("items"))
        .to_options();

    test_zsh_comp(
        &parser,
        &[""],
        &[
            [
                "--argument",
                "--argument <ARG>    this is an argument",
                "",
                "items",
            ],
            ["-b", "-b <BANANA>", "", "items"],
        ],
    );
}

#[test]
fn zsh_many_positionals() {
    let parser = positional("POS").many().to_options();
    test_zsh_comp(&parser, &[""], &[["<POS>", "<POS>", "", ""]]);
    test_zsh_comp(&parser, &["p"], &[["p", "p", "", ""]]);
}

#[test]
fn zsh_help_single_line_only() {
    let parser = short('a').help("hello\nworld").argument("X").to_options();
    test_zsh_comp(&parser, &[""], &[["-a", "-a <X>    hello", "", ""]]);
}

#[test]
fn bash_help_single_line_only() {
    let a = short('a').help("hello\nworld").argument("X");
    let b = short('b').help("hello\nworld").argument("X");
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(1))
        .unwrap_err()
        .unwrap_stdout();

    assert_eq!(r, "-a\thello\n-b\thello\n");
}

#[test]
fn derive_decorations() {
    #[derive(Debug, Clone, Bpaf)]
    #[allow(dead_code)]
    #[bpaf(complete_style(CompleteDecor::VisibleGroup("== Cargo options")))]
    struct CargoOpts {
        /// optimize
        release: bool,
        /// pick target
        target: String,
    }

    #[derive(Debug, Clone, Bpaf)]
    #[allow(dead_code)]
    #[bpaf(complete_style(CompleteDecor::VisibleGroup("== Application options")))]
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

    let opts = opts();

    test_zsh_comp(
        &opts,
        &[""],
        &[
            [
                "--release",
                "--release    optimize",
                "== Cargo options",
                "== Cargo options",
            ],
            [
                "--target",
                "--target <ARG>    pick target",
                "== Cargo options",
                "== Cargo options",
            ],
            [
                "--focus",
                "--focus <ARG>    pick focus",
                "== Application options",
                "== Application options",
            ],
            [
                "--inline",
                "--inline    inline rust",
                "== Application options",
                "== Application options",
            ],
        ],
    );
}

#[test]
fn zsh_complete_info() {
    fn foo(_input: &String) -> Vec<(&'static str, Option<&'static str>)> {
        vec![("hello", Some("word")), ("sample", None)]
    }
    let parser = short('a').argument("X").complete(foo).to_options();

    test_zsh_comp(&parser, &[""], &[["-a", "-a <X>", "", ""]]);

    test_zsh_comp(&parser, &["-"], &[["-a", "-a <X>", "", ""]]);
    test_zsh_comp(
        &parser,
        &["-a", ""],
        &[
            ["hello", "hello    word", "", ""],
            ["sample", "sample", "", ""],
        ],
    );
}
