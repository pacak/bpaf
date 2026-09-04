//! Compilation time benchmark target: a derive macro over a struct with 100 fields,
#![allow(dead_code)]

use std::num::ParseIntError;

use bpaf::*;

const COMPLETIONS: &[&str] = &["alpha", "beta"];
const COMPLETIONS2: &[&str] = &["a", "b"];

fn parse_num(s: String) -> Result<u32, ParseIntError> {
    s.parse()
}

fn alternative() -> impl Parser<Output = u32> {
    fail("missing")
}

fn num_to_string(v: &u32) -> String {
    v.to_string()
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
struct Opts {
    #[bpaf(long, argument("N"), fallback_str("42"), display_fallback)]
    f_0: u32,

    #[bpaf(long, argument("N"), fallback(42), debug_fallback)]
    f_1: u32,

    #[bpaf(long, argument("N"), guard(|v: &u32| *v < 100, "too big"), hide)]
    f_2: u32,

    #[bpaf(long, argument("N"), optional)]
    f_3: Option<u32>,

    #[bpaf(long, argument("N"), many, last)]
    f_4: Vec<u32>,

    #[bpaf(long, switch, count)]
    f_5: usize,

    #[bpaf(long, argument::<String>("N"), map(|s| s.len()))]
    f_6: usize,

    #[bpaf(long, argument::<String>("N"), parse(parse_num))]
    f_7: u32,

    #[bpaf(long, argument("N"), fallback_with(|| Ok::<_, String>(42u32)))]
    f_8: u32,

    #[bpaf(long, argument("N"), complete(COMPLETIONS), group_help("completable"))]
    f_9: String,

    #[bpaf(long, argument("N"), some("at least one"), guard(|v: &Vec<u32>| !v.is_empty(), "not empty"))]
    f_10: Vec<u32>,

    #[bpaf(long, argument("N"), collect)]
    f_11: Vec<u32>,

    #[bpaf(long, argument("N"), custom_usage("N VALUE"), fallback(0))]
    f_12: u32,

    #[bpaf(long, argument("N"), or_else(alternative), fallback(0))]
    f_13: u32,

    #[bpaf(long, argument("N"), hide_usage, fallback(0))]
    f_14: u32,

    #[bpaf(positional("P"), strict, fallback(0))]
    f_15: u32,

    #[bpaf(positional("P"), posix, fallback(0))]
    f_16: u32,

    #[bpaf(positional("P"), complete(COMPLETIONS2), fallback(String::new()))]
    f_17: String,

    #[bpaf(positional("P"), fallback(0))]
    f_18: u32,

    #[bpaf(long, argument("N"), fallback(0), format_fallback(num_to_string))]
    f_19: u32,

    #[bpaf(long, argument("N"), fallback_str("42"), display_fallback)]
    f_20: u32,

    #[bpaf(long, argument("N"), fallback(42), debug_fallback)]
    f_21: u32,

    #[bpaf(long, argument("N"), guard(|v: &u32| *v < 100, "too big"), hide)]
    f_22: u32,

    #[bpaf(long, argument("N"), optional)]
    f_23: Option<u32>,

    #[bpaf(long, argument("N"), many, last)]
    f_24: Vec<u32>,

    #[bpaf(long, switch, count)]
    f_25: usize,

    #[bpaf(long, argument::<String>("N"), map(|s| s.len()))]
    f_26: usize,

    #[bpaf(long, argument::<String>("N"), parse(parse_num))]
    f_27: u32,

    #[bpaf(long, argument("N"), fallback_with(|| Ok::<_, String>(42u32)))]
    f_28: u32,

    #[bpaf(long, argument("N"), complete(COMPLETIONS), group_help("completable"))]
    f_29: String,

    #[bpaf(long, argument("N"), some("at least one"), guard(|v: &Vec<u32>| !v.is_empty(), "not empty"))]
    f_30: Vec<u32>,

    #[bpaf(long, argument("N"), collect)]
    f_31: Vec<u32>,

    #[bpaf(long, argument("N"), custom_usage("N VALUE"), fallback(0))]
    f_32: u32,

    #[bpaf(long, argument("N"), or_else(alternative), fallback(0))]
    f_33: u32,

    #[bpaf(long, argument("N"), hide_usage, fallback(0))]
    f_34: u32,

    #[bpaf(positional("P"), strict, fallback(0))]
    f_35: u32,

    #[bpaf(positional("P"), posix, fallback(0))]
    f_36: u32,

    #[bpaf(positional("P"), complete(COMPLETIONS2), fallback(String::new()))]
    f_37: String,

    #[bpaf(positional("P"), fallback(0))]
    f_38: u32,

    #[bpaf(long, argument("N"), fallback(0), format_fallback(num_to_string))]
    f_39: u32,

    #[bpaf(long, argument("N"), fallback_str("42"), display_fallback)]
    f_40: u32,

    #[bpaf(long, argument("N"), fallback(42), debug_fallback)]
    f_41: u32,

    #[bpaf(long, argument("N"), guard(|v: &u32| *v < 100, "too big"), hide)]
    f_42: u32,

    #[bpaf(long, argument("N"), optional)]
    f_43: Option<u32>,

    #[bpaf(long, argument("N"), many, last)]
    f_44: Vec<u32>,

    #[bpaf(long, switch, count)]
    f_45: usize,

    #[bpaf(long, argument::<String>("N"), map(|s| s.len()))]
    f_46: usize,

    #[bpaf(long, argument::<String>("N"), parse(parse_num))]
    f_47: u32,

    #[bpaf(long, argument("N"), fallback_with(|| Ok::<_, String>(42u32)))]
    f_48: u32,

    #[bpaf(long, argument("N"), complete(COMPLETIONS), group_help("completable"))]
    f_49: String,

    #[bpaf(long, argument("N"), some("at least one"), guard(|v: &Vec<u32>| !v.is_empty(), "not empty"))]
    f_50: Vec<u32>,

    #[bpaf(long, argument("N"), collect)]
    f_51: Vec<u32>,

    #[bpaf(long, argument("N"), custom_usage("N VALUE"), fallback(0))]
    f_52: u32,

    #[bpaf(long, argument("N"), or_else(alternative), fallback(0))]
    f_53: u32,

    #[bpaf(long, argument("N"), hide_usage, fallback(0))]
    f_54: u32,

    #[bpaf(positional("P"), strict, fallback(0))]
    f_55: u32,

    #[bpaf(positional("P"), posix, fallback(0))]
    f_56: u32,

    #[bpaf(positional("P"), complete(COMPLETIONS2), fallback(String::new()))]
    f_57: String,

    #[bpaf(positional("P"), fallback(0))]
    f_58: u32,

    #[bpaf(long, argument("N"), fallback(0), format_fallback(num_to_string))]
    f_59: u32,

    #[bpaf(long, argument("N"), fallback_str("42"), display_fallback)]
    f_60: u32,

    #[bpaf(long, argument("N"), fallback(42), debug_fallback)]
    f_61: u32,

    #[bpaf(long, argument("N"), guard(|v: &u32| *v < 100, "too big"), hide)]
    f_62: u32,

    #[bpaf(long, argument("N"), optional)]
    f_63: Option<u32>,

    #[bpaf(long, argument("N"), many, last)]
    f_64: Vec<u32>,

    #[bpaf(long, switch, count)]
    f_65: usize,

    #[bpaf(long, argument::<String>("N"), map(|s| s.len()))]
    f_66: usize,

    #[bpaf(long, argument::<String>("N"), parse(parse_num))]
    f_67: u32,

    #[bpaf(long, argument("N"), fallback_with(|| Ok::<_, String>(42u32)))]
    f_68: u32,

    #[bpaf(long, argument("N"), complete(COMPLETIONS), group_help("completable"))]
    f_69: String,

    #[bpaf(long, argument("N"), some("at least one"), guard(|v: &Vec<u32>| !v.is_empty(), "not empty"))]
    f_70: Vec<u32>,

    #[bpaf(long, argument("N"), collect)]
    f_71: Vec<u32>,

    #[bpaf(long, argument("N"), custom_usage("N VALUE"), fallback(0))]
    f_72: u32,

    #[bpaf(long, argument("N"), or_else(alternative), fallback(0))]
    f_73: u32,

    #[bpaf(long, argument("N"), hide_usage, fallback(0))]
    f_74: u32,

    #[bpaf(positional("P"), strict, fallback(0))]
    f_75: u32,

    #[bpaf(positional("P"), posix, fallback(0))]
    f_76: u32,

    #[bpaf(positional("P"), complete(COMPLETIONS2), fallback(String::new()))]
    f_77: String,

    #[bpaf(positional("P"), fallback(0))]
    f_78: u32,

    #[bpaf(long, argument("N"), fallback(0), format_fallback(num_to_string))]
    f_79: u32,

    #[bpaf(long, argument("N"), fallback_str("42"), display_fallback)]
    f_80: u32,

    #[bpaf(long, argument("N"), fallback(42), debug_fallback)]
    f_81: u32,

    #[bpaf(long, argument("N"), guard(|v: &u32| *v < 100, "too big"), hide)]
    f_82: u32,

    #[bpaf(long, argument("N"), optional)]
    f_83: Option<u32>,

    #[bpaf(long, argument("N"), many, last)]
    f_84: Vec<u32>,

    #[bpaf(long, switch, count)]
    f_85: usize,

    #[bpaf(long, argument::<String>("N"), map(|s| s.len()))]
    f_86: usize,

    #[bpaf(long, argument::<String>("N"), parse(parse_num))]
    f_87: u32,

    #[bpaf(long, argument("N"), fallback_with(|| Ok::<_, String>(42u32)))]
    f_88: u32,

    #[bpaf(long, argument("N"), complete(COMPLETIONS), group_help("completable"))]
    f_89: String,

    #[bpaf(long, argument("N"), some("at least one"), guard(|v: &Vec<u32>| !v.is_empty(), "not empty"))]
    f_90: Vec<u32>,

    #[bpaf(long, argument("N"), collect)]
    f_91: Vec<u32>,

    #[bpaf(long, argument("N"), custom_usage("N VALUE"), fallback(0))]
    f_92: u32,

    #[bpaf(long, argument("N"), or_else(alternative), fallback(0))]
    f_93: u32,

    #[bpaf(long, argument("N"), hide_usage, fallback(0))]
    f_94: u32,

    #[bpaf(positional("P"), strict, fallback(0))]
    f_95: u32,

    #[bpaf(positional("P"), posix, fallback(0))]
    f_96: u32,

    #[bpaf(positional("P"), complete(COMPLETIONS2), fallback(String::new()))]
    f_97: String,

    #[bpaf(positional("P"), fallback(0))]
    f_98: u32,

    #[bpaf(long, argument("N"), fallback(0), format_fallback(num_to_string))]
    f_99: u32,
}

#[test]
fn compiles() {
    let _parser = opts();
}
