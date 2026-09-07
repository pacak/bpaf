use crate::derive_bpaf;
use pretty_assertions as _;
use pretty_assertions::assert_eq;
use quote::{ToTokens, quote};

#[track_caller]
fn res(input: proc_macro2::TokenStream, expected: proc_macro2::TokenStream) {
    let out = derive_bpaf(input).unwrap();
    assert_eq!(
        out.to_token_stream().to_string(),
        expected.to_token_stream().to_string()
    );
}

#[track_caller]
fn fail(input: proc_macro2::TokenStream) {
    assert!(derive_bpaf(input).is_err());
}

#[test]
fn cargo_command_helper() {
    let input = quote! {
        #[bpaf(options("asm"))]
        struct Opts {
            verbose: bool
        }
    };

    let expected = quote! {
        #[doc(hidden)]
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
    res(input, expected);
}

#[test]
fn fallback_usage_top() {
    let input = quote! {
        #[bpaf(options, fallback_to_usage)]
        struct Opts {
            verbose: bool
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opts() -> ::bpaf::OptionParser<Opts> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let verbose = ::bpaf::long("verbose").switch();
                ::bpaf::construct!(Opts { verbose, })
            }
            .to_options()
            .fallback_to_usage()
        }
    };
    res(input, expected);
}

#[test]
fn top_struct_options1() {
    let input = quote! {
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
        #[doc(hidden)]
        fn opt() -> ::bpaf::OptionParser<Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(Opt {})
                .to_options()
                .descr("those are options")
                .header(h)
                .footer(f)
        }
    };

    res(input, expected);
}

#[test]
fn struct_options2() {
    let input = quote! {
        #[bpaf(options)]
        /// those are options
        struct Opt {}
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> ::bpaf::OptionParser<Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(Opt {})
                .to_options()
                .descr("those are options")
        }
    };

    res(input, expected);
}

#[test]
fn options_with_custom_usage() {
    let input = quote! {
        #[bpaf(options, usage("App: usage"))]
        struct Opt {}
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> ::bpaf::OptionParser<Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(Opt {})
                .to_options()
                .usage("App: usage")
        }
    };

    res(input, expected);
}

#[test]
fn top_struct_construct() {
    let input = quote! {
        struct Opt { verbose: bool }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let verbose = ::bpaf::long("verbose").switch();
                ::bpaf::construct!(Opt { verbose, })
            }
        }
    };

    res(input, expected);
}

#[test]
fn top_enum_construct() {
    let input = quote! {
        enum Opt { Foo { verbose_name: bool }}
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let verbose_name = ::bpaf::long("verbose-name").switch();
                ::bpaf::construct!(Opt::Foo { verbose_name, })
            }
        }
    };

    res(input, expected);
}

#[test]
fn named_to_positional_with_metavar() {
    let input = quote! {
        struct Options {
            #[bpaf(positional("PATH"))]
            path: PathBuf,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn options() -> impl ::bpaf::Parser<Output=Options> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let path = ::bpaf::positional("PATH");
                ::bpaf::construct!(Options { path, })
            }
        }
    };

    res(input, expected);
}

#[test]
fn named_to_positional_without_metavar() {
    let input = quote! {
        struct Options {
            #[bpaf(positional)]
            path: PathBuf,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn options() -> impl ::bpaf::Parser<Output=Options> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let path = ::bpaf::positional("ARG");
                ::bpaf::construct!(Options { path, })
            }
        }
    };

    res(input, expected);
}

#[test]
fn private_visibility() {
    let input = quote! {
        #[bpaf(private)]
        pub struct Options {
            path: PathBuf,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn options() -> impl ::bpaf::Parser<Output=Options> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let path = ::bpaf::long("path").argument("ARG");
                ::bpaf::construct!(Options { path, })
            }
        }
    };

    res(input, expected);
}

#[test]
fn no_fields_declaration() {
    let input = quote! {
        struct Opts {}
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opts() -> impl ::bpaf::Parser<Output=Opts> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(Opts {})
        }
    };

    res(input, expected);
}

#[test]
fn req_flag_struct() {
    let input = quote! {
        struct Foo;
    };

    let expected = quote! {
        #[doc(hidden)]
        fn foo() -> impl ::bpaf::Parser<Output=Foo> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(Foo)
        }
    };

    res(input, expected);
}

#[test]
fn generate_options() {
    let input = quote! {
        #[bpaf(options, generate(oof))]
        struct Foo;
    };

    let expected = quote! {
        #[doc(hidden)]
        fn oof() -> ::bpaf::OptionParser<Foo> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(Foo).to_options()
        }
    };

    res(input, expected);
}

#[test]
fn generate_parser() {
    let input = quote! {
        #[bpaf(generate(oof))]
        struct Foo;
    };

    let expected = quote! {
        #[doc(hidden)]
        fn oof() -> impl ::bpaf::Parser<Output=Foo> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(Foo)
        }
    };

    res(input, expected);
}

#[test]
fn unnamed_struct() {
    let input = quote! {
        #[bpaf(options)]
        struct Opt(
            /// help
            PathBuf
        );
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> ::bpaf::OptionParser<Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let f0 = ::bpaf::positional("ARG").help("help");
                ::bpaf::construct!(Opt(f0,))
            }
            .to_options()
        }
    };

    res(input, expected);
}

