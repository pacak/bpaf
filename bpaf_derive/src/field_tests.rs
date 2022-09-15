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
        ::bpaf::long("number").help("help").argument("ARG").from_str::<usize>()
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
        ::bpaf::short('n').long("number").argument("ARG").from_str::<usize>()
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
        ::bpaf::long("number").argument("ARG").from_str::<f64>().fallback(3.1415)
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
        ::bpaf::long("number").argument("ARG").from_str::<f64>().fallback_with(external)
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
        ::bpaf::long("number").argument("ARG").from_str::<usize>().guard(positive, "msg")
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
        ::bpaf::long("number").argument("ARG").from_str::<usize>().guard(positive, MSG)
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
        #[bpaf(argument("NUM"), from_str(usize), map(double))]
        number: usize
    };
    let output = quote! {
        ::bpaf::long("number").argument("NUM").from_str::<usize>().map(double)
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
        ::bpaf::positional("ARG").from_str::<usize>().guard(odd, "must be odd")
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
        ::bpaf::long("speed").argument("SPEED").from_str::<f64>().fallback(42.0)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn check_many_files_implicit() {
    let input: NamedField = parse_quote! {
        files: Vec<std::path::PathBuf>
    };
    let output = quote! {
        ::bpaf::long("files").argument_os("ARG").map(std::path::PathBuf::from).many()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn check_option_file_implicit() {
    let input: NamedField = parse_quote! {
        files: Option<PathBuf>
    };
    let output = quote! {
        ::bpaf::long("files").argument_os("ARG").map(PathBuf::from).optional()
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
        ::bpaf::long("num").argument("ARG").from_str::<u32>().guard(positive, "must be positive").fallback(1)
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
fn optional_named_pathed() {
    let input: NamedField = parse_quote! {
        #[bpaf(long, short)]
        pub config: Option<aws::Location>
    };
    let output = quote! {
        ::bpaf::long("config")
            .short('c')
            .argument("ARG")
            .from_str::<aws::Location>()
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
            .from_str::<aws::Location>()
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
            .argument("N")
            .from_str::<u64>()
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
            .argument("N")
            .from_str::<u64>()
            .optional()
            .complete(magic)
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn some_arguments() {
    let input: NamedField = parse_quote! {
        #[bpaf(argument("N"), from_str(u32), some("need params"))]
        config: Vec<u32>
    };
    let output = quote! {
        ::bpaf::long("config")
            .argument("N")
            .from_str::<u32>()
            .some("need params")
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn env_argument() {
    let input: NamedField = parse_quote! {
        #[bpaf(env(sim::DB), argument("N"), from_str(u32), some("need params"))]
        config: Vec<u32>
    };
    let output = quote! {
        ::bpaf::long("config")
            .env(sim::DB)
            .argument("N")
            .from_str::<u32>()
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
#[should_panic]
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
        ::bpaf::any("ARG").help("help").os()
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
        ::bpaf::any("FOO").help("help")
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
        ::bpaf::any("FOO").help("help").many()
    };
    assert_eq!(input.to_token_stream().to_string(), output.to_string());
}

#[test]
fn any_field_4() {
    let input: UnnamedField = parse_quote! {
        #[bpaf(any("FOO"), os)]
        /// help
        Vec<OsString>
    };
    let output = quote! {
        ::bpaf::any("FOO").help("help").os().many()
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
