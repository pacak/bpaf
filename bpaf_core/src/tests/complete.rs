use crate::{Parser, construct, long, short};

#[test]
fn simple_complete_command() {
    let a = short('a').req_flag('a').to_options().command("alpha");
    let b = short('b').req_flag('b');
    let c = short('c').switch();
    let ab = construct!([a, b]);
    let parser = construct!(ab, c).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    let expected = "alpha (None)\n\
                    -b (None)\n";
    assert_eq!(r, expected);

    // let r = parser.run_inner(("", "-b")).unwrap_err().unwrap_stdout();
    // let expected = "-b (None)\n";
    // assert_eq!(r, expected);
    //
    // let r = parser.run_inner(("-b", "")).unwrap_err().unwrap_stdout();
    // let expected = "-b (None)\n";
    // assert_eq!(r, expected);
}

#[test]
fn simple_complete_named() {
    let a = long("missy").req_flag('a');
    let b = long("missle-launcher").req_flag('b');
    let c = short('m').req_flag('c');
    let abc = construct!([a, b, c]);
    let name = long("name").argument::<String>("NAME");
    let parser = construct!(abc, name).to_options();

    let r = parser.run_inner(("--name=bob", "--missy")).unwrap_err();
    let r = format!("{r:?}");
    assert_eq!(
        r,
        "CompReply([Named { name: Long(\"missy\"), meta: None, help: None }])"
    );

    // let Error::Complete(c) = parser.run_inner(("--name=Bob", "--miss")).unwrap_err() else {
    //     panic!();
    // };
    // let expected = "[Item { name: Long(\"missy\"), meta: None, help: None }, Item { name: Long(\"missle-launcher\"), meta: None, help: None }]";
    // assert_eq!(format!("{:?}", c.as_slice()), expected);
}

#[test]
fn simple_complete_for_value() {
    let a = short('a').req_flag(());
    let b = short('b')
        .argument::<u32>("B")
        .complete(|_s| vec![("42".into(), None)]);
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(("-b", "")).unwrap_err();
    // let r = parser.run_inner(("-b=", "")).unwrap_err();
    let r = parser.run_inner(("", "-b=")).unwrap_err();
    let r = format!("{r:?}");
    assert_eq!(
        r,
        r#"CompReply([Value { group: None, value: "42", hint: None }])"#
    );
}