#[test]
fn unnamed_enum() {
    let input = quote! {
        #[bpaf(options, version)]
        enum Opt1 {
            Con1(PathBuf, usize)
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt1() -> ::bpaf::OptionParser<Opt1> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let f0 = ::bpaf::positional("ARG");
                let f1 = ::bpaf::positional("ARG");
                ::bpaf::construct!(Opt1::Con1(f0, f1,))
            }
            .to_options()
            .version(env!("CARGO_PKG_VERSION"))
        }
    };

    res(input, expected);
}

#[test]
fn version_with_commands() {
    let input = quote! {
        #[bpaf(options, version)]
        enum Action {
            Alpha,
            Beta,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
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

    res(input, expected);
}

#[test]
fn version_with_commands_with_cargo_helper() {
    let input = quote! {
        #[bpaf(options("subcargo"), version)]
        enum Action {
            #[bpaf(command)]
            Alpha,
            #[bpaf(command)]
            Beta,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
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

    res(input, expected);
}

#[test]
fn help_generation() {
    let input = quote! {
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
        #[doc(hidden)]
        fn opt() -> ::bpaf::OptionParser<Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let f0 = ::bpaf::positional("ARG");
                ::bpaf::construct!(Opt(f0, ))
            }
            .to_options()
            .descr("descr\n  a")
            .footer("footer\n a")
        }
    };

    res(input, expected);
}

#[test]
fn max_width() {
    let input = quote! {
        #[bpaf(options, max_width(110))]
        struct Opt {
            verbose: bool,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> ::bpaf::OptionParser<Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let verbose = ::bpaf::long("verbose").switch();
                ::bpaf::construct!(Opt { verbose, })
            }
            .to_options()
            .max_width(110)
        }
    };

    res(input, expected);
}

#[test]
fn enum_to_flag_and_switches() {
    let input = quote! {
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
        #[doc(hidden)]
        pub fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = ::bpaf::long("Foo").long("fo").req_flag(Opt::Foo);
                let alt1 = ::bpaf::short('p').req_flag(Opt::Pff);
                let alt2 = ::bpaf::long("bar-foo").req_flag(Opt::BarFoo);
                let alt3 = {
                    let f0 = ::bpaf::long("bazz").argument("ARG");
                    ::bpaf::construct!(Opt::Baz(f0, ))
                };
                let alt4 = {
                    let strange = ::bpaf::long("strange").argument("ARG");
                    ::bpaf::construct!(Opt::Strange { strange, })
                };
                let alt5 = ::bpaf::pure(Opt::Alpha).to_options().usage("custom").command("alpha");
                let alt6 = ::bpaf::pure(Opt::Omega).to_options().command("omega");
                ::bpaf::construct!([alt0, alt1, alt2, alt3, alt4, alt5, alt6, ])
            }
        }
    };

    res(input, expected);
}

#[test]
fn fallback_for_options() {
    let input = quote! {
        #[bpaf(options, fallback(Opts::Dummy))]
        enum Opts {
            Llvm,
            Att,
            Dummy,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
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

    res(input, expected);
}

#[test]
fn implicitly_named_switch() {
    let input = quote! {
        #[bpaf(options, fallback(Opts::Dummy),)]
        struct Opts (#[bpaf(long("release"), switch,)] bool);
    };

    let expected = quote! {
        #[doc(hidden)]
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

    res(input, expected);
}

#[test]
fn explicit_external() {
    let input = quote! {
        #[bpaf(options)]
        struct Options {
            #[bpaf(external(actions), fallback(Action::List))]
            action: Action,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
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

    res(input, expected);
}

#[test]
fn implicit_external() {
    let input = quote! {
        #[bpaf(options)]
        struct Options {
            #[bpaf(external, fallback(Action::List))]
            action: Action,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
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

    res(input, expected);
}

#[test]
fn external_unit_variant() {
    let input = quote! {
        enum Opt {
            #[bpaf(external(parse_foo))]
            Foo,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            parse_foo()
        }
    };

    res(input, expected);
}

#[test]
fn external_fielded_variant() {
    let input = quote! {
        enum Opt {
            #[bpaf(external(parse_foo))]
            Foo { a: bool },
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            parse_foo()
        }
    };

    res(input, expected);
}

#[test]
fn external_fielded_variant_implicit_name() {
    let input = quote! {
        enum Opt {
            #[bpaf(external)]
            Foo { a: bool },
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            foo()
        }
    };

    res(input, expected);
}

#[test]
fn or_else_top_enum() {
    let input = quote! {
        #[bpaf(or_else(failed))]
        enum Opt { Foo, Bar }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = ::bpaf::long("foo").req_flag(Opt::Foo);
                let alt1 = ::bpaf::long("bar").req_flag(Opt::Bar);
                ::bpaf::construct!([alt0, alt1,])
            }
            .or_else(failed())
        }
    };

    res(input, expected);
}

#[test]
fn fallback_for_enum() {
    let input = quote! {
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
        #[doc(hidden)]
        fn decision() -> impl ::bpaf::Parser<Output=Decision> {
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

    res(input, expected);
}

#[test]
fn fallback_for_struct() {
    let input = quote! {
        #[bpaf(fallback(Value { count: 10 }))]
        struct Value {
            count: usize,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn value() -> impl ::bpaf::Parser<Output=Value> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let count = ::bpaf::long("count").argument("ARG");
                ::bpaf::construct!(Value { count, })
            }
            .fallback(Value { count: 10 })
        }
    };

    res(input, expected);
}

#[test]
fn fallback_str_for_struct() {
    let input = quote! {
        #[bpaf(fallback_str("10"))]
        struct Value {
            count: usize,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn value() -> impl ::bpaf::Parser<Output=Value> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let count = ::bpaf::long("count").argument("ARG");
                ::bpaf::construct!(Value { count, })
            }
            .fallback_str("10")
        }
    };

    res(input, expected);
}

#[test]
fn box_for_struct() {
    let input = quote! {
        #[bpaf(boxed)]
        struct Opts {
            a: String,
            b: String,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opts() -> impl ::bpaf::Parser<Output=Opts> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let a = ::bpaf::short('a').argument("ARG");
                let b = ::bpaf::short('b').argument("ARG");
                ::bpaf::construct!(Opts { a, b, })
            }
            .into_box()
        }
    };

    res(input, expected);
}

#[test]
fn ingore_doc_comment_enum() {
    let input = quote! {
        #[bpaf(ignore_rustdoc)]
        /// present
        enum Mode {
            /// intel help
            Intel,
            /// att help
            Att,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn mode() -> impl ::bpaf::Parser<Output=Mode> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = ::bpaf::long("intel").req_flag(Mode::Intel).help("intel help");
                let alt1 = ::bpaf::long("att").req_flag(Mode::Att).help("att help");
                ::bpaf::construct!([alt0, alt1, ])
            }
        }
    };

    res(input, expected);
}

#[test]
fn ingore_doc_comment_top_level_2() {
    let input = quote! {
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
        #[doc(hidden)]
        fn mode() -> ::bpaf::OptionParser<Mode> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = ::bpaf::long("intel").req_flag(Mode::Intel).help("intel help");
                let alt1 = ::bpaf::long("att").req_flag(Mode::Att).help("att help");
                ::bpaf::construct!([alt0, alt1,])
            }
            .to_options()
        }
    };

    res(input, expected);
}

#[test]
fn top_comment_is_group_help_enum() {
    let input = quote! {
        /// present
        enum Mode {
            /// intel help
            Intel,
            /// att help
            Att,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn mode() -> impl ::bpaf::Parser<Output=Mode> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = ::bpaf::long("intel").req_flag(Mode::Intel).help("intel help");
                let alt1 = ::bpaf::long("att").req_flag(Mode::Att).help("att help");
                ::bpaf::construct!([alt0, alt1, ])
            }
            .group_help("present")
        }
    };

    res(input, expected);
}

#[test]
fn top_comment_is_group_help_struct() {
    let input = quote! {
        /// present
        struct Mode {
            /// help
            intel: bool,
            /// help
            att: bool,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn mode() -> impl ::bpaf::Parser<Output=Mode> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let intel = ::bpaf::long("intel").switch().help("help");
                let att = ::bpaf::long("att").switch().help("help");
                ::bpaf::construct!(Mode { intel, att, })
            }
            .group_help("present")
        }
    };

    res(input, expected);
}

