use crate::fields::{parse_named, parse_unnamed};
use pretty_assertions::assert_eq;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Result, parse,
    parse::{Parse, Parser},
    parse_quote, parse2,
};

#[derive(Debug)]
struct UnnamedField {
    parser: TokenStream,
}

impl Parse for UnnamedField {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        Ok(Self {
            parser: parse_unnamed(input)?,
        })
    }
}

impl ToTokens for UnnamedField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.parser.to_tokens(tokens)
    }
}

#[derive(Debug)]
struct NamedField {
    parser: TokenStream,
}

impl Parse for NamedField {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let (_, parser) = parse_named(input)?;
        Ok(Self { parser })
    }
}

impl ToTokens for NamedField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.parser.to_tokens(tokens)
    }
}

#[test]
fn implicit_parser() {
    let input: NamedField = parse_quote! {
        /// help
        number: usize
    };
    let output = quote! {
        ::bpaf::long("number").argument("ARG").help("help")
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn implicit_parser_custom_help() {
    let input: NamedField = parse_quote! {
        /// help
        #[bpaf(help(custom_help))]
        number: usize
    };
    let output = quote! {
        ::bpaf::long("number").argument("ARG").help(custom_help)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn short_long() {
    let input: NamedField = parse_quote! {
        #[bpaf(short, long)]
        number: usize
    };
    let output = quote! {
        ::bpaf::short('n').long("number").argument("ARG")
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn derive_fallback() {
    let input: NamedField = parse_quote! {
        #[bpaf(fallback(3.1415))]
        number: f64
    };
    let output = quote! {
        ::bpaf::long("number").argument("ARG").fallback(3.1415)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn derive_fallback_display() {
    let input: NamedField = parse_quote! {
        #[bpaf(fallback(3.1415), display_fallback)]
        number: f64
    };
    let output = quote! {
        ::bpaf::long("number").argument("ARG").fallback(3.1415).display_fallback()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn adjacent_argument() {
    let input: NamedField = parse_quote! {
        #[bpaf(argument, adjacent)]
        number: f64
    };
    let output = quote! {
        ::bpaf::long("number").argument("ARG").adjacent()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn derive_fallback_with() {
    let input: NamedField = parse_quote! {
        #[bpaf(fallback_with(external))]
        number: f64
    };
    let output = quote! {
        ::bpaf::long("number").argument("ARG").fallback_with(external)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn derive_fallback_str() {
    let input: NamedField = parse_quote! {
        #[bpaf(fallback_str("42"))]
        number: f64
    };
    let output = quote! {
        ::bpaf::long("number").argument("ARG").fallback_str("42")
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn derive_external_help() {
    let input: NamedField = parse_quote! {
        /// help
        #[bpaf(external(level))]
        number: f64
    };
    let output = quote! {
        level()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn derive_external_no_help() {
    let input: NamedField = parse_quote! {
        #[bpaf(external(level))]
        number: f64
    };
    let output = quote! {
        level()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn derive_external_with_path() {
    let input: NamedField = parse_quote! {
        #[bpaf(external(path::level))]
        number: f64
    };
    let output = quote! {
        path::level()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn derive_external_nohelp() {
    let input: NamedField = parse_quote! {
        /// help
        #[bpaf(external(level))]
        number: f64
    };
    let output = quote! {
        level()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn derive_field_guard() {
    let input: NamedField = parse_quote! {
        #[bpaf(guard(positive, "msg"))]
        number: usize
    };
    let output = quote! {
        ::bpaf::long("number").argument("ARG").guard(positive, "msg")
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn derive_field_guard_const() {
    let input: NamedField = parse_quote! {
        #[bpaf(guard(positive, MSG))]
        number: usize
    };
    let output = quote! {
        ::bpaf::long("number").argument("ARG").guard(positive, MSG)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn derive_help() {
    let input: NamedField = parse_quote! {
        /// multi
        ///
        /// vis
        ///  hidden
        pub(crate) flag: bool
    };
    let output = quote! {
        ::bpaf::long("flag").switch().help("multi\n\nvis\n hidden")
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn retain_fish_for_arg() {
    let input: NamedField = parse_quote! {
        #[bpaf(argument::<String>("NAME"), map(transmogrify))]
        chicken: Chicken
    };
    let output = quote! {
        ::bpaf::long("chicken").argument::<String>("NAME").map(transmogrify)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn map_requires_explicit_parser() {
    let input: NamedField = parse_quote! {
        #[bpaf(argument::<usize>("NUM"), map(double))]
        number: usize
    };
    let output = quote! {
        ::bpaf::long("number").argument::<usize>("NUM").map(double)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn check_guard() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(guard(odd, "must be odd"))]
        usize
    };

    let output = quote! {
        ::bpaf::positional("ARG").guard(odd, "must be odd")
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn pure_value() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(pure(42))]
        /// Ignored
        usize
    };

    let output = quote! {
        ::bpaf::pure(42)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn pure_with_value() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(pure_with(detect_color))]
        /// Ignored
        usize
    };

    let output = quote! {
        ::bpaf::pure_with(detect_color)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn check_fallback() {
    let input: NamedField = parse_quote! {
        #[bpaf(argument("SPEED"), fallback(42.0))]
        speed: f64
    };
    let output = quote! {
        ::bpaf::long("speed").argument("SPEED").fallback(42.0)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn check_many_files_implicit() {
    let input: NamedField = parse_quote! {
        files: Vec<std::path::PathBuf>
    };
    let output = quote! {
        ::bpaf::long("files").argument("ARG").many()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn or_else_postpr_named() {
    let input: NamedField = parse_quote! {
        #[bpaf(argument("SPEED"), or_else(other_parser))]
        speed: f64
    };
    let output = quote! {
        ::bpaf::long("speed").argument("SPEED").or_else(other_parser())
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn or_else_postpr_unnamed() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(positional("SPEED"), or_else(other_parser))]
        f64
    };
    let output = quote! {
        ::bpaf::positional("SPEED").or_else(other_parser())
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

// skip catch for now
// #[test]
// fn many_catch() {
//     let input: NamedField = parse_quote! {
//         #[bpaf(argument("FILE"), many, catch)]
//         files: Vec<std::path::PathBuf>
//     };
//     let output = quote! {
//         ::bpaf::long("files").argument("FILE").many().catch()
//     };
//     assert_eq!(input.to_token_stream().to_string(), output.to_string());
// }
//
// #[test]
// fn collect_catch() {
//     let input: NamedField = parse_quote! {
//         #[bpaf(argument("FILE"), collect, catch)]
//         files: Vec<std::path::PathBuf>
//     };
//     let output = quote! {
//         ::bpaf::long("files").argument("FILE").collect().catch()
//     };
//     assert_eq!(input.to_token_stream().to_string(), output.to_string());
// }
//
// #[test]
// fn option_catch() {
//     let input: NamedField = parse_quote! {
//         #[bpaf(argument("FILE"), optional, catch)]
//         files: Option<std::path::PathBuf>
//     };
//     let output = quote! {
//         ::bpaf::long("files").argument("FILE").optional().catch()
//     };
//     assert_eq!(input.to_token_stream().to_string(), output.to_string());
// }
//
// #[test]
// fn some_catch() {
//     let input: NamedField = parse_quote! {
//         #[bpaf(argument("ARG"), some("files"), catch)]
//         files: Vec<std::path::PathBuf>
//     };
//     let output = quote! {
//         ::bpaf::long("files").argument("ARG").some("files").catch()
//     };
//     assert_eq!(input.to_token_stream().to_string(), output.to_string());
// }

#[test]
fn check_option_file_implicit() {
    let input: NamedField = parse_quote! {
        files: Option<PathBuf>
    };
    let output = quote! {
        ::bpaf::long("files").argument("ARG").optional()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn check_guard_fallback() {
    let input: NamedField = parse_quote! {
        #[bpaf(guard(positive, "must be positive"), fallback(1))]
        num: u32
    };
    let output = quote! {
        ::bpaf::long("num").argument("ARG").guard(positive, "must be positive").fallback(1)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn better_error_for_unnamed_argument() {
    let input = quote!(
        #[bpaf(argument("FILE"))]
        pub PathBuf
    );
    let err = parse2::<UnnamedField>(input).unwrap_err().to_string();
    assert_eq!(
        err,
        r#"Can't derive an explicit name for unnamed struct, try adding a name here like short('f') or long("name")"#
    );
}

#[test]
fn postprocessing_after_external() {
    let input: NamedField = parse_quote! {
        #[bpaf(external(verbose), fallback(42))]
        verbose: usize
    };
    let output = quote! {
        verbose().fallback(42)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn optional_external() {
    let input: NamedField = parse_quote! {
        #[bpaf(external(verbose))]
        verbose: Option<String>
    };
    let output = quote! {
        verbose()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn optional_external_shortcut() {
    let input: NamedField = parse_quote! {
        #[bpaf(external)]
        verbose: Option<String>
    };
    let output = quote! {
        verbose()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn optional_external_unnamed() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(external(verbose))]
        Option<String>
    };
    let output = quote! {
        verbose()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn optional_field_is_sane() {
    let input: NamedField = parse_quote! {
        name: Option<String>
    };
    let output = quote! {
        ::bpaf::long("name").argument("ARG").optional()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn vec_field_is_sane() {
    let input: NamedField = parse_quote! {
        names: Vec<String>
    };
    let output = quote! {
        ::bpaf::long("names").argument("ARG").many()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn positional_named_fields() {
    let input: NamedField = parse_quote! {
        #[bpaf(positional("ARG"))]
        name: String
    };
    let output = quote! {
        ::bpaf::positional("ARG")
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn strict_positional_named_fields() {
    let input: NamedField = parse_quote! {
        #[bpaf(positional("ARG"), strict)]
        name: String
    };
    let output = quote! {
        ::bpaf::positional("ARG").strict()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn non_strict_positional_named_fields() {
    let input: NamedField = parse_quote! {
        #[bpaf(positional("ARG"), non_strict)]
        name: String
    };
    let output = quote! {
        ::bpaf::positional("ARG").non_strict()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn posix_positional_named_fields() {
    let input: NamedField = parse_quote! {
        #[bpaf(positional("ARG"), posix)]
        name: String
    };
    let output = quote! {
        ::bpaf::positional("ARG").posix()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn posix_positional_unnamed_fields() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(positional("ARG"), posix)]
        String
    };
    let output = quote! {
        ::bpaf::positional("ARG").posix()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn strict_posix_positional_named_fields() {
    let input: NamedField = parse_quote! {
        #[bpaf(positional("ARG"), strict, posix)]
        name: String
    };
    let output = quote! {
        ::bpaf::positional("ARG").strict().posix()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn optional_named_pathed() {
    let input: NamedField = parse_quote! {
        #[bpaf(long, short)]
        pub config: Option<aws::Location>
    };
    let output = quote! {
        ::bpaf::long("config")
            .short('c')
            .argument("ARG")
            .optional()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn optional_unnamed_pathed() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(long("config"), short('c'))]
        Option<aws::Location>
    };
    let output = quote! {
        ::bpaf::long("config")
            .short('c')
            .argument("ARG")
            .optional()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn implicit_optional_argument_with_name() {
    let input: NamedField = parse_quote! {
        #[bpaf(argument("N"))]
        config: Option<u64>
    };
    let output = quote! {
        ::bpaf::long("config")
            .argument("N")
            .optional()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn explicit_optional_argument_with_name() {
    let input: NamedField = parse_quote! {
        #[bpaf(argument("N"), optional)]
        config: Option<u64>
    };
    let output = quote! {
        ::bpaf::long("config")
            .argument("N")
            .optional()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

// group - shell completion, not needed
// #[test]
// fn optional_argument_with_name_complete() {
//     let input: NamedField = parse_quote! {
//         #[bpaf(argument("N"), complete(magic), group("hi"))]
//         config: Option<u64>
//     };
//     let output = quote! {
//         ::bpaf::long("config")
//             .argument("N")
//             .complete(magic)
//             .optional()
//             .group("hi")
//     };
//     assert_eq!(input.to_token_stream().to_string(), output.to_string());
// }

#[test]
fn many_argument_with_name_complete() {
    let input: NamedField = parse_quote! {
        #[bpaf(argument("N"), complete(magic))]
        config: Vec<u64>
    };
    let output = quote! {
        ::bpaf::long("config")
            .argument("N")
            .complete(magic)
            .many()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn some_arguments() {
    let input: NamedField = parse_quote! {
        #[bpaf(argument("N"), some("need params"))]
        config: Vec<u32>
    };
    let output = quote! {
        ::bpaf::long("config")
            .argument("N")
            .some("need params")
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn env_argument() {
    let input: NamedField = parse_quote! {
        #[bpaf(env(sim::DB), argument("N"), some("need params"))]
        config: Vec<u32>
    };
    let output = quote! {
        ::bpaf::env(sim::DB)
            .long("config")
            .argument("N")
            .some("need params")
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn explicit_switch_argument() {
    let input: NamedField = parse_quote! {
        #[bpaf(switch)]
        item: bool
    };
    let output = quote! {
        ::bpaf::long("item").switch()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn explicit_req_flag_argument() {
    let input: NamedField = parse_quote! {
        #[bpaf(req_flag(true))]
        item: bool
    };
    let output = quote! {
        ::bpaf::long("item").req_flag(true)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn implicit_switch_argument() {
    let input: NamedField = parse_quote! {
        item: bool
    };
    let output = quote! {
        ::bpaf::long("item").switch()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn explicit_flag_argument_1() {
    let input: NamedField = parse_quote! {
        #[bpaf(flag(true, false))]
        item: bool
    };
    let output = quote! {
        ::bpaf::long("item").flag(true, false)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn explicit_flag_argument_2() {
    let input: NamedField = parse_quote! {
        #[bpaf(flag(True, False))]
        item: Bool
    };
    let output = quote! {
        ::bpaf::long("item").flag(True, False)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn explicit_flag_argument_3() {
    let input: NamedField = parse_quote! {
        #[bpaf(flag(True, False), optional)]
        item: Option<Bool>
    };
    let output = quote! {
        ::bpaf::long("item").flag(True, False).optional()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn hide_and_group_help() {
    let input: NamedField = parse_quote! {
        #[bpaf(hide, group_help("potato"))]
        item: bool
    };
    let output = quote! {
        ::bpaf::long("item").switch().hide().group_help("potato")
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn any_field_1() {
    let input: NamedField = parse_quote! {
        #[bpaf(any("ARG", Some))]
        /// help
        field: OsString
    };
    let output = quote! {
        ::bpaf::any("ARG", Some).help("help")
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn unnamed_field_with_ignore_rustdoc() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(any("FOO", Some), ignore_rustdoc)]
        /// help
        String
    };
    let output = quote! {
        ::bpaf::any("FOO", Some)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn any_field_2() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(any("FOO", Some))]
        /// help
        String
    };
    let output = quote! {
        ::bpaf::any("FOO", Some).help("help")
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn any_field_3() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(any("FOO", Some))]
        /// help
        Vec<String>
    };
    let output = quote! {
        ::bpaf::any("FOO", Some).help("help").many()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn any_field_4() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(any("FOO", Some))]
        /// help
        Vec<OsString>
    };
    let output = quote! {
        ::bpaf::any("FOO", Some).help("help").many()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn any_field_custom_help() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(any("FOO", Some), help(custom_help))]
        /// help
        Vec<OsString>
    };
    let output = quote! {
        ::bpaf::any("FOO", Some).help(custom_help).many()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn any_field_5() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(any("FOO", check))]
        /// help
        Vec<OsString>
    };
    let output = quote! {
        ::bpaf::any("FOO", check).help("help").many()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn any_field_many_custom_help() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(any("FOO", check), help(custom_help))]
        /// help
        Vec<OsString>
    };
    let output = quote! {
        ::bpaf::any("FOO", check).help(custom_help).many()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn any_field_6() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(any("FOO", |x| (x == "--lit").then_some(())))]
        /// help
        ()
    };
    let output = quote! {
        ::bpaf::any("FOO", |x| (x == "--lit").then_some(())).help("help")
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn unit_fields_are_required() {
    let input: NamedField = parse_quote! {
        /// help
        name: ()
    };
    let output = quote! {
        ::bpaf::long("name").req_flag(()).help("help")
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn ignore_rustdoc_without_help() {
    let input: NamedField = parse_quote! {
        /// help
        #[bpaf(ignore_rustdoc)]
        name: ()
    };
    let output = quote! {
        ::bpaf::long("name").req_flag(())
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn ignore_rustdoc_with_help() {
    let input: NamedField = parse_quote! {
        /// help
        #[bpaf(help("custom help"), ignore_rustdoc)]
        name: ()
    };
    let output = quote! {
        ::bpaf::long("name").req_flag(()).help("custom help")
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn unit_fields_are_required_custom_help() {
    let input: NamedField = parse_quote! {
        /// help
        #[bpaf(help(custom_help))]
        name: ()
    };
    let output = quote! {
        ::bpaf::long("name").req_flag(()).help(custom_help)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn hide_usage() {
    let input: NamedField = parse_quote! {
        #[bpaf(hide_usage)]
        field: u32
    };
    let output = quote! {
        ::bpaf::long("field").argument("ARG").hide_usage()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn custom_usage() {
    let input: NamedField = parse_quote! {
        #[bpaf(custom_usage(usage()))]
        field: u32
    };
    let output = quote! {
        ::bpaf::long("field").argument("ARG").custom_usage(usage())
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn argument_with_manual_parse() {
    let input: NamedField = parse_quote! {
        #[bpaf(argument::<String>("N"), parse(twice_the_num))]
        number: u32
    };
    let output = quote! {
        ::bpaf::long("number")
            .argument::<String>("N")
            .parse(twice_the_num)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

// consumer must be first
// #[test]
// fn optional_external_strange() {
//     let input: NamedField = parse_quote! {
//         #[bpaf(optional, external(seed),)]
//         number: u32
//     };
//
//     let output = quote! {
//         seed().optional()
//     };
//     assert_eq!(input.to_token_stream().to_string(), output.to_string());
// }

#[test]
fn fallback_with_lambda() {
    let input: NamedField = parse_quote! {
        /// help
        #[bpaf(
            argument::<String>("FLAGS"),
            fallback_with(|| Ok::<_, ()>("http-only")),
        )]
        session_flags: String
    };

    let output = quote! {
        ::bpaf::long("session-flags")
            .argument::<String>("FLAGS")
            .help("help")
            .fallback_with(|| Ok::<_,()>("http-only"))
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn positional_bool() {
    let input: NamedField = parse_quote! {
        #[bpaf(positional::<bool>("O_O"))]
        flag: bool
    };
    let output = quote! {
        ::bpaf::positional::<bool>("O_O")
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn pure_optional_named() {
    let input: NamedField = parse_quote! {
        #[bpaf(pure(x))]
        flag: Option<Vec<X>>
    };
    let output = quote! {
        ::bpaf::pure(x)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn pure_vec_named() {
    let input: NamedField = parse_quote! {
        #[bpaf(pure(x))]
        flag: Vec<X>
    };
    let output = quote! {
        ::bpaf::pure(x)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn pure_optional_pos() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(pure(x))]
         Option<Vec<X>>
    };
    let output = quote! {
        ::bpaf::pure(x)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn raw_literal() {
    let input: NamedField = parse_quote! {
        r#in: bool
    };
    let output = quote! {
        ::bpaf::long("in").switch()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn any_two_turbofish() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(any::<&str, String>("FOO", check))]
        /// help
        String
    };
    let output = quote! {
        ::bpaf::any::<&str, String>("FOO", check).help("help")
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[track_caller]
fn field_res(input: TokenStream, expected: TokenStream) {
    let field: NamedField = parse2(input).unwrap();
    assert_eq!(field.to_token_stream().to_string(), expected.to_string());
}

#[track_caller]
fn field_fail(input: TokenStream, expected_err: &str) {
    let err = parse2::<NamedField>(input).unwrap_err().to_string();
    assert_eq!(err, expected_err)
}

#[track_caller]
fn unnamed_res(input: TokenStream, expected: TokenStream) {
    let field: UnnamedField = parse2(input).unwrap();
    assert_eq!(field.to_token_stream().to_string(), expected.to_string());
}

#[track_caller]
fn unnamed_fail(input: TokenStream, expected_err: &str) {
    let err = parse2::<UnnamedField>(input).unwrap_err().to_string();
    assert_eq!(err, expected_err)
}

#[test]
fn implicit_parser_with_help() {
    field_res(
        parse_quote! {
            /// help
            number: usize
        },
        quote!(::bpaf::long("number").argument("ARG").help("help")),
    );
}

#[test]
fn guard_with_closure_comma() {
    field_res(
        parse_quote! {
            #[bpaf(guard(|a, b| a < b, "must be ordered"))]
            pair: (u32, u32)
        },
        quote!(
            ::bpaf::long("pair")
                .argument("ARG")
                .guard(|a, b| a < b, "must be ordered")
        ),
    );
}

#[test]
fn guard_with_turbofish_comma() {
    field_res(
        parse_quote! {
            #[bpaf(argument::<u32>("N"), guard(check::<u32, u32>, "msg"))]
            number: u32
        },
        quote!(
            ::bpaf::long("number")
                .argument::<u32>("N")
                .guard(check::<u32, u32>, "msg")
        ),
    );
}

#[test]
fn flag_with_three_arguments_fails() {
    let input = quote! {
        #[bpaf(flag(a, b, c))]
        item: bool
    };
    assert!(parse2::<NamedField>(input).is_err());
}

#[test]
fn map_typed_argument() {
    field_res(
        parse_quote! {
            #[bpaf(argument::<usize>("NUM"), map(double))]
            number: usize
        },
        quote!(::bpaf::long("number").argument::<usize>("NUM").map(double)),
    );
}

#[test]
fn map_implicit_consumer() {
    field_res(
        parse_quote! {
            #[bpaf(map(double))]
            number: usize
        },
        quote!(::bpaf::long("number").argument("ARG").map(double)),
    );
}

#[test]
fn map_untyped_consumer() {
    field_res(
        parse_quote! {
            #[bpaf(argument("X"), map(double))]
            number: usize
        },
        quote!(::bpaf::long("number").argument("X").map(double)),
    );
}

#[test]
fn map_turbofish_untyped_consumer() {
    field_res(
        parse_quote! {
            #[bpaf(map::<String>(stringify))]
            number: String
        },
        quote!(
            ::bpaf::long("number")
                .argument("ARG")
                .map::<_, String>(stringify)
        ),
    );
}

#[test]
fn parse_implicit_consumer() {
    field_res(
        parse_quote! {
            #[bpaf(parse(split_and_parse))]
            number: u32
        },
        quote!(
            ::bpaf::long("number")
                .argument("ARG")
                .parse(split_and_parse)
        ),
    );
}

#[test]
fn map_positional_untyped_consumer() {
    field_res(
        parse_quote! {
            #[bpaf(positional("N"), map(double))]
            number: usize
        },
        quote!(::bpaf::positional("N").map(double)),
    );
}

#[test]
fn many_argument() {
    field_res(
        parse_quote! {
            #[bpaf(argument("FILE"), many)]
            files: Vec<std::path::PathBuf>
        },
        quote!(::bpaf::long("files").argument("FILE").many()),
    );
}

#[test]
fn collect_argument() {
    field_res(
        parse_quote! {
            #[bpaf(argument("FILE"), collect)]
            files: Vec<std::path::PathBuf>
        },
        quote!(::bpaf::long("files").argument("FILE").collect()),
    );
}

#[test]
fn map_then_many() {
    field_res(
        parse_quote! {
            #[bpaf(argument::<u32>("N"), map(f), many)]
            items: Vec<u32>
        },
        quote!(::bpaf::long("items").argument::<u32>("N").map(f).many()),
    );
}

#[test]
fn many_then_map() {
    field_res(
        parse_quote! {
            #[bpaf(argument::<u32>("N"), many, map(f))]
            items: Vec<u32>
        },
        quote!(::bpaf::long("items").argument::<u32>("N").many().map(f)),
    );
}

#[test]
fn optional_then_guard() {
    field_res(
        parse_quote! {
            #[bpaf(argument::<u32>("N"), optional, guard(p, "m"))]
            value: Option<u32>
        },
        quote!(
            ::bpaf::long("value")
                .argument::<u32>("N")
                .optional()
                .guard(p, "m")
        ),
    );
}

#[test]
fn guard_then_optional() {
    field_res(
        parse_quote! {
            #[bpaf(argument::<u32>("N"), guard(p, "m"), optional)]
            value: Option<u32>
        },
        quote!(
            ::bpaf::long("value")
                .argument::<u32>("N")
                .guard(p, "m")
                .optional()
        ),
    );
}

#[test]
fn option_argument() {
    field_res(
        parse_quote! {
            #[bpaf(argument("FILE"), optional)]
            files: Option<std::path::PathBuf>
        },
        quote!(::bpaf::long("files").argument("FILE").optional()),
    );
}

#[test]
fn some_argument() {
    field_res(
        parse_quote! {
            #[bpaf(argument("ARG"), some("files"))]
            files: Vec<std::path::PathBuf>
        },
        quote!(::bpaf::long("files").argument("ARG").some("files")),
    );
}

#[test]
fn optional_argument_with_name_complete() {
    field_res(
        parse_quote! {
            #[bpaf(argument("N"), complete(magic), group("hi"))]
            config: Option<u64>
        },
        quote!(
            ::bpaf::long("config")
                .argument("N")
                .complete(magic)
                .optional()
                .group_help("hi")
        ),
    );
}

#[test]
fn any_field_two_turbofish() {
    field_res(
        parse_quote! {
            #[bpaf(any::<&str, String>("FOO", check))]
            /// help
            field: String
        },
        quote!(::bpaf::any::<&str, String>("FOO", check).help("help")),
    );
}

#[test]
fn two_consumers() {
    let input = quote! {
        #[bpaf(argument("X"), switch)]
        field: u32
    };
    field_fail(input, "Only one consumer per attribute is allowed!");
}

#[test]
fn duplicate_help() {
    field_res(
        parse_quote! {
            #[bpaf(help("a"), help("b"))]
            field: u32
        },
        quote!(::bpaf::long("field").argument("ARG").help("b")),
    );
}

#[test]
fn name_on_free_consumer() {
    let input = quote! {
        #[bpaf(positional, short)]
        field: u32
    };
    field_fail(input, "unexpected annotation");
}

#[test]
fn two_switches() {
    let input = quote! {
        #[bpaf(short, switch, short, switch)]
        field: bool
    };
    field_fail(input, "Only one consumer per attribute is allowed!");
}

#[test]
fn modifier_consumer_mismatch() {
    let input = quote! {
        #[bpaf(argument("X"), strict)]
        field: u32
    };
    field_fail(input, "unexpected annotation");
}

#[test]
fn flag_modifier_on_argument() {
    let input = quote! {
        #[bpaf(argument("X"), default)]
        field: u32
    };
    field_fail(input, "unexpected annotation");
}

#[test]
fn modifiers_after_postpr() {
    let input = quote! {
        #[bpaf(argument("X"), guard(x, "msg"), adjacent)]
        field: u32
    };
    field_fail(input, "unexpected annotation");
}

#[test]
fn untyped_consumer_with_type_changing_post() {
    field_res(
        parse_quote! {
            #[bpaf(argument("X"), map(f))]
            field: u32
        },
        quote!(::bpaf::long("field").argument("X").map(f)),
    );
}

#[test]
fn unknown_attribute() {
    let input = quote! {
        #[bpaf(such_attribute)]
        field: u32
    };
    field_fail(input, "unexpected annotation");
}

#[test]
fn unnamed_derived_positional() {
    unnamed_res(parse_quote!(usize), quote!(::bpaf::positional("ARG")));
}

#[test]
fn unnamed_derived_positional_with_help() {
    unnamed_res(
        parse_quote! {
            /// help
            usize
        },
        quote!(::bpaf::positional("ARG").help("help")),
    );
}

#[test]
fn unnamed_option_implicit() {
    unnamed_res(
        parse_quote!(Option<u32>),
        quote!(::bpaf::positional("ARG").optional()),
    );
}

#[test]
fn unnamed_vec_implicit() {
    unnamed_res(
        parse_quote!(Vec<u32>),
        quote!(::bpaf::positional("ARG").many()),
    );
}

#[test]
fn unnamed_check_guard() {
    unnamed_res(
        parse_quote! {
            #[bpaf(guard(odd, "must be odd"))]
            usize
        },
        quote!(::bpaf::positional("ARG").guard(odd, "must be odd")),
    );
}

#[test]
fn unnamed_pure_value() {
    unnamed_res(
        parse_quote! {
            #[bpaf(pure(42))]
            usize
        },
        quote!(::bpaf::pure(42)),
    );
}

#[test]
fn unnamed_pure_with_value() {
    unnamed_res(
        parse_quote! {
            #[bpaf(pure_with(detect_color))]
            usize
        },
        quote!(::bpaf::pure_with(detect_color)),
    );
}

#[test]
fn unnamed_strict_posix_positional() {
    unnamed_res(
        parse_quote! {
            #[bpaf(positional("ARG"), strict, posix)]
            String
        },
        quote!(::bpaf::positional("ARG").strict().posix()),
    );
}

#[test]
fn unnamed_optional_named_pathed() {
    unnamed_res(
        parse_quote! {
            #[bpaf(long("config"), short('c'))]
            Option<aws::Location>
        },
        quote!(::bpaf::long("config").short('c').argument("ARG").optional()),
    );
}

#[test]
fn unnamed_any_field() {
    unnamed_res(
        parse_quote! {
            #[bpaf(any("FOO", Some))]
            /// help
            Vec<String>
        },
        quote!(::bpaf::any("FOO", Some).help("help").many()),
    );
}

#[test]
fn unnamed_any_two_turbofish() {
    unnamed_res(
        parse_quote! {
            #[bpaf(any::<&str, String>("FOO", check))]
            /// help
            String
        },
        quote!(::bpaf::any::<&str, String>("FOO", check).help("help")),
    );
}

#[test]
fn unnamed_ignore_rustdoc() {
    unnamed_res(
        parse_quote! {
            #[bpaf(any("FOO", Some), ignore_rustdoc)]
            /// help
            String
        },
        quote!(::bpaf::any("FOO", Some)),
    );
}

#[test]
fn unnamed_external_path() {
    unnamed_res(
        parse_quote! {
            #[bpaf(external(path::level))]
            usize
        },
        quote!(path::level()),
    );
}

#[test]
fn unnamed_switch_with_explicit_name() {
    unnamed_res(
        parse_quote! {
            #[bpaf(switch, long("x"))]
            bool
        },
        quote!(::bpaf::long("x").switch()),
    );
}

#[test]
fn unnamed_derived_bool() {
    let input = quote!(bool);
    unnamed_fail(
        input,
        "Refusing to derive a positional item for bool, you can fix this by either adding a short/long name or making it positional explicitly",
    );
}

#[test]
fn unnamed_derived_unit() {
    let input = quote!(());
    unnamed_fail(
        input,
        "Refusing to derive a positional item for (), you can fix this by either adding a short/long name or making it positional explicitly",
    );
}

#[test]
fn unnamed_bare_short() {
    let input = quote! {
        #[bpaf(short)]
        u32
    };
    unnamed_fail(
        input,
        "Can't derive an explicit name for unnamed struct, try adding a name here like short('f') or long(\"name\")",
    );
}

#[test]
fn unnamed_bare_long() {
    let input = quote! {
        #[bpaf(long)]
        u32
    };
    unnamed_fail(
        input,
        "Can't derive an explicit name for unnamed struct, try adding a name here like short('f') or long(\"name\")",
    );
}

#[test]
fn unnamed_external_bare() {
    let input = quote! {
        #[bpaf(external)]
        u32
    };
    unnamed_fail(
        input,
        "Can't derive name for this external, try specifying one",
    );
}

#[test]
fn name_only_field() {
    let field = quote::quote! {
        #[bpaf(long("verb"))]
        pub verbose: bool,
    };
    let (_, ts) = parse_named.parse2(field).unwrap();
    assert_eq!(
        ts.to_string(),
        quote::quote!(::bpaf::long("verb").switch()).to_string()
    );
}

#[test]
fn implicit_name_switch2() {
    let field = quote::quote! {
        #[bpaf(switch, long("verbose"))]
        pub verbose: bool,
    };
    let (_, ts) = parse_named.parse2(field).unwrap();
    assert_eq!(
        ts.to_string(),
        quote::quote!(::bpaf::long("verbose").switch()).to_string()
    );
}

#[test]
fn implicit_name_switch() {
    let field = quote::quote! {
        #[bpaf(switch)]
        pub verbose: bool,
    };
    let (_, ts) = parse_named.parse2(field).unwrap();
    assert_eq!(
        ts.to_string(),
        quote::quote!(::bpaf::long("verbose").switch()).to_string()
    );
}

#[test]
fn no_implicit_repeat_for_external() {
    let field = quote::quote! {
        #[bpaf(external(verbose))]
        verbose: Option<String>,
    };
    let (_, ts) = parse_named.parse2(field).unwrap();
    assert_eq!(ts.to_string(), quote::quote!(verbose()).to_string());
}

#[test]
fn no_implicit_repeat_for_pure() {
    let field = quote::quote! {
        #[bpaf(pure(x))]
        flag: Vec<X>,
    };
    let (_, ts) = parse_named.parse2(field).unwrap();
    assert_eq!(ts.to_string(), quote::quote!(::bpaf::pure(x)).to_string());
}

#[test]
fn rustdoc_help() {
    let field = quote::quote! {
        #[bpaf(help("help"), ignore_rustdoc)]
        /// also help
        verbose: bool,
    };

    let (_, ts) = parse_named.parse2(field).unwrap();
    assert_eq!(
        ts.to_string(),
        quote::quote!(::bpaf::long("verbose").switch().help("help")).to_string()
    );
}
