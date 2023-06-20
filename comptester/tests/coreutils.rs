use comptester::*;
use pretty_assertions::assert_eq;

#[test]
fn all_options_zsh() {
    let buf = zsh_comptest("coreutils \t", false).unwrap();
    let expected = r"% coreutils
arch                     -- Print machine architecture.
b2sum                    -- Print or check BLAKE2 (512-bit) checksums.
base32                   -- Base32 encode or decode FILE, or standard input, to standard output.
basename
cat";
    assert_eq!(buf, expected);
}

#[test]
fn all_options_bash() {
    let buf = bash_comptest("coreutils \t\t", false).unwrap();
    let expected = r"%
arch                     -- Print machine architecture.
b2sum                    -- Print or check BLAKE2 (512-bit) checksums.
base32                   -- Base32 encode or decode FILE, or standard input, to standard output.
basename
cat";
    assert_eq!(buf, expected);
}

#[test]
fn cat_zsh() {
    let buf = zsh_comptest("coreutils cat -- \t", false).unwrap();
    assert_eq!(
        buf,
        r"% coreutils cat --
      FILE"
    );
}

#[test]
fn cat_bash() {
    let buf = bash_comptest("coreutils cat -- \t\t", false).unwrap();
    assert_eq!(buf, "%\nFILE");
}

/*
//let buf = zsh_comptest("derive_show_asm ?", false).unwrap();
#[test]
fn all_options_zsh() {
    let buf = zsh_comptest("derive_show_asm \t", false).unwrap();

    let expected = r"% derive_show_asm
--att                    -- Generate assembly using AT&T style
--dry                    -- Produce a build plan instead of actually building
--frozen                 -- Requires Cargo.lock and cache are up to date
--intel                  -- Generate assembly using Intel style
--locked                 -- Requires Cargo.lock is up to date
--manifest-path=PATH     -- Path to Cargo.toml
--offline                -- Run without accessing the network
--package=SPEC           -- Package to use if ambigous
--target-dir=DIR         -- Custom target directory for generated artifacts
--verbose                -- more verbose output, can be specified multiple times
Select artifact to use for analysis
--lib                    -- Show results from library code  --example=EXAMPLE        -- Show results from an example
--test=TEST              -- Show results from a test        --bin=BIN                -- Show results from a binary
--bench=BENCH            -- Show results from a benchmark
How to render output
--rust                   -- Print interleaved Rust code
--color                  -- Enable color highlighting
--no-color               -- Disable color highlighting
--full-name              -- include full demangled name instead of just prefix
Item to pick from the output
FUNCTION                 -- Complete or partial function name to filter";
    assert_eq!(buf, expected);
}

#[test]
fn single_result_zsh() {
    let buf = zsh_comptest("derive_show_asm --li\t", false).unwrap();
    assert_eq!(buf, "% derive_show_asm --lib");
}

#[test]
fn zsh_file_completion() {
    let buf = zsh_comptest("derive_show_asm --manifest-path \t", false).unwrap();
    assert_eq!(
        buf,
        "% derive_show_asm --manifest-path
Cargo.toml  src/        tests/"
    );

    let buf = zsh_comptest("derive_show_asm --manifest-path C\t", false).unwrap();
    assert_eq!(buf, "% derive_show_asm --manifest-path Cargo.toml");
}

#[test]
fn zsh_example_single() {
    let buf = zsh_comptest("derive_show_asm --example de\t", false).unwrap();
    assert_eq!(buf, "% derive_show_asm --example derive_show_asm");
}

#[test]
fn zsh_example_variants() {
    let buf = zsh_comptest("derive_show_asm --example co\t", false).unwrap();
    assert_eq!(
        buf,
        "% derive_show_asm --example co
Select artifact to use for analysis
coreutils  comonad"
    );
    let buf = zsh_comptest("derive_show_asm --example core\t", false).unwrap();

    assert_eq!(buf, "% derive_show_asm --example coreutils");
}

#[test]
fn single_result_bash() {
    let buf = bash_comptest("derive_show_asm --li\t", false).unwrap();
    assert_eq!(buf, "% derive_show_asm --lib");
}

#[test]
fn all_options_bash() {
    let buf = bash_comptest("derive_show_asm \t\t", false).unwrap();
    let expected = r"%
--manifest-path=PATH     -- Path to Cargo.toml
--target-dir=DIR         -- Custom target directory for generated artifacts
--package=SPEC           -- Package to use if ambigous
Select artifact to use for analysis
--lib                    -- Show results from library code
--test=TEST              -- Show results from a test
--bench=BENCH            -- Show results from a benchmark
--example=EXAMPLE        -- Show results from an example
--bin=BIN                -- Show results from a binary
--dry                    -- Produce a build plan instead of actually building
--frozen                 -- Requires Cargo.lock and cache are up to date
--locked                 -- Requires Cargo.lock is up to date
--offline                -- Run without accessing the network
How to render output
--rust                   -- Print interleaved Rust code
--color                  -- Enable color highlighting
--no-color               -- Disable color highlighting
--full-name              -- include full demangled name instead of just prefix
--verbose                -- more verbose output, can be specified multiple times
--intel                  -- Generate assembly using Intel style
--att                    -- Generate assembly using AT&T style
Item to pick from the output
FUNCTION                 -- Complete or partial function name to filter";
    assert_eq!(buf, expected);
}

#[test]
fn bash_file_completion() {
    let buf = bash_comptest("derive_show_asm --manifest-path \t\t", false).unwrap();
    assert_eq!(buf, "%\nCargo.toml  src/        tests/");

    // file completion with mask in bash uses _filedir which
    // renders directories all the time
    let buf = bash_comptest("derive_show_asm --manifest-path Ca\t\t", false).unwrap();
    assert_eq!(buf, "%\nCargo.toml  src/        tests/");
}

#[test]
fn bash_example_single() {
    let buf = bash_comptest("derive_show_asm --example de\t", false).unwrap();
    assert_eq!(buf, "% derive_show_asm --example derive_show_asm");
}

#[test]
fn bash_example_variants() {
    let buf = bash_comptest("derive_show_asm --example co\t\t", false).unwrap();
    assert_eq!(
        buf,
        "%\nSelect artifact to use for analysis  coreutils                            comonad"
    );
    let buf = bash_comptest("derive_show_asm --example core\t", false).unwrap();

    assert_eq!(buf, "% derive_show_asm --example coreutils");
}
*/