#[test]
fn hidden_command() {
    let input = quote! {
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
        #[doc(hidden)]
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

    res(input, expected);
}

#[test]
fn enum_markdownish() {
    let input = quote! {
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
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output = Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let field = ::bpaf::long("field").switch();
                ::bpaf::construct!(Opt::Opt { field ,})
            }
            .to_options()
            .descr("Make a tree")
            .header("x")
            .footer("Examples:\n\n```sh\ncargo 1\ncargo 2\n```")
            .command("opt")
        }
    };

    res(input, expected);
}

#[test]
fn fallback_usage_lut_1() {
    let input = quote! {
        #[bpaf(options)]
        enum Lutgen {
            /// descr
            #[bpaf(command, fallback_to_usage)]
            Generate,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn lutgen() -> ::bpaf::OptionParser<Lutgen> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(Lutgen::Generate)
                .to_options()
                .descr("descr")
                .fallback_to_usage()
                .command("generate")
                .to_options()
        }
    };

    res(input, expected);
}

#[test]
fn hidden_default_enum_singleton() {
    let input = quote! {
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
        #[doc(hidden)]
        fn decision() -> impl ::bpaf::Parser<Output=Decision> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = ::bpaf::long("YES").req_flag(Decision::Yes).help("HALP");
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

    res(input, expected);
}

#[test]
fn default_req_flag() {
    let input = quote! {
        #[bpaf(options)]
        enum Mode {
            #[bpaf(short('a'), default)]
            A,
            #[bpaf(short('b'))]
            B,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn mode() -> ::bpaf::OptionParser<Mode> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = ::bpaf::short('a').req_flag(Mode::A).default();
                let alt1 = ::bpaf::short('b').req_flag(Mode::B);
                ::bpaf::construct!([alt0, alt1,])
            }
            .to_options()
        }
    };

    res(input, expected);
}

#[test]
fn enum_short_variant_name() {
    let input = quote! {
        enum Opt {
            A,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::short('a').req_flag(Opt::A)
        }
    };

    res(input, expected);
}

#[test]
fn unknown_option_annotation() {
    fail(quote! {
        #[bpaf(options, such_attribute)]
        struct Foo {}
    });
}

#[test]
fn single_unit_command() {
    let input = quote! {
        #[bpaf(command)]
        struct One;
    };

    let expected = quote! {
        #[doc(hidden)]
        fn one() -> impl ::bpaf::Parser<Output=One> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(One).to_options().command("one")
        }
    };

    res(input, expected);
}

#[test]
fn struct_command_no_decor() {
    let input = quote! {
        /// those are options
        #[bpaf(command)]
        struct Opt;
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(Opt)
                .to_options()
                .descr("those are options")
                .command("opt")
        }
    };

    res(input, expected);
}

