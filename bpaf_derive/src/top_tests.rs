use crate::Top;
use pretty_assertions::assert_eq;
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
            ::bpaf::cargo_helper("asm", {
                let verbose = ::bpaf::long("verbose").switch();
                ::bpaf::construct!(Opts { verbose, })
            })
            .to_options()
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn top_struct_construct() {
    let top: Top = parse_quote! {
        struct Opt { verbose: bool }
    };

    let expected = quote! {
        fn opt() -> impl ::bpaf::Parser<Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let verbose = ::bpaf::long("verbose").switch();
                ::bpaf::construct!(Opt { verbose, })
            }
        }
    };

    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn top_enum_construct() {
    let top: Top = parse_quote! {
        enum Opt { Foo { verbose_name: bool }}
    };

    let expected = quote! {
        fn opt() -> impl ::bpaf::Parser<Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let verbose_name = ::bpaf::long("verbose-name").switch();
                ::bpaf::construct!(Opt::Foo { verbose_name, })
            }
        }
    };

    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn top_struct_options1() {
    let top: Top = parse_quote! {
        /// those are options
        ///
        ///
        /// header
        ///
        ///
        /// footer
        #[bpaf(options, header(h), footer(f))]
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
                .header(h)
                .footer(f)
        }
    };

    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn options_with_custom_usage() {
    let top: Top = parse_quote! {
        #[bpaf(options, usage("App: usage"))]
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
                .usage("App: usage")
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
fn struct_command_no_decor() {
    let input: Top = parse_quote! {
        /// those are options
        #[bpaf(command)]
        struct Opt;
    };

    let expected = quote! {
        fn opt() -> impl ::bpaf::Parser<Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(Opt)
                .to_options()
                .descr("those are options")
                .command("opt")
        }
    };
    assert_eq!(input.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn struct_command_decor() {
    let input: Top = parse_quote! {
        /// those are options
        #[bpaf(command, descr(descr), header(header), footer(footer), help(help))]
        struct Opt;
    };

    let expected = quote! {
        fn opt() -> impl ::bpaf::Parser<Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(Opt)
                .to_options()
                .descr(descr)
                .header(header)
                .footer(footer)
                .command("opt")
                .help(help)
        }
    };
    assert_eq!(input.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn struct_command_short() {
    let input: Top = parse_quote! {
        /// those are options
        #[bpaf(command, short('x'))]
        struct O{ }
    };

    let expected = quote! {
        fn o() -> impl ::bpaf::Parser<O> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                ::bpaf::construct!(O{ })
            }
            .to_options()
            .descr("those are options")
            .command("o")
            .short('x')

        }
    };
    assert_eq!(input.to_token_stream().to_string(), expected.to_string());
}

/*
#[should_panic(expected = "Can't construct a parser from empty enum")]
#[test]
fn empty_enum() {
    let x: Top = parse_quote! {
        enum Opt { }
    };
    todo!("{:?}", x);
}
*/

#[test]
fn unnamed_command_enum() {
    let input: Top = parse_quote! {
        #[bpaf(command)]
        enum Opts {
            #[bpaf(command("alpha1"))]
            Alpha,
            Beta,
            Gamma,
        }
    };

    let expected = quote! {
        fn opts() -> impl ::bpaf::Parser<Opts> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = ::bpaf::pure(Opts::Alpha).to_options().command("alpha1");
                let alt1 = ::bpaf::long("beta").req_flag(Opts::Beta);
                let alt2 = ::bpaf::long("gamma").req_flag(Opts::Gamma);
                ::bpaf::construct!([alt0, alt1, alt2,])
            }
            .to_options()
            .command("opts")
        }
    };
    assert_eq!(input.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn enum_markdownish() {
    let input: Top = parse_quote! {
        enum Opt {
            /// Make a tree
            ///
            ///
            ///
            ///
            /// Examples:
            ///
            /// ```sh
            /// cargo 1
            /// cargo 2
            /// ```
            #[bpaf(command, header("x"))]
            Opt { field: bool },
        }
    };

    let expected = quote! {
        fn opt() -> impl ::bpaf::Parser<Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let field = ::bpaf::long("field").switch();
                ::bpaf::construct!(Opt::Opt { field ,})
            }
            .to_options()
            .footer("Examples:\n\n```sh\ncargo 1\ncargo 2\n```")
            .descr("Make a tree")
            .header("x")
            .command("opt")
        }
    };

    assert_eq!(input.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn enum_command() {
    let input: Top = parse_quote! {
        // those are options
        #[bpaf(options, header(h), footer(f))]
        enum Opt {
            #[bpaf(command("foo"))]
            /// foo doc
            ///
            ///
            /// header
            ///
            ///
            /// footer
            Foo { field: usize },
            /// bar doc
            #[bpaf(command, adjacent)]
            Bar { field: bool }
        }
    };

    let expected = quote! {
        fn opt() -> ::bpaf::OptionParser<Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = {
                    let field = ::bpaf::long("field").argument::<usize>("ARG");
                    ::bpaf::construct!(Opt::Foo { field, })
                }
                .to_options()
                .footer("footer")
                .header("header")
                .descr("foo doc")
                .command("foo");

                let alt1 = {
                    let field = ::bpaf::long("field").switch();
                    ::bpaf::construct!(Opt::Bar { field, })
                }
                .to_options()
                .descr("bar doc")
                .command("bar")
                .adjacent();
                ::bpaf::construct!([alt0, alt1, ])
            }
            .to_options()
            .header(h)
            .footer(f)
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
                let f0 = ::bpaf::positional::<PathBuf>("ARG").help("help");
                ::bpaf::construct!(Opt(f0,))
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
                let f0 = ::bpaf::positional::<PathBuf>("ARG");
                let f1 = ::bpaf::positional::<usize>("ARG");
                ::bpaf::construct!(Opt1::Con1(f0, f1,))
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
        pub enum Opt {
            #[bpaf(long("Foo"), long("fo"))]
            Foo,
            #[bpaf(short)]
            Pff,
            BarFoo,
            Baz(#[bpaf(argument, long("bazz"))] String),
            Strange { strange: String },
            #[bpaf(command("alpha"), usage("custom"))]
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
                let alt0 = ::bpaf::long("Foo").long("fo").req_flag(Opt::Foo);
                let alt1 = ::bpaf::short('p').req_flag(Opt::Pff);
                let alt2 = ::bpaf::long("bar-foo").req_flag(Opt::BarFoo);
                let alt3 = {
                    let f0 = ::bpaf::long("bazz").argument::<String>("ARG");
                    ::bpaf::construct!(Opt::Baz(f0, ))
                };
                let alt4 = {
                    let strange = ::bpaf::long("strange").argument::<String>("ARG");
                    ::bpaf::construct!(Opt::Strange { strange, })
                };
                let alt5 = ::bpaf::pure(Opt::Alpha).to_options().usage("custom").command("alpha");
                let alt6 = ::bpaf::pure(Opt::Omega).to_options().command("omega");
                ::bpaf::construct!([alt0, alt1, alt2, alt3, alt4, alt5, alt6, ])
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
                let f0 = ::bpaf::positional::<PathBuf>("ARG");
                ::bpaf::construct!(Opt(f0, ))
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
                ::bpaf::construct!([alt0, alt1, ])
            }
            .to_options()
            .version(env!("CARGO_PKG_VERSION"))
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn hidden_command() {
    let top: Top = parse_quote! {
        #[bpaf(options)]
        enum Action {
            #[bpaf(command)]
            /// visible help
            Visible,
            /// hidden help
            #[bpaf(command, hide)]
            Hidden,
        }
    };
    let expected = quote! {
        fn action() -> ::bpaf::OptionParser<Action> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = ::bpaf::pure(Action::Visible)
                    .to_options()
                    .descr("visible help")
                    .command("visible");

                let alt1 = ::bpaf::pure(Action::Hidden)
                    .to_options()
                    .descr("hidden help")
                    .command("hidden")
                    .hide();

                ::bpaf::construct!([alt0, alt1, ])
            }
            .to_options()
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn req_flag_struct() {
    let top: Top = parse_quote! {
        struct Foo;
    };

    let expected = quote! {
        fn foo() -> impl ::bpaf::Parser<Foo> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::long("foo").req_flag(Foo)
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn generate_parser() {
    let top: Top = parse_quote! {
            #[bpaf(generate(oof))]
            struct Foo;
    };
    let expected = quote! {
        fn oof() -> impl ::bpaf::Parser<Foo> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::long("foo").req_flag(Foo)
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn generate_options() {
    let top: Top = parse_quote! {
            #[bpaf(options, generate(oof))]
            struct Foo;
    };
    let expected = quote! {
        fn oof() -> ::bpaf::OptionParser<Foo> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::long("foo").req_flag(Foo).to_options()
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn generate_command() {
    let top: Top = parse_quote! {
            #[bpaf(command, generate(oof))]
            struct Foo;
    };
    let expected = quote! {
        fn oof() -> impl ::bpaf::Parser<Foo> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(Foo).to_options().command("foo")
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn command_with_aliases_struct() {
    let top: Top = parse_quote! {
        #[bpaf(command, short('c'), long("long"), long("long2"))]
        /// help
        struct Command {
            i: bool,
        }
    };

    let expected = quote! {
        fn command() -> impl ::bpaf::Parser<Command> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let i = ::bpaf::short('i').switch();
                ::bpaf::construct!(Command { i, })
            }
            .to_options()
            .descr("help")
            .command("command")
            .short('c')
            .long("long")
            .long("long2")
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn command_with_aliases_enum() {
    let top: Top = parse_quote! {
        enum Options {
            #[bpaf(command("command"), short('c'), long("long"), long("long2"))]
            /// help
            Command {
                i: bool,
            }
        }
    };

    let expected = quote! {
        fn options() -> impl ::bpaf::Parser<Options> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let i = ::bpaf::short('i').switch();
                ::bpaf::construct!(Options::Command { i, })
            }
            .to_options()
            .descr("help")
            .command("command")
            .short('c')
            .long("long")
            .long("long2")
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
                ::bpaf::cargo_helper("subcargo", {
                    let alt0 = ::bpaf::pure(Action::Alpha).to_options().command("alpha");
                    let alt1 = ::bpaf::pure(Action::Beta).to_options().command("beta");
                    ::bpaf::construct!([alt0, alt1, ])
                })
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
            #[bpaf(positional("PATH"))]
            path: PathBuf,
        }

    };

    let expected = quote! {
        fn options() -> impl ::bpaf::Parser<Options> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let path = ::bpaf::positional::<PathBuf>("PATH");
                ::bpaf::construct!(Options { path, })
            }
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn named_to_positional_without_metavar() {
    let top: Top = parse_quote! {
        struct Options {
            #[bpaf(positional)]
            path: PathBuf,
        }

    };

    let expected = quote! {
        fn options() -> impl ::bpaf::Parser<Options> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let path = ::bpaf::positional::<PathBuf>("ARG");
                ::bpaf::construct!(Options { path, })
            }
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
                let path = ::bpaf::long("path").argument::<PathBuf>("ARG");
                ::bpaf::construct!(Options { path, })
            }
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn hidden_default_enum_singleton() {
    let top: Top = parse_quote! {
        #[bpaf(fallback(Decision::No))]
        enum Decision {
            /// HALP
            #[bpaf(long("YES"))]
            Yes,
            #[bpaf(hide)]
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
                let alt1 = ::bpaf::long("no").req_flag(Decision::No).hide();
                let alt2 = ::bpaf::env("x").long("maybe").req_flag(Decision::Maybe);
                let alt3 = ::bpaf::long("dunno").req_flag(Decision::Dunno);
                let alt4 = ::bpaf::short('u').req_flag(Decision::Umm);
                let alt5 = ::bpaf::short('U').req_flag(Decision::Ummmmmmm);
                ::bpaf::construct!([alt0, alt1, alt2, alt3, alt4, alt5,])
            }
            .fallback(Decision::No)
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}
/*
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
*/

#[test]
fn explicit_external() {
    let top: Top = parse_quote! {
        #[bpaf(options)]
        struct Options {
            #[bpaf(external(actions), fallback(Action::List))]
            action: Action,
        }
    };

    let expected = quote! {
        fn options() -> ::bpaf::OptionParser<Options> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let action = actions().fallback(Action::List);
                ::bpaf::construct!(Options { action, })
            }
            .to_options()
        }
    };

    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn implicit_external() {
    let top: Top = parse_quote! {
        #[bpaf(options)]
        struct Options {
            #[bpaf(external, fallback(Action::List))]
            action: Action,
        }
    };

    let expected = quote! {
        fn options() -> ::bpaf::OptionParser<Options> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let action = action().fallback(Action::List);
                ::bpaf::construct!(Options { action, })
            }
            .to_options()
        }
    };

    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn fallback_for_options() {
    let top: Top = parse_quote! {
        #[bpaf(options, fallback(Opts::Dummy))]
        enum Opts {
            Llvm,
            Att,
            Dummy,
        }
    };

    let expected = quote! {
        fn opts() -> ::bpaf::OptionParser<Opts> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = ::bpaf::long("llvm").req_flag(Opts::Llvm);
                let alt1 = ::bpaf::long("att").req_flag(Opts::Att);
                let alt2 = ::bpaf::long("dummy").req_flag(Opts::Dummy);
                ::bpaf::construct!([alt0, alt1, alt2,])
            }
            .fallback(Opts::Dummy)
            .to_options()
        }
    };

    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn implicitly_named_switch() {
    let top: Top = parse_quote! {
        #[bpaf(options, fallback(Opts::Dummy),)]
        struct Opts (#[bpaf(long("release"), switch,)] bool);
    };

    let expected = quote! {
        fn opts() -> ::bpaf::OptionParser<Opts> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let f0 = ::bpaf::long("release").switch();
                ::bpaf::construct!(Opts(f0,))
            }
            .fallback(Opts::Dummy)
            .to_options()
        }
    };

    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn fallback_for_enum() {
    let top: Top = parse_quote! {
        #[bpaf(fallback(Decision::No),)]
        enum Decision {
            Yes,
            #[bpaf(short, long("nay"),)]
            No,
            #[bpaf(skip,)]
            Undecided,
        }
    };

    let expected = quote! {
        fn decision() -> impl ::bpaf::Parser<Decision> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = ::bpaf::long("yes").req_flag(Decision::Yes);
                let alt1 = ::bpaf::short('n').long("nay").req_flag(Decision::No);
                ::bpaf::construct!([alt0, alt1,])
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
                let count = ::bpaf::long("count").argument::<usize>("ARG");
                ::bpaf::construct!(Value { count, })
            }
            .fallback(Value { count: 10 })
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn adjacent_for_struct() {
    let top: Top = parse_quote! {
        #[bpaf(adjacent)]
        struct Opts {
            a: String,
            b: String,
        }
    };

    let expected = quote! {
        fn opts() -> impl ::bpaf::Parser<Opts> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let a = ::bpaf::short('a').argument::<String>("ARG");
                let b = ::bpaf::short('b').argument::<String>("ARG");
                ::bpaf::construct!(Opts { a, b, })
            }
            .adjacent()
        }
    };

    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn box_for_struct() {
    let top: Top = parse_quote! {
        #[bpaf(boxed)]
        struct Opts {
            a: String,
            b: String,
        }
    };

    let expected = quote! {
        fn opts() -> impl ::bpaf::Parser<Opts> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let a = ::bpaf::short('a').argument::<String>("ARG");
                let b = ::bpaf::short('b').argument::<String>("ARG");
                ::bpaf::construct!(Opts { a, b, })
            }
            .boxed()
        }
    };

    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn no_fields_declaration() {
    let top: Top = parse_quote! {
        struct Opts {}
    };

    let expected = quote! {
        fn opts() -> impl ::bpaf::Parser<Opts> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                ::bpaf::construct!(Opts {})
            }
        }
    };

    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn single_unit_command() {
    let top: Top = parse_quote! {
        #[bpaf(command)]
        struct One;
    };

    let expected = quote! {
        fn one() -> impl ::bpaf::Parser<One> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(One).to_options().command("one")
        }
    };

    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn single_unit_adjacent_command() {
    let top: Top = parse_quote! {
        #[bpaf(command, adjacent,)]
        struct One;
    };

    let expected = quote! {
        fn one() -> impl ::bpaf::Parser<One> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(One).to_options().command("one").adjacent()
        }
    };

    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn ingore_doc_comment_top_level_1() {
    let top: Top = parse_quote! {
        #[derive(Debug, Clone, Bpaf)]
        /// present
        #[bpaf(ignore_rustdoc)]
        enum Mode {
            /// intel help
            Intel,
            /// att help
            Att,
        }
    };

    let expected = quote! {
        fn mode() -> impl ::bpaf::Parser<Mode> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = ::bpaf::long("intel").help("intel help").req_flag(Mode::Intel);
                let alt1 = ::bpaf::long("att").help("att help").req_flag(Mode::Att);
                ::bpaf::construct!([alt0, alt1, ])
            }
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn ingore_doc_comment_top_level_2() {
    let top: Top = parse_quote! {
        #[derive(Debug, Clone, Bpaf)]
        #[bpaf(options, ignore_rustdoc)]
        /// present
        enum Mode {
            /// intel help
            Intel,
            /// att help
            Att,
        }
    };

    let expected = quote! {
        fn mode() -> ::bpaf::OptionParser<Mode> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = ::bpaf::long("intel").help("intel help").req_flag(Mode::Intel);
                let alt1 = ::bpaf::long("att").help("att help").req_flag(Mode::Att);
                ::bpaf::construct!([alt0, alt1,])
            }
            .to_options()
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn top_comment_is_group_help_enum() {
    let top: Top = parse_quote! {
        #[derive(Debug, Clone, Bpaf)]
        /// present
        enum Mode {
            /// intel help
            Intel,
            /// att help
            Att,
        }
    };

    let expected = quote! {
        fn mode() -> impl ::bpaf::Parser<Mode> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = ::bpaf::long("intel").help("intel help").req_flag(Mode::Intel);
                let alt1 = ::bpaf::long("att").help("att help").req_flag(Mode::Att);
                ::bpaf::construct!([alt0, alt1, ])
            }
            .group_help("present")
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn top_comment_is_group_help_struct() {
    let top: Top = parse_quote! {
        #[derive(Debug, Clone, Bpaf)]
        /// present
        struct Mode {
            /// help
            intel: bool,
            /// help
            att: bool,
        }
    };

    let expected = quote! {
        fn mode() -> impl ::bpaf::Parser<Mode> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let intel = ::bpaf::long("intel").help("help").switch();
                let att = ::bpaf::long("att").help("help").switch();
                ::bpaf::construct!(Mode { intel, att, })
            }
            .group_help("present")
        }
    };
    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn custom_bpaf_path() {
    let input: Top = parse_quote! {
        // those are options
        #[bpaf(options, header(h), footer(f), bpaf_path = ::indirector::bpaf)]
        enum Opt {
            #[bpaf(command("foo"))]
            /// foo doc
            ///
            ///
            /// header
            ///
            ///
            /// footer
            Foo { field: usize },
            /// bar doc
            #[bpaf(command, adjacent)]
            Bar { field: bool }
        }
    };

    let expected = quote! {
        fn opt() -> ::indirector::bpaf::OptionParser<Opt> {
            #[allow(unused_imports)]
            use ::indirector::bpaf::Parser;
            {
                let alt0 = {
                    let field = ::indirector::bpaf::long("field").argument::<usize>("ARG");
                    ::indirector::bpaf::construct!(Opt::Foo { field, })
                }
                .to_options()
                .footer("footer")
                .header("header")
                .descr("foo doc")
                .command("foo");

                let alt1 = {
                    let field = ::indirector::bpaf::long("field").switch();
                    ::indirector::bpaf::construct!(Opt::Bar { field, })
                }
                .to_options()
                .descr("bar doc")
                .command("bar")
                .adjacent();
                ::indirector::bpaf::construct!([alt0, alt1, ])
            }
            .to_options()
            .header(h)
            .footer(f)
        }
    };
    assert_eq!(input.to_token_stream().to_string(), expected.to_string());
}

/*
#[test]
fn push_down_command() {
    let top: Top = parse_quote! {
        #[derive(Bpaf)]
        #[bpaf(command)]
        enum Options {
            Alpha,
            Beta,
        }
    };

    let expected = quote! {};

    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}

#[test]
fn push_down_argument() {
    let top: Top = parse_quote! {
        #[derive(Bpaf)]
        #[bpaf(command)]
        enum Options {
            #[bpaf(short)]
            Alpha(String),
            Beta,
        }
    };

    let expected = quote! {};

    assert_eq!(top.to_token_stream().to_string(), expected.to_string());
}
*/
