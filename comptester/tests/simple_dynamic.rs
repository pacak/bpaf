use comptester::*;
use pretty_assertions::assert_eq;

//let buf = zsh_comptest("derive_show_asm ^X?", false).unwrap();
#[test]
fn sd_all_options_zsh() {
    let buf = zsh_comptest("simple_dynamic --crate \t").unwrap();
    let expected = "% simple_dynamic --crate
NAME: Select crate for analysis
cargo-hackerman          -- Workspace hack management and package/feature query
cargo-prebuilt           -- Download prebuilt crate binaries
cargo-show-asm           -- Display generated assembly
cargo-supply-chain       -- Gather author, contributor, publisher data on crates
chezmoi_modify_manager   -- Chezmoi addon to patch ini files
xvf                      -- Easy archive extraction
newdoc                   -- Generate pre-populated module files
nust64                   -- Tools for compiling a Rust project into an N64 ROM
uggo                     -- CLI tool to query builds from u.gg";

    assert_eq!(buf, expected);
    //    let buf = zsh_comptest("simple_dynamic ?").unwrap();
    //    todo!("\n{}", buf);
}