#[test]
fn struct_command_decor() {
    let input = quote! {
        /// those are options
        #[bpaf(command, descr(descr), header(header), footer(footer), help(help))]
        struct Opt;
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
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

    res(input, expected);
}

#[test]
fn struct_command_short() {
    let input = quote! {
        /// those are options
        #[bpaf(command, short('x'))]
        struct O{ }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn o() -> impl ::bpaf::Parser<Output=O> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(O{ })
                .to_options()
                .descr("those are options")
                .command("o")
                .short('x')
        }
    };

    res(input, expected);
}

#[test]
fn command_with_aliases_struct() {
    let input = quote! {
        #[bpaf(command, short('c'), long("long"), long("long2"))]
        /// help
        struct Command {
            i: bool,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn command() -> impl ::bpaf::Parser<Output=Command> {
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

    res(input, expected);
}

#[test]
fn or_else_top_command() {
    let input = quote! {
        #[bpaf(command, or_else(failed))]
        struct Opt;
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow (unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(Opt)
                .to_options()
                .command("opt")
                .or_else(failed())
        }
    };

    res(input, expected);
}

#[test]
fn command_alias_order_preserved() {
    let input = quote! {
        #[bpaf(command, long("l"), short('s'))]
        struct Opt;
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(Opt)
                .to_options()
                .command("opt")
                .long("l")
                .short('s')
        }
    };

    res(input, expected);
}

#[test]
fn fallback_usage_subcommand() {
    let input = quote! {
        /// those are options
        #[bpaf(command, fallback_to_usage)]
        struct Opt (PathBuf);
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output = Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let f0 = ::bpaf::positional("ARG");
                ::bpaf::construct!(Opt(f0,))
            }
            .to_options()
            .descr("those are options")
            .fallback_to_usage()
            .command("opt")
        }
    };

    res(input, expected);
}

#[test]
fn generate_command() {
    let input = quote! {
        #[bpaf(command, generate(oof))]
        struct Foo;
    };

    let expected = quote! {
        #[doc(hidden)]
        fn oof() -> impl ::bpaf::Parser<Output=Foo> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(Foo).to_options().command("foo")
        }
    };

    res(input, expected);
}

#[test]
fn command_with_boxed() {
    let input = quote! {
        #[bpaf(command, boxed)]
        struct Opts {
            a: String,
            b: String,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opts() -> impl ::bpaf::Parser<Output=Opts> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let a = ::bpaf::short('a').argument("ARG");
                let b = ::bpaf::short('b').argument("ARG");
                ::bpaf::construct!(Opts { a, b, })
            }
            .to_options()
            .command("opts")
            .into_box()
        }
    };

    res(input, expected);
}

#[test]
fn struct_nest() {
    let input = quote! {
        #[bpaf(nest, long("tag"))]
        struct Fooo {
            name: String,
            verbose: bool,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn fooo() -> impl ::bpaf::Parser<Output = Fooo> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::long("tag").nest({
                let name = ::bpaf::long("name").argument("ARG");
                let verbose = ::bpaf::long("verbose").switch();
                ::bpaf::construct!(Fooo { name, verbose, })
            })
        }
    };

    res(input, expected);
}

#[test]
fn struct_nest_implicit_name() {
    let input = quote! {
        #[bpaf(nest)]
        struct Fooo {
            name: String,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn fooo() -> impl ::bpaf::Parser<Output = Fooo> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::long("fooo").nest({
                let name = ::bpaf::long("name").argument("ARG");
                ::bpaf::construct!(Fooo { name, })
            })
        }
    };

    res(input, expected);
}

#[test]
fn struct_nest_with_help() {
    let input = quote! {
        #[bpaf(nest, long("tag"), help("custom help"))]
        struct Fooo { }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn fooo() -> impl ::bpaf::Parser<Output = Fooo> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::long("tag").help("custom help").nest(::bpaf::pure(Fooo {}))
        }
    };

    res(input, expected);
}

#[test]
fn struct_nest_doc_help() {
    let input = quote! {
        /// My struct
        #[bpaf(nest, long("tag"))]
        struct Fooo { }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn fooo() -> impl ::bpaf::Parser<Output = Fooo> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::long("tag").help("My struct").nest(::bpaf::pure(Fooo {}))
        }
    };

    res(input, expected);
}

#[test]
fn struct_nest_empty_struct() {
    let input = quote! {
        #[bpaf(nest, long("x"))]
        struct Empty { }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn empty() -> impl ::bpaf::Parser<Output = Empty> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::long("x").nest(::bpaf::pure(Empty {}))
        }
    };

    res(input, expected);
}

#[test]
fn struct_nest_unit_struct() {
    let input = quote! {
        #[bpaf(nest, long("unit"))]
        struct Unit;
    };

    let expected = quote! {
        #[doc(hidden)]
        fn unit() -> impl ::bpaf::Parser<Output = Unit> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::long("unit").nest(::bpaf::pure(Unit))
        }
    };

    res(input, expected);
}

#[test]
fn struct_nest_tuple_struct() {
    let input = quote! {
        #[bpaf(nest, long("pair"))]
        struct Pair(#[bpaf(positional("A"))] String, #[bpaf(positional("B"))] u32);
    };

    let expected = quote! {
        #[doc(hidden)]
        fn pair() -> impl ::bpaf::Parser<Output = Pair> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::long("pair").nest({
                let f0 = ::bpaf::positional("A");
                let f1 = ::bpaf::positional("B");
                ::bpaf::construct!(Pair(f0, f1, ))
            })
        }
    };

    res(input, expected);
}

