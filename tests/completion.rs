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
        .run_inner(Args::from(&["--"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
--avocado            Use avocado
--banana             Use banana
--bananananana       I'm Batman
--calculator <EXPR>  calculator expression
"
    );

    let r = parser
        .run_inner(Args::from(&["-vvvv"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-vvvv\n");

    let r = parser
        .run_inner(Args::from(&["-v"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "");

    let r = parser
        .run_inner(Args::from(&["--b"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
--banana        Use banana
--bananananana  I'm Batman
"
    );

    let r = parser
        .run_inner(Args::from(&["--a"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();

    assert_eq!(r, "--avocado\n");

    let r = parser
        .run_inner(Args::from(&["--banana"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
--banana        Use banana
--bananananana  I'm Batman
"
    );

    let r = parser
        .run_inner(Args::from(&["--bananan"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--bananananana\n");

    let r = parser
        .run_inner(Args::from(&["-b"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--banana\n");
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
        .run_inner(Args::from(&["check"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "check\n");

    let r = parser
        .run_inner(Args::from(&["c"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
check  check packages
clean  clean target dir
"
    );

    let r = parser
        .run_inner(Args::from(&["c", "--p"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--potato\n");

    let r = parser
        .run_inner(Args::from(&["x"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "");

    let r = parser
        .run_inner(Args::from(&[]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\
check  check packages
clean  clean target dir
build  build project
"
    );

    let r = parser
        .run_inner(Args::from(&["ch"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "check\n");

    let r = parser
        .run_inner(Args::from(&["check"]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--potato\n");
}

#[test]
fn static_complete_test_3() {
    let a = long("potato").help("po").argument("P");
    let b = long("banana").help("ba").argument("B");
    let ab = construct!(a, b);
    let c = long("durian").argument("D");
    let parser = construct!(ab, c).to_options();

    let r = parser
        .run_inner(Args::from(&[]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--potato <P>  po\n--banana <B>  ba\n--durian <D>\n");

    let r = parser
        .run_inner(Args::from(&["-"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--potato <P>  po\n--banana <B>  ba\n--durian <D>\n");

    let r = parser
        .run_inner(Args::from(&["--"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--potato <P>  po\n--banana <B>  ba\n--durian <D>\n");

    let r = parser
        .run_inner(Args::from(&["--d"]).set_comp(true))
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
        .run_inner(Args::from(&["-a"]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "<A>\n");

    let r = parser
        .run_inner(Args::from(&["-a"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n");

    let r = parser
        .run_inner(Args::from(&[]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a <A>\n-b <B>\n");

    let r = parser
        .run_inner(Args::from(&["-"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a <A>\n-b <B>\n");

    let r = parser
        .run_inner(Args::from(&["--"]).set_comp(true))
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
        .run_inner(Args::from(&["-a", "x"]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-b <B>\n-c <C>\n-d <D>\n");

    let r = parser
        .run_inner(Args::from(&["-a"]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "<A>\n");

    let r = parser
        .run_inner(Args::from(&["-a"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n");

    let r = parser
        .run_inner(Args::from(&[]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a <A>\n-b <B>\n-c <C>\n-d <D>\n");

    let r = parser
        .run_inner(Args::from(&["-"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a <A>\n-b <B>\n-c <C>\n-d <D>\n");

    let r = parser
        .run_inner(Args::from(&["--"]).set_comp(true))
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
        .run_inner(Args::from(&[]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a <A>\n-b <B>\n");

    let r = parser
        .run_inner(Args::from(&["-a", "x"]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-b\n");

    let r = parser
        .run_inner(Args::from(&["-a"]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "<A>\n");

    let r = parser
        .run_inner(Args::from(&["-a"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n");

    let r = parser
        .run_inner(Args::from(&[]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a <A>\n-b <B>\n");

    let r = parser
        .run_inner(Args::from(&["-b"]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "<B>\n");

    let r = parser
        .run_inner(Args::from(&["-b", "x"]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a <A>\n-b <B>\n");
}

#[test]
fn static_complete_test_7() {
    let a = short('a').help("switch").switch();
    let b = positional("FILE").help("File to use");
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&[]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a      switch\n<FILE>  File to use\n");

    let r = parser
        .run_inner(Args::from(&["-a"]).set_comp(false))
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
        .run_inner(Args::from(&[]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "nom\n");

    let r = parser
        .run_inner(Args::from(&["nom"]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--durian\n");

    let r = parser
        .run_inner(Args::from(&["nom", "-a"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--durian\n");

    let r = parser
        .run_inner(Args::from(&["nom", "-a"]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "");
}

#[test]
fn dynamic_complete_test_1() {
    fn completer(input: Option<&String>) -> Vec<(&'static str, Option<&'static str>)> {
        let items = ["alpha", "beta", "banana", "cat", "durian"];
        items
            .iter()
            .filter(|item| input.map_or(true, |input| item.starts_with(input)))
            .map(|item| (*item, None))
            .collect::<Vec<_>>()
    }

    let parser = short('a').argument("ARG").comp(completer).to_options();

    let r = parser
        .run_inner(Args::from(&["-a", "b"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "beta\nbanana\n");

    let r = parser
        .run_inner(Args::from(&["-a", "be"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "beta\n");

    let r = parser
        .run_inner(Args::from(&["-a", "beta"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "beta\n");

    let r = parser
        .run_inner(Args::from(&[]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n");

    let r = parser
        .run_inner(Args::from(&["-a"]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "alpha\nbeta\nbanana\ncat\ndurian\n");
}

#[test]
fn dynamic_complete_test_2() {
    let parser = short('a').argument("ARG").to_options();

    // we don't know how to complete "b", compgen in bash returns an empty line, so should we
    let r = parser
        .run_inner(Args::from(&["-a", "b"]).set_comp(true))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "b\n");
}

#[test]
fn dynamic_complete_test_3() {
    fn complete_calculator(input: Option<&String>) -> Vec<(&'static str, Option<&'static str>)> {
        let items = ["alpha", "beta", "banana", "cat", "durian"];
        items
            .iter()
            .filter(|item| input.map_or(true, |input| item.starts_with(input)))
            .map(|item| (*item, None))
            .collect::<Vec<_>>()
    }

    let a = short('a').long("avocado").help("Use avocado").switch();
    let b = short('b').long("banana").help("Use banana").switch();
    let bb = long("bananananana").help("I'm Batman").switch();
    let c = long("calculator")
        .help("calculator expression")
        .argument("EXPR")
        .comp(complete_calculator);
    let parser = construct!(a, b, bb, c).to_options();

    let r = parser
        .run_inner(Args::from(&["--calculator"]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!("alpha\nbeta\nbanana\ncat\ndurian\n", r);
}

#[test]
fn dynamic_complete_test_4() {
    fn complete_calculator(input: Option<&String>) -> Vec<(&'static str, Option<&'static str>)> {
        let names = ["Yuri", "Lupusregina", "Solution", "Shizu", "Entoma"];
        names
            .iter()
            .filter(|item| input.map_or(true, |input| item.starts_with(input)))
            .map(|item| (*item, Some(*item)))
            .collect::<Vec<_>>()
    }

    let parser = long("name")
        .argument("NAME")
        .comp(complete_calculator)
        .to_options();

    let r = parser
        .run_inner(Args::from(&["--name"]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "Yuri         Yuri\nLupusregina  Lupusregina\nSolution     Solution\nShizu        Shizu\nEntoma       Entoma\n");

    let r = parser
        .run_inner(Args::from(&["--name", "L"]).set_comp(true))
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
        .run_inner(Args::from(&[]).set_comp(false))
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
        .run_inner(Args::from(&[]).set_comp(false))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "-a\n");
}
