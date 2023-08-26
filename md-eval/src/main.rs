// docs needs to combine both docs2 and documentation
//
// bpaf documentation needs to include source code as well as results of running this code on a
// command line. For example documentation for `guard` might include combinatoric and derive
// examples plus output from both. Something needs to test if output matches if there are multiple
// examples.
//
// documentation can be nested
//
// ways to make it work:
//
// 1. documentation is living along with rust source and is extracted with syn during
//    compilation time - seems to hard to get working with nesting
//
// 2. documentation is separated from sources in .md files, can be nested to form structure for
//    _documentation

use bpaf::*;
use md_eval::*;
use std::path::PathBuf;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
struct Options {
    #[bpaf(short, long)]
    pretty: bool,
    //    completion: bool,
    #[bpaf(positional)]
    target: PathBuf,
}

fn pretty_print(rendered: &str) -> anyhow::Result<String> {
    let parsed = syn::parse_file(rendered)?;
    Ok(prettyplease::unparse(&parsed))
}

fn process(path: &std::path::Path) -> anyhow::Result<()> {
    let doc = md_eval::md::Document::load(path)?;
    todo!("{}", pretty_print(&doc.render_rust("asdf".as_ref())?.code)?);

    //    use pulldown_cmark::*;
    //    use pulldown_cmark_to_cmark::*;

    let data = std::fs::read_to_string(path)?;
    let parser = pulldown_cmark::Parser::new(&data);

    let mut out = String::new();

    pulldown_cmark_to_cmark::cmark(
        parser.map(|x| {
            println!("{:?}", x);
            x
        }),
        &mut out,
    )
    .unwrap();

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let opts = options().fallback_to_usage().run();

    process(&opts.target)?;

    /*
        //    let arena = Default::default();
        let x = construct_module(&opts.target)?.code;
        match pretty_print(&x) {
            Ok(x) => println!("{x}"),
            Err(_) => println!("{x}"),
        }
    */
    /*
        let module = import_module(&opts.target)?;

        if opts.pretty {
            let rendered = module.to_string();
            let parsed = syn::parse_file(&rendered)?;
            let module = prettyplease::unparse(&parsed);
            println!("{module}");
        } else {
            println!("{module}");
        }
    */
    /*
    let md = render_module(
        &opts.target,
        &["results are here".into(), "x".into(), "asdf".into()],
    )?;
    //    let md = pretty_print(&md)?;
    println!("{md}");
    println!("{:?}", opts.target);*/

    Ok(())
}