#[test]
fn struct_nest_short() {
    let input = quote! {
        #[bpaf(nest, short('t'), long("tag"))]
        struct Tagged;
    };

    let expected = quote! {
        #[doc(hidden)]
        fn tagged() -> impl ::bpaf::Parser<Output = Tagged> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::short('t').long("tag").nest(::bpaf::pure(Tagged))
        }
    };

    res(input, expected);
}

#[test]
fn struct_nest_multiple_names() {
    let input = quote! {
        #[bpaf(nest, short('a'), short('b'), long("x"), long("y"))]
        struct Multi(#[bpaf(positional("A"))] String);
    };

    let expected = quote! {
        #[doc(hidden)]
        fn multi() -> impl ::bpaf::Parser<Output = Multi> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::short('a').short('b').long("x").long("y").nest({
                let f0 = ::bpaf::positional("A");
                ::bpaf::construct!(Multi(f0, ))
            })
        }
    };

    res(input, expected);
}

#[test]
fn struct_nest_auto_long_with_doc() {
    let input = quote! {
        /// My auto struct
        #[bpaf(nest)]
        struct AutoStruct;
    };

    let expected = quote! {
        #[doc(hidden)]
        fn auto_struct() -> impl ::bpaf::Parser<Output = AutoStruct> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::long("auto-struct").help("My auto struct").nest(::bpaf::pure(AutoStruct))
        }
    };

    res(input, expected);
}

#[test]
fn struct_nest_with_boxed() {
    let input = quote! {
        #[bpaf(nest, long("tag"), boxed)]
        struct Fooo {
            verbose: bool,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn fooo() -> impl ::bpaf::Parser<Output = Fooo> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::long("tag").nest({
                let verbose = ::bpaf::long("verbose").switch();
                ::bpaf::construct!(Fooo { verbose, })
            })
            .into_box()
        }
    };

    res(input, expected);
}

#[test]
fn struct_nest_nested_long() {
    let input = quote! {
        #[bpaf(nest, long)]
        struct Named;
    };

    let expected = quote! {
        #[doc(hidden)]
        fn named() -> impl ::bpaf::Parser<Output = Named> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::long("named").nest(::bpaf::pure(Named))
        }
    };

    res(input, expected);
}

#[test]
fn struct_nest_nested_short() {
    let input = quote! {
        #[bpaf(nest, short)]
        struct Named;
    };

    let expected = quote! {
        #[doc(hidden)]
        fn named() -> impl ::bpaf::Parser<Output = Named> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::short('n').nest(::bpaf::pure(Named))
        }
    };

    res(input, expected);
}

#[test]
fn struct_literal() {
    let input = quote! {
        #[bpaf(literal("tag"))]
        struct Fooo {
            name: String,
            verbose: bool,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn fooo() -> impl ::bpaf::Parser<Output = Fooo> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::literal("tag").nest({
                let name = ::bpaf::long("name").argument("ARG");
                let verbose = ::bpaf::long("verbose").switch();
                ::bpaf::construct!(Fooo { name, verbose, })
            })
        }
    };

    res(input, expected);
}

#[test]
fn struct_literal_implicit_name() {
    let input = quote! {
        #[bpaf(literal)]
        struct Fooo {
            name: String,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn fooo() -> impl ::bpaf::Parser<Output = Fooo> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::literal("fooo").nest({
                let name = ::bpaf::long("name").argument("ARG");
                ::bpaf::construct!(Fooo { name, })
            })
        }
    };

    res(input, expected);
}

#[test]
fn struct_literal_with_help() {
    let input = quote! {
        #[bpaf(literal("tag"), help("custom help"))]
        struct Fooo { }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn fooo() -> impl ::bpaf::Parser<Output = Fooo> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::literal("tag").help("custom help").nest(::bpaf::pure(Fooo {}))
        }
    };

    res(input, expected);
}

#[test]
fn struct_literal_doc_help() {
    let input = quote! {
        /// My struct
        #[bpaf(literal("tag"))]
        struct Fooo { }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn fooo() -> impl ::bpaf::Parser<Output = Fooo> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::literal("tag").help("My struct").nest(::bpaf::pure(Fooo {}))
        }
    };

    res(input, expected);
}

#[test]
fn struct_literal_unit_struct() {
    let input = quote! {
        #[bpaf(literal("unit"))]
        struct Unit;
    };

    let expected = quote! {
        #[doc(hidden)]
        fn unit() -> impl ::bpaf::Parser<Output = Unit> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::literal("unit").nest(::bpaf::pure(Unit))
        }
    };

    res(input, expected);
}

#[test]
fn struct_literal_multiple_names() {
    let input = quote! {
        #[bpaf(literal("x"), short('a'), long("y"))]
        struct Multi(#[bpaf(positional("A"))] String);
    };

    let expected = quote! {
        #[doc(hidden)]
        fn multi() -> impl ::bpaf::Parser<Output = Multi> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::literal("x").short('a').long("y").nest({
                let f0 = ::bpaf::positional("A");
                ::bpaf::construct!(Multi(f0, ))
            })
        }
    };

    res(input, expected);
}

#[test]
fn struct_literal_nested_long() {
    let input = quote! {
        #[bpaf(literal, long)]
        struct Named;
    };

    let expected = quote! {
        #[doc(hidden)]
        fn named() -> impl ::bpaf::Parser<Output = Named> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::literal("named").long("named").nest(::bpaf::pure(Named))
        }
    };

    res(input, expected);
}

