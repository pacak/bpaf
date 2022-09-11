use crate::Top;
use quote::{quote, ToTokens};
use syn::parse_quote;

#[test]
fn cargo_command_helper() {
    let top: Top = parse_quote! {
        #[bpaf(options("asm"))]
        struct Opts {
            verbose: bool
        }
    };

    let expected = quote! {
        fn opts() -> ::bpaf::OptionParser<Opts> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                ::bpaf::cargo_helper("asm", {
                    let verbose = ::bpaf::long("verbose").switch();
                    ::bpaf::construct!(Opts { verbose })
                })
            }
            .to_options()
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn top_struct_construct() {
    let top: Top = parse_quote! {
        #[bpaf(construct)]
        struct Opt { verbose: bool }
    };

    let expected = quote! {
        fn opt() -> impl ::bpaf::Parser<Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let verbose = ::bpaf::long("verbose").switch();
                ::bpaf::construct!(Opt { verbose })
            }
        }
    };

    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn top_enum_construct() {
    let top: Top = parse_quote! {
        #[bpaf(construct)]
        enum Opt { Foo { verbose_name: bool }}
    };

    let expected = quote! {
        fn opt() -> impl ::bpaf::Parser<Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let verbose_name = ::bpaf::long("verbose-name").switch();
                ::bpaf::construct!(Opt::Foo { verbose_name })
            }
        }
    };

    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn top_struct_options1() {
    let top: Top = parse_quote! {
        /// those are options
        #[bpaf(options)]
        struct Opt {}
    };

    let expected = quote! {
        fn opt() -> ::bpaf::OptionParser<Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
                {
                    ::bpaf::construct!(Opt {})
                }
                .to_options()
                .descr("those are options")
        }
    };

    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn struct_options2() {
    let input: Top = parse_quote! {
        #[bpaf(options)]
        /// those are options
        struct Opt {}
    };

    let expected = quote! {
        fn opt() -> ::bpaf::OptionParser<Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                ::bpaf::construct!(Opt {})
            }
            .to_options()
            .descr("those are options")
        }
    };

    assert_eq!(input.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn struct_command() {
    let input: Top = parse_quote! {
        /// those are options
        #[bpaf(command)]
        struct Opt {}
    };

    let expected = quote! {
        fn opt() -> impl ::bpaf::Parser<Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let inner_cmd =
                    { ::bpaf::construct!(Opt {}) }
                    .to_options()
                    .descr("those are options")
                ;
                ::bpaf::command("opt", inner_cmd).help("those are options")
            }
        }
    };
    assert_eq!(input.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn enum_command() {
    let input: Top = parse_quote! {
        /// those are options
        enum Opt {
            #[bpaf(command("foo"))]
            /// foo doc
            Foo { field: usize },
            /// bar doc
            #[bpaf(command)]
            Bar { field: bool }
        }
    };

    let expected = quote! {
        fn opt() -> impl ::bpaf::Parser<Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = {
                    let inner_cmd = {
                        let field = ::bpaf::long("field").argument("ARG").from_str::<usize>();
                        ::bpaf::construct!(Opt::Foo { field })
                    }
                    .to_options()
                    .descr("foo doc");
                    ::bpaf::command("foo", inner_cmd).help("foo doc")
                };
                let alt1 = {
                    let inner_cmd = {
                        let field = ::bpaf::long("field").switch();
                        ::bpaf::construct!(Opt::Bar { field })
                    }
                    .to_options()
                    .descr("bar doc");
                    ::bpaf::command("bar", inner_cmd).help("bar doc")
                };
                ::bpaf::construct!([alt0, alt1])
            }
        }
    };
    assert_eq!(input.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn unnamed_struct() {
    let top: Top = parse_quote! {
        #[bpaf(options)]
        struct Opt(
            /// help
            PathBuf
        );
    };

    let expected = quote! {
        fn opt() -> ::bpaf::OptionParser<Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let f0 = ::bpaf::positional_os("ARG").help("help").map(PathBuf::from);
                ::bpaf::construct!(Opt(f0))
            }
            .to_options()
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn unnamed_enum() {
    let top: Top = parse_quote! {
        #[bpaf(options, version)]
        enum Opt1 {
            Con1(PathBuf, usize)
        }
    };

    let expected = quote! {
        fn opt1() -> ::bpaf::OptionParser<Opt1> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let f0 = ::bpaf::positional_os("ARG").map(PathBuf::from);
                let f1 = ::bpaf::positional("ARG").from_str::<usize>();
                ::bpaf::construct!(Opt1::Con1(f0, f1))
            }
            .to_options()
            .version(env!("CARGO_PKG_VERSION"))
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn enum_to_flag_and_switches() {
    let top: Top = parse_quote! {
        #[bpaf(construct)]
        pub enum Opt {
            #[bpaf(long("Foo"))]
            Foo,
            #[bpaf(short)]
            Pff,
            BarFoo,
            Baz(#[bpaf(long("bazz"))] String),
            Strange { strange: String },
            #[bpaf(command("alpha"))]
            Alpha,
            #[bpaf(command)]
            Omega,
        }
    };

    let expected = quote! {
        pub fn opt() -> impl ::bpaf::Parser<Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = ::bpaf::long("Foo").req_flag(Opt::Foo);
                let alt1 = ::bpaf::short('p').req_flag(Opt::Pff);
                let alt2 = ::bpaf::long("bar-foo").req_flag(Opt::BarFoo);
                let alt3 = {
                    let f0 = ::bpaf::long("bazz").argument("ARG");
                    ::bpaf::construct!(Opt::Baz(f0))
                };
                let alt4 = {
                    let strange = ::bpaf::long("strange").argument("ARG");
                    ::bpaf::construct!(Opt::Strange { strange })
                };
                let alt5 = {
                    let inner_cmd = ::bpaf::pure(Opt::Alpha).to_options();
                    ::bpaf::command("alpha", inner_cmd)
                };
                let alt6 = {
                    let inner_cmd = ::bpaf::pure(Opt::Omega).to_options() ;
                    ::bpaf::command("omega", inner_cmd)
                };
                ::bpaf::construct!([alt0, alt1, alt2, alt3, alt4, alt5, alt6])
            }
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn help_generation() {
    let top: Top = parse_quote! {
        /// descr
        ///   a
        ///
        ///
        ///
        ///
        /// footer
        ///  a
        #[bpaf(options)]
        struct Opt(PathBuf);
    };

    let expected = quote! {
        fn opt() -> ::bpaf::OptionParser<Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let f0 = ::bpaf::positional_os("ARG").map(PathBuf::from);
                ::bpaf::construct!(Opt(f0))
            }
            .to_options()
            .descr("descr\n  a")
            .footer("footer\n a")
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn version_with_commands() {
    let top: Top = parse_quote! {
        #[bpaf(options, version)]
        enum Action {
            Alpha,
            Beta,
        }
    };
    let expected = quote! {
        fn action() -> ::bpaf::OptionParser<Action> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = ::bpaf::long("alpha").req_flag(Action::Alpha);
                let alt1 = ::bpaf::long("beta").req_flag(Action::Beta);
                ::bpaf::construct!([alt0, alt1])
            }
            .to_options()
            .version(env!("CARGO_PKG_VERSION"))
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn command_with_aliases() {
    let top: Top = parse_quote! {
        #[bpaf(command, short('c'), long("long"))]
        struct Command {
            i: bool,
        }
    };

    let expected = quote! {
        fn command() -> impl ::bpaf::Parser<Command> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let inner_cmd = {
                    let i = ::bpaf::short('i').switch();
                    ::bpaf::construct!(Command { i })
                }
                .to_options();
                ::bpaf::command("command", inner_cmd)
                    .short('c')
                    .long("long")
            }
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn version_with_commands_with_cargo_helper() {
    let top: Top = parse_quote! {
        #[bpaf(options("subcargo"), version)]
        enum Action {
            #[bpaf(command)]
            Alpha,
            #[bpaf(command)]
            Beta,
        }
    };

    let expected = quote! {
        fn action() -> ::bpaf::OptionParser<Action> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                ::bpaf::cargo_helper("subcargo", {
                    let alt0 = {
                        let inner_cmd = ::bpaf::pure(Action::Alpha).to_options();
                        ::bpaf::command("alpha", inner_cmd)
                    };
                    let alt1 = {
                        let inner_cmd = ::bpaf::pure(Action::Beta).to_options();
                        ::bpaf::command("beta", inner_cmd)
                    };
                    ::bpaf::construct!([alt0, alt1])
                })
            }
            .to_options()
            .version(env!("CARGO_PKG_VERSION"))
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn named_to_positional_with_metavar() {
    let top: Top = parse_quote! {
        struct Options {
            #[bpaf(positional_os("PATH"))]
            path: PathBuf,
        }

    };

    let expected = quote! {
        fn options() -> impl ::bpaf::Parser<Options> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let path = ::bpaf::positional_os("PATH").map(PathBuf::from);
                ::bpaf::construct!(Options { path })
            }
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn named_to_positional_without_metavar() {
    let top: Top = parse_quote! {
        struct Options {
            #[bpaf(positional_os)]
            path: PathBuf,
        }

    };

    let expected = quote! {
        fn options() -> impl ::bpaf::Parser<Options> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let path = ::bpaf::positional_os("ARG").map(PathBuf::from);
                ::bpaf::construct!(Options { path })
            }
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn comp_visibility_struct() {
    let top: Top = parse_quote! {
        #[bpaf(complete_style(x))]
        pub struct Options {
            path: PathBuf,
        }
    };
    let expected = quote! {
        pub fn options() -> impl ::bpaf::Parser<Options> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let path = ::bpaf::long("path").argument_os("ARG").map(PathBuf::from);
                :: bpaf :: construct ! (Options { path })
            }.complete_style(x)
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn comp_visibility_enum() {
    let top: Top = parse_quote! {
        #[bpaf(complete_style(x))]
        pub enum Foo {
            Bar {
                path: PathBuf,
            }
        }
    };
    let expected = quote! {
        pub fn foo() -> impl ::bpaf::Parser<Foo> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let path = ::bpaf::long("path").argument_os("ARG").map(PathBuf::from);
                :: bpaf :: construct ! (Foo::Bar { path })
            }
            .complete_style(x)
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn private_visibility() {
    let top: Top = parse_quote! {
        #[bpaf(private)]
        pub struct Options {
            path: PathBuf,
        }

    };

    let expected = quote! {
        fn options() -> impl ::bpaf::Parser<Options> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let path = ::bpaf::long("path").argument_os("ARG").map(PathBuf::from);
                ::bpaf::construct!(Options { path })
            }
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn hidden_default_enum_singleton() {
    let top: Top = parse_quote! {
        enum Decision {
            /// HALP
            #[bpaf(long("YES"))]
            Yes,
            #[bpaf(default, hide)]
            No,
            #[bpaf(env("x"))]
            Maybe,
            #[bpaf(long("dunno"))]
            Dunno,
            #[bpaf(short('u'))]
            Umm,
            #[bpaf(short('U'))]
            Ummmmmmm,
        }
    };

    let expected = quote! {
        fn decision() -> impl ::bpaf::Parser<Decision> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = ::bpaf::long("YES").help("HALP").req_flag(Decision::Yes);
                let alt1 = ::bpaf::long("no")
                    .flag(Decision::No, Decision::No)
                    .hide();
                let alt2 = ::bpaf::long("maybe").env("x").req_flag(Decision::Maybe);
                let alt3 = ::bpaf::long("dunno").req_flag(Decision::Dunno);
                let alt4 = ::bpaf::short('u').req_flag(Decision::Umm);
                let alt5 = ::bpaf::short('U').req_flag(Decision::Ummmmmmm);
                ::bpaf::construct!([alt0, alt1, alt2, alt3, alt4, alt5])
            }
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
#[should_panic(expected = "Not a valid inner attribute")]
fn enum_singleton_unk() {
    let _top: Top = parse_quote! {
        enum X {
            #[bpaf(zzz)]
            Y
        }
    };
}

#[test]
fn fallback_for_enum() {
    let top: Top = parse_quote! {
        #[bpaf(fallback(Decision::No))]
        enum Decision {
            Yes,
            No,
        }
    };

    let expected = quote! {
        fn decision() -> impl ::bpaf::Parser<Decision> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = ::bpaf::long("yes").req_flag(Decision::Yes);
                let alt1 = ::bpaf::long("no").req_flag(Decision::No);
                ::bpaf::construct!([alt0, alt1])
            }
            .fallback(Decision::No)
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn fallback_for_struct() {
    let top: Top = parse_quote! {
        #[bpaf(fallback(Value { count: 10 }))]
        struct Value {
            count: usize,
        }
    };

    let expected = quote! {
        fn value() -> impl ::bpaf::Parser<Value> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let count = ::bpaf::long("count").argument("ARG").from_str::<usize>();
                ::bpaf::construct!(Value { count })
            }
            .fallback(Value { count: 10 })
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}
