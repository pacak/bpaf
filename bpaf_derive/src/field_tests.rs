use crate::field::*;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::Parse;
use syn::parse2;
use syn::{parse, parse_quote, Result};

#[derive(Debug, Clone)]
struct UnnamedField {
    parser: Field,
}

impl Parse for UnnamedField {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let parser = Field::parse_unnamed(input)?;
        Ok(Self { parser })
    }
}

impl ToTokens for UnnamedField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.parser.to_tokens(tokens)
    }
}

#[derive(Debug, Clone)]
struct NamedField {
    parser: Field,
}

impl Parse for NamedField {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let parser = Field::parse_named(input)?;
        Ok(Self { parser })
    }
}

impl ToTokens for NamedField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.parser.to_tokens(tokens)
    }
}

#[track_caller]
fn field_trans_fail(input: TokenStream, expected_err: &str) {
    let err = syn::parse2::<NamedField>(input).unwrap_err().to_string();
    assert_eq!(err, expected_err)
}

#[test]
fn implicit_parser() {
    let input: NamedField = parse_quote! {
        /// help
        number: usize
    };
    let output = quote! {
        ::bpaf::long("number").help("help").argument::<usize>("ARG")
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
        ::bpaf::short('n').long("number").argument::<usize>("ARG")
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
        ::bpaf::long("number").argument::<f64>("ARG").fallback(3.1415)
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
        ::bpaf::long("number").argument::<f64>("ARG").fallback(3.1415).display_fallback()
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
        ::bpaf::long("number").argument::<f64>("ARG").fallback_with(external)
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
        ::bpaf::long("number").argument::<usize>("ARG").guard(positive, "msg")
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
        ::bpaf::long("number").argument::<usize>("ARG").guard(positive, MSG)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn derive_help() {
    let input: NamedField = parse_quote! {
        /// multi
        ///
        /// vis
        ///
        ///
        /// hidden
        pub(crate) flag: bool
    };
    let output = quote! {
        ::bpaf::long("flag").help("multi\nvis").switch()
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
fn map_requires_explicit_parser2() {
    let input = quote! {
        #[bpaf(map(double))]
        pub number: usize
    };
    let err = "Can't derive implicit consumer with this annotation present";
    field_trans_fail(input, err);
}

#[test]
fn check_guard() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(guard(odd, "must be odd"))]
        usize
    };

    let output = quote! {
        ::bpaf::positional::<usize>("ARG").guard(odd, "must be odd")
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
        ::bpaf::long("speed").argument::<f64>("SPEED").fallback(42.0)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn check_many_files_implicit() {
    let input: NamedField = parse_quote! {
        files: Vec<std::path::PathBuf>
    };
    let output = quote! {
        ::bpaf::long("files").argument::<std::path::PathBuf>("ARG").many()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn many_catch() {
    let input: NamedField = parse_quote! {
        #[bpaf(catch)]
        files: Vec<std::path::PathBuf>
    };
    let output = quote! {
        ::bpaf::long("files").argument::<std::path::PathBuf>("ARG").many().catch()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn option_catch() {
    let input: NamedField = parse_quote! {
        #[bpaf(catch)]
        files: Option<std::path::PathBuf>
    };
    let output = quote! {
        ::bpaf::long("files").argument::<std::path::PathBuf>("ARG").optional().catch()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn some_catch() {
    let input: NamedField = parse_quote! {
        #[bpaf(argument("ARG"), some("files"), catch)]
        files: Vec<std::path::PathBuf>
    };
    let output = quote! {
        ::bpaf::long("files").argument::<std::path::PathBuf>("ARG").some("files").catch()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn check_option_file_implicit() {
    let input: NamedField = parse_quote! {
        files: Option<PathBuf>
    };
    let output = quote! {
        ::bpaf::long("files").argument::<PathBuf>("ARG").optional()
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
        ::bpaf::long("num").argument::<u32>("ARG").guard(positive, "must be positive").fallback(1)
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
        "This consumer needs a name, you can specify it with long(\"name\") or short('n')"
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
        ::bpaf::long("name").argument::<String>("ARG").optional()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn vec_field_is_sane() {
    let input: NamedField = parse_quote! {
        names: Vec<String>
    };
    let output = quote! {
        ::bpaf::long("names").argument::<String>("ARG").many()
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
        ::bpaf::positional::<String>("ARG")
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
            .argument::<aws::Location>("ARG")
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
            .argument::<aws::Location>("ARG")
            .optional()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn optional_argument_with_name() {
    let input: NamedField = parse_quote! {
        #[bpaf(argument("N"))]
        config: Option<u64>
    };
    let output = quote! {
        ::bpaf::long("config")
            .argument::<u64>("N")
            .optional()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn optional_argument_with_name_complete() {
    let input: NamedField = parse_quote! {
        #[bpaf(argument("N"), complete(magic))]
        config: Option<u64>
    };
    let output = quote! {
        ::bpaf::long("config")
            .argument::<u64>("N")
            .optional()
            .complete(magic)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn optional_argument_with_name_shell_complete() {
    let input: NamedField = parse_quote! {
        #[bpaf(argument("N"), complete_shell(magic))]
        config: Option<u64>
    };
    let output = quote! {
        ::bpaf::long("config")
            .argument::<u64>("N")
            .optional()
            .complete_shell(magic)
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
            .argument::<u32>("N")
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
        ::bpaf::long("config")
            .env(sim::DB)
            .argument::<u32>("N")
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
        #[bpaf(any)]
        /// help
        field: OsString
    };
    let output = quote! {
        ::bpaf::any::<OsString>("ARG").help("help")
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn any_field_2() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(any("FOO"))]
        /// help
        String
    };
    let output = quote! {
        ::bpaf::any::<String>("FOO").help("help")
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn any_field_3() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(any("FOO"))]
        /// help
        Vec<String>
    };
    let output = quote! {
        ::bpaf::any::<String>("FOO").help("help").many()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn any_field_4() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(any("FOO"))]
        /// help
        Vec<OsString>
    };
    let output = quote! {
        ::bpaf::any::<OsString>("FOO").help("help").many()
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
        ::bpaf::long("name").help("help").req_flag(())
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
        ::bpaf::long("field").argument::<u32>("ARG").hide_usage()
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

#[test]
fn optional_external_strange() {
    let input: TokenStream = quote! {
        #[bpaf(optional, external(seed))]
        number: u32
    };
    let msg =
        "keyword `external` is at stage `external` can't follow previous stage (postprocessing)";
    field_trans_fail(input, msg);
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