#[test]
fn enum_nest_named() {
    let input = quote! {
        enum Opt {
            /// Add a file
            #[bpaf(nest, short('a'), long("add"))]
            Add {
                #[bpaf(positional("FILE"))]
                files: PathBuf,
            }
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::short('a').long("add").help("Add a file").nest({
                let files = ::bpaf::positional("FILE");
                ::bpaf::construct!(Opt::Add { files, })
            })
        }
    };

    res(input, expected);
}

#[test]
fn enum_nest_named_implicit_name() {
    let input = quote! {
        enum Opt {
            /// Add a file
            #[bpaf(nest)]
            Add {
                #[bpaf(positional("FILE"))]
                files: PathBuf,
            }
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::long("add").help("Add a file").nest({
                let files = ::bpaf::positional("FILE");
                ::bpaf::construct!(Opt::Add { files, })
            })
        }
    };

    res(input, expected);
}

#[test]
fn enum_nest_unnamed() {
    let input = quote! {
        enum Opt {
            /// Add a file
            #[bpaf(nest, short('a'), long("add"))]
            Add(#[bpaf(positional("FILE"))] PathBuf),
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::short('a').long("add").help("Add a file").nest({
                let f0 = ::bpaf::positional("FILE");
                ::bpaf::construct!(Opt::Add(f0, ))
            })
        }
    };

    res(input, expected);
}

#[test]
fn enum_nest_unit() {
    let input = quote! {
        enum Opt {
            #[bpaf(nest, short('s'))]
            Status,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::short('s').nest(::bpaf::pure(Opt::Status))
        }
    };

    res(input, expected);
}

#[test]
fn nest_variant_untyped_postpr() {
    let input = quote! {
        enum Opt {
            /// help
            #[bpaf(nest, long("tag"), map(f))]
            Foo,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::long("tag")
                .help("help")
                .nest(::bpaf::pure(Opt::Foo))
                .map(f)
        }
    };

    res(input, expected);
}

#[test]
fn enum_nest_auto_short() {
    let input = quote! {
        enum Opt {
            #[bpaf(nest, short, long("add"))]
            Add {
                #[bpaf(positional("FILE"))]
                files: PathBuf,
            }
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::short('a').long("add").nest({
                let files = ::bpaf::positional("FILE");
                ::bpaf::construct!(Opt::Add { files, })
            })
        }
    };

    res(input, expected);
}

#[test]
fn enum_nest_auto_long() {
    let input = quote! {
        enum Opt {
            #[bpaf(nest, short('a'), long)]
            Add {
                #[bpaf(positional("FILE"))]
                files: PathBuf,
            }
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::short('a').long("add").nest({
                let files = ::bpaf::positional("FILE");
                ::bpaf::construct!(Opt::Add { files, })
            })
        }
    };

    res(input, expected);
}

#[test]
fn enum_nest_multiple_names() {
    let input = quote! {
        enum Opt {
            #[bpaf(nest, short('a'), short('b'), long("x"), long("y"))]
            Add(#[bpaf(positional("FILE"))] PathBuf),
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::short('a').short('b').long("x").long("y").nest({
                let f0 = ::bpaf::positional("FILE");
                ::bpaf::construct!(Opt::Add(f0, ))
            })
        }
    };

    res(input, expected);
}

#[test]
fn enum_nest_doc_help() {
    let input = quote! {
        enum Opt {
            /// Add a file
            #[bpaf(nest, short('a'))]
            Add(#[bpaf(positional("FILE"))] PathBuf),
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::short('a').help("Add a file").nest({
                let f0 = ::bpaf::positional("FILE");
                ::bpaf::construct!(Opt::Add(f0, ))
            })
        }
    };

    res(input, expected);
}

#[test]
fn enum_nest_with_hide() {
    let input = quote! {
        enum Opt {
            #[bpaf(nest, short('a'), hide)]
            Add(#[bpaf(positional("FILE"))] PathBuf),
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::short('a').nest({
                let f0 = ::bpaf::positional("FILE");
                ::bpaf::construct!(Opt::Add(f0, ))
            })
            .hide()
        }
    };

    res(input, expected);
}

#[test]
fn enum_nest_help_override() {
    let input = quote! {
        enum Opt {
            /// Doc help
            #[bpaf(nest, short('a'), help("Override"))]
            Add(#[bpaf(positional("FILE"))] PathBuf),
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::short('a').help("Override").nest({
                let f0 = ::bpaf::positional("FILE");
                ::bpaf::construct!(Opt::Add(f0, ))
            })
        }
    };

    res(input, expected);
}

#[test]
fn enum_literal_named() {
    let input = quote! {
        enum Opt {
            /// Add a file
            #[bpaf(literal("add"), short('a'), long("add"))]
            Add {
                #[bpaf(positional("FILE"))]
                files: PathBuf,
            }
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::literal("add").short('a').long("add").help("Add a file").nest({
                let files = ::bpaf::positional("FILE");
                ::bpaf::construct!(Opt::Add { files, })
            })
        }
    };

    res(input, expected);
}

#[test]
fn enum_literal_implicit_name() {
    let input = quote! {
        enum Opt {
            /// Add a file
            #[bpaf(literal)]
            Add {
                #[bpaf(positional("FILE"))]
                files: PathBuf,
            }
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::literal("add").help("Add a file").nest({
                let files = ::bpaf::positional("FILE");
                ::bpaf::construct!(Opt::Add { files, })
            })
        }
    };

    res(input, expected);
}

#[test]
fn enum_literal_unnamed() {
    let input = quote! {
        enum Opt {
            /// Add a file
            #[bpaf(literal("add"), short('a'))]
            Add(#[bpaf(positional("FILE"))] PathBuf),
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::literal("add").short('a').help("Add a file").nest({
                let f0 = ::bpaf::positional("FILE");
                ::bpaf::construct!(Opt::Add(f0, ))
            })
        }
    };

    res(input, expected);
}

#[test]
fn enum_literal_unit() {
    let input = quote! {
        enum Opt {
            #[bpaf(literal("status"), short('s'))]
            Status,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::literal("status").short('s').nest(::bpaf::pure(Opt::Status))
        }
    };

    res(input, expected);
}

#[test]
fn enum_literal_unit_with_help() {
    let input = quote! {
        enum Opt {
            /// Show the status
            #[bpaf(literal("status"), short('s'))]
            Status,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::literal("status")
                .short('s')
                .help("Show the status")
                .nest(::bpaf::pure(Opt::Status))
        }
    };

    res(input, expected);
}

#[test]
fn enum_literal_unit_untyped_postpr() {
    let input = quote! {
        enum Opt {
            #[bpaf(literal("tag"), map(f))]
            Foo,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::literal("tag").nest(::bpaf::pure(Opt::Foo)).map(f)
        }
    };

    res(input, expected);
}

#[test]
fn enum_literal_with_hide() {
    let input = quote! {
        enum Opt {
            #[bpaf(literal("add"), hide)]
            Add(#[bpaf(positional("FILE"))] PathBuf),
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::literal("add").nest({
                let f0 = ::bpaf::positional("FILE");
                ::bpaf::construct!(Opt::Add(f0, ))
            })
            .hide()
        }
    };

    res(input, expected);
}

#[test]
fn literal_variant_untyped_postpr() {
    let input = quote! {
        enum Opt {
            /// help
            #[bpaf(literal("tag"), map(f))]
            Foo { x: bool },
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::literal("tag")
                .help("help")
                .nest({
                    let x = ::bpaf::short('x').switch();
                    ::bpaf::construct!(Opt::Foo { x, })
                })
                .map(f)
        }
    };

    res(input, expected);
}

#[test]
fn explicit_construct() {
    let input = quote! {
        #[bpaf(construct)]
        struct Opt { verbose: bool }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let verbose = ::bpaf::long("verbose").switch();
                ::bpaf::construct!(Opt { verbose, })
            }
        }
    };

    res(input, expected);
}

#[test]
fn explicit_construct_with_postpr() {
    let input = quote! {
        #[bpaf(construct, fallback(Opt::Dummy))]
        enum Opt { A, Dummy }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = ::bpaf::short('a').req_flag(Opt::A);
                let alt1 = ::bpaf::long("dummy").req_flag(Opt::Dummy);
                ::bpaf::construct!([alt0, alt1,])
            }
            .fallback(Opt::Dummy)
        }
    };

    res(input, expected);
}

#[test]
fn construct_variant_group_help() {
    let input = quote! {
        enum Opt {
            #[bpaf(group_help("g"), hide)]
            Foo { a: bool },
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let a = ::bpaf::short('a').switch();
                ::bpaf::construct!(Opt::Foo { a, })
            }
            .group_help("g")
            .hide()
        }
    };

    res(input, expected);
}

#[test]
fn construct_variant_untyped_postpr() {
    let input = quote! {
        enum Opt {
            /// help
            #[bpaf(map(f))]
            Foo { a: bool },
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let a = ::bpaf::short('a').switch();
                ::bpaf::construct!(Opt::Foo { a, })
            }
            .group_help("help")
            .map(f)
        }
    };

    res(input, expected);
}

#[test]
fn construct_variant_generate_ignored() {
    let input = quote! {
        enum Opt {
            #[bpaf(generate(other))]
            Foo { a: bool },
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let a = ::bpaf::short('a').switch();
                ::bpaf::construct!(Opt::Foo { a, })
            }
        }
    };

    res(input, expected);
}

