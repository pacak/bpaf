use comptester::*;
use pretty_assertions::assert_eq;

//let buf = zsh_comptest("derive_show_asm ?", false).unwrap();
#[test]
fn all_options_zsh() {
    let buf = zsh_comptest("derive_show_asm \t").unwrap();
    // let buf = zsh_comptest("derive_show_asm ?", false).unwrap();

    let expected = r"% derive_show_asm
--manifest-path=PATH     -- Path to Cargo.toml
--target-dir=DIR         -- Custom target directory for generated artifacts
--package=SPEC           -- Package to use if ambigous
--dry                    -- Produce a build plan instead of actually building
--frozen                 -- Requires Cargo.lock and cache are up to date
--locked                 -- Requires Cargo.lock is up to date
--offline                -- Run without accessing the network
--intel                  -- Generate assembly using Intel style
--att                    -- Generate assembly using AT&T style
Select artifact to use for analysis
--lib                    -- Show results from library code
--test=TEST              -- Show results from a test
--bench=BENCH            -- Show results from a benchmark
--example=EXAMPLE        -- Show results from an example
--bin=BIN                -- Show results from a binary
How to render output
--rust                   -- Print interleaved Rust code
--color                  -- Enable color highlighting
--no-color               -- Disable color highlighting
--full-name              -- include full demangled name instead of just prefix
Item to pick from the output
FUNCTION: Complete or partial function name to filter";
    assert_eq!(buf, expected);
}

#[test]
fn all_options_fish() {
    //    let buf = fish_comptest("derive_show_asm \t", true).unwrap();
    let buf = fish_comptest("derive_show_asm -\t").unwrap();

    let expected = "% derive_show_asm --att
--att                        (Generate assembly using AT&T style)
--bench                           (Show results from a benchmark)
--bin                                (Show results from a binary)
--color                               (Enable color highlighting)
--dry         (Produce a build plan instead of actually building)
--example                          (Show results from an example)
--frozen           (Requires Cargo.lock and cache are up to date)
--full-name  (include full demangled name instead of just prefix)
--intel                     (Generate assembly using Intel style)
--lib                            (Show results from library code)
--locked                      (Requires Cargo.lock is up to date)
--manifest-path                              (Path to Cargo.toml)
--no-color                           (Disable color highlighting)
--offline                     (Run without accessing the network)
--package                            (Package to use if ambigous)
--rust                              (Print interleaved Rust code)
--target-dir    (Custom target directory for generated artifacts)
--test                                 (Show results from a test)";
    assert_eq!(buf, expected);
}

#[test]
fn single_result_zsh() {
    let buf = zsh_comptest("derive_show_asm --li\t").unwrap();
    assert_eq!(buf, "% derive_show_asm --lib");
}

#[test]
fn single_result_fish() {
    let buf = fish_comptest("derive_show_asm --li\t").unwrap();
    assert_eq!(buf, "% derive_show_asm --lib");
}

#[test]
fn zsh_file_completion() {
    let buf = zsh_comptest("derive_show_asm --manifest-path \t").unwrap();
    assert_eq!(
        buf,
        "% derive_show_asm --manifest-path
Cargo.toml  src/        tests/"
    );

    let buf = zsh_comptest("derive_show_asm --manifest-path C\t").unwrap();
    assert_eq!(buf, "% derive_show_asm --manifest-path Cargo.toml");
}

#[test]
fn zsh_example_single() {
    let buf = zsh_comptest("derive_show_asm --example de\t").unwrap();
    assert_eq!(buf, "% derive_show_asm --example derive_show_asm");
}

#[test]
fn fish_example_single() {
    let buf = fish_comptest("derive_show_asm --example de\t").unwrap();
    assert_eq!(
        buf,
        "% derive_show_asm --example derive_show_asm derive_show_asm"
    );
}

#[test]
fn zsh_example_variants() {
    let buf = zsh_comptest("derive_show_asm --example co\t").unwrap();
    assert_eq!(
        buf,
        "% derive_show_asm --example co
Select artifact to use for analysis
coreutils
comonad"
    );
    let buf = zsh_comptest("derive_show_asm --example core\t").unwrap();

    assert_eq!(buf, "% derive_show_asm --example coreutils");
}

#[test]
fn fish_example_variants() {
    let buf = fish_comptest("derive_show_asm --example co\t").unwrap();
    assert_eq!(
        buf,
        "% derive_show_asm --example comonad
comonad  coreutils"
    );
    let buf = fish_comptest("derive_show_asm --example core\t").unwrap();

    assert_eq!(buf, "% derive_show_asm --example coreutils coreutils");
}

#[test]
fn single_result_bash() {
    let buf = bash_comptest("derive_show_asm --li\t").unwrap();
    assert_eq!(buf, "% derive_show_asm --lib");
}

#[test]
fn all_options_bash() {
    let buf = bash_comptest("derive_show_asm \t\t").unwrap();
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
--intel                  -- Generate assembly using Intel style
--att                    -- Generate assembly using AT&T style
Item to pick from the output
FUNCTION: Complete or partial function name to filter";
    assert_eq!(buf, expected);
}

#[test]
fn bash_file_completion() {
    let buf = bash_comptest("derive_show_asm --manifest-path \t\t").unwrap();
    // TODO - "FUNCTION" looks a bit wonky here...
    assert_eq!(buf, "%\nCargo.toml  src/        tests/");

    // file completion with mask in bash uses _filedir which
    // renders directories all the time
    let buf = bash_comptest("derive_show_asm --manifest-path Ca\t\t").unwrap();
    assert_eq!(buf, "%\nCargo.toml  src/        tests/");
}

#[test]
fn bash_example_single() {
    let buf = bash_comptest("derive_show_asm --example de\t").unwrap();
    assert_eq!(buf, "% derive_show_asm --example derive_show_asm");
}

#[test]
fn bash_example_variants() {
    let buf = bash_comptest("derive_show_asm --example co\t\t").unwrap();
    assert_eq!(
        buf,
        "%\nSelect artifact to use for analysis    coreutils
EXAMPLE: Show results from an example  comonad"
    );
    let buf = bash_comptest("derive_show_asm --example core\t").unwrap();

    assert_eq!(buf, "% derive_show_asm --example coreutils");
}