#[test]
fn construct_variant_help_rejected() {
    fail(quote! {
        enum Opt {
            #[bpaf(help("h"))]
            Foo { a: bool },
        }
    });
}

#[test]
fn top_construct_group_help_before_postpr() {
    let input = quote! {
        /// present
        #[bpaf(fallback(Opt::Dummy))]
        enum Opt { A, Dummy }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let alt0 = ::bpaf::short('a').req_flag(Opt::A);
                let alt1 = ::bpaf::long("dummy").req_flag(Opt::Dummy);
                ::bpaf::construct!([alt0, alt1,])
            }
            .group_help("present")
            .fallback(Opt::Dummy)
        }
    };

    res(input, expected);
}

#[test]
fn command_variant_alias_names() {
    let input = quote! {
        enum Options {
            #[bpaf(command("command"), short('c'), long("long"), long("long2"))]
            /// help
            Command {
                i: bool,
            }
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn options() -> impl ::bpaf::Parser<Output=Options> {
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

    res(input, expected);
}

#[test]
fn command_variant_untyped_postpr() {
    let input = quote! {
        enum Opt {
            /// help
            #[bpaf(command, map(f))]
            Foo,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(Opt::Foo)
                .to_options()
                .descr("help")
                .command("foo")
                .map(f)
        }
    };

    res(input, expected);
}

#[test]
fn command_fielded_variant_untyped_postpr() {
    let input = quote! {
        enum Opt {
            /// help
            #[bpaf(command, map(f))]
            Foo { a: bool },
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output=Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let a = ::bpaf::short('a').switch();
                ::bpaf::construct!(Opt::Foo { a, })
            }
            .to_options()
            .descr("help")
            .command("foo")
            .map(f)
        }
    };

    res(input, expected);
}

#[test]
fn top_map_requires_turbofish_and_changes_type() {
    let input = quote! {
        #[bpaf(options, map::<String>(|x| x.to_string()))]
        struct Opt { val: u32 }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> ::bpaf::OptionParser<String> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let val = ::bpaf::long("val").argument("ARG");
                ::bpaf::construct!(Opt { val, })
            }
            .map::<_, String>(|x| x.to_string())
            .to_options()
        }
    };

    res(input, expected);
}

#[test]
fn top_parse_turbofish_construct() {
    let input = quote! {
        #[bpaf(parse::<u32>(parse_num))]
        struct Opt { val: u32 }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output = u32> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let val = ::bpaf::long("val").argument("ARG");
                ::bpaf::construct!(Opt { val, })
            }
            .parse::<_, u32, _>(parse_num)
        }
    };

    res(input, expected);
}

#[test]
fn top_map_command() {
    let input = quote! {
        #[bpaf(command, map::<String>(f))]
        struct Opt;
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> impl ::bpaf::Parser<Output = String> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(Opt)
                .to_options()
                .command("opt")
                .map::<_, String>(f)
        }
    };

    res(input, expected);
}

#[test]
fn top_map_nest() {
    let input = quote! {
        #[bpaf(nest, long("tag"), map::<bool>(f))]
        struct Fooo { }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn fooo() -> impl ::bpaf::Parser<Output = bool> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::long("tag").nest(::bpaf::pure(Fooo {})).map::<_, bool>(f)
        }
    };

    res(input, expected);
}

#[test]
fn top_map_nest_without_turbofish_fails() {
    fail(quote! {
        #[bpaf(nest, long("tag"), map(f))]
        struct Fooo { }
    });
}

#[test]
fn top_map_literal() {
    let input = quote! {
        #[bpaf(literal("tag"), map::<bool>(f))]
        struct Fooo { }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn fooo() -> impl ::bpaf::Parser<Output = bool> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::literal("tag").nest(::bpaf::pure(Fooo {})).map::<_, bool>(f)
        }
    };

    res(input, expected);
}

#[test]
fn top_map_literal_without_turbofish_fails() {
    fail(quote! {
        #[bpaf(literal("tag"), map(f))]
        struct Fooo { }
    });
}

#[test]
fn top_guard_then_map_ordering() {
    let input = quote! {
        #[bpaf(options, guard(check, "msg"), map::<String>(m), fallback(1))]
        struct Opt {}
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> ::bpaf::OptionParser<String> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(Opt {})
                .guard(check, "msg")
                .map::<_, String>(m)
                .fallback(1)
                .to_options()
        }
    };

    res(input, expected);
}

#[test]
fn top_fallback_family() {
    let input = quote! {
        #[bpaf(options, fallback(1), display_fallback, hide, custom_usage(usage()))]
        struct Opt {}
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> ::bpaf::OptionParser<Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(Opt {})
                .fallback(1)
                .display_fallback()
                .hide()
                .custom_usage(usage())
                .to_options()
        }
    };

    res(input, expected);
}

#[test]
fn top_map_without_turbofish_fails() {
    fail(quote! {
        #[bpaf(options, map(double))]
        struct Opt {};
    });
}

#[test]
fn top_parse_without_turbofish_fails() {
    fail(quote! {
        #[bpaf(options, parse(twice))]
        struct Opt {};
    });
}

#[test]
fn top_map_command_without_turbofish_fails() {
    fail(quote! {
        #[bpaf(command, map(double))]
        struct Opt {};
    });
}

#[test]
fn construct_rejects_options_only_op() {
    fail(quote! {
        #[bpaf(construct, descr("nope"))]
        struct Opt {}
    });
}

#[test]
fn unit_enum_makes_req() {
    let input = quote! {
        enum Foo { Foo }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn foo() -> impl ::bpaf::Parser<Output=Foo> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::long("foo").req_flag(Foo::Foo)
        }
    };

    res(input, expected);
}

#[test]
fn fallback_usage_lut_2() {
    let input = quote! {
        #[bpaf(options)]
        enum Lutgen {
            /// descr
            #[bpaf(command, fallback_to_usage)]
            Generate {},
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn lutgen() -> ::bpaf::OptionParser<Lutgen> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            ::bpaf::pure(Lutgen::Generate {})
                .to_options()
                .descr("descr")
                .fallback_to_usage()
                .command("generate")
                .to_options()
        }
    };

    res(input, expected);
}

#[test]
fn unnamed_command_enum() {
    let input = quote! {
        #[bpaf(command)]
        enum Opts {
            #[bpaf(command("alpha1"))]
            Alpha,
            Beta,
            Gamma,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opts() -> impl ::bpaf::Parser<Output=Opts> {
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

    res(input, expected);
}

#[test]
fn help_parser() {
    let input = quote! {
        #[bpaf(options, help_parser(help::short_long()))]
        struct Opt {
            verbose: bool,
        }
    };

    let expected = quote! {
        #[doc(hidden)]
        fn opt() -> ::bpaf::OptionParser<Opt> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            {
                let verbose = ::bpaf::long("verbose").switch();
                ::bpaf::construct!(Opt { verbose, })
            }
            .to_options()
            .help_parser(help::short_long())
        }
    };

    res(input, expected);
}
