//! This module contains tests for scenarios when we want to wake up multiple parsers on the same
//! trigger, potentially from several different pecking sets.

use crate::*;

macro_rules! arranged {
    (($a:ident $ab:tt $b:ident) $abcd:tt ($c:ident $cd:tt $d:ident)) => {
        arrange(
            [$a.clone(), $b.clone(), $c.clone(), $d.clone()],
            arranged!($abcd),
            arranged!($ab),
            arranged!($cd),
        )
    };
    (+) => {
        Kind::Sum
    };
    (*) => {
        Kind::Prod
    };
}

fn run_arranged(
    parser: impl Parser<[Result<bool, Error>; 4]> + 'static,
    input: &[&str],
) -> [bool; 4] {
    let r = run(parser, input).unwrap();
    let v = r
        .into_iter()
        .map(|v| v.unwrap_or(false))
        .collect::<Vec<_>>();
    v.try_into().unwrap()
}

#[test]
fn short_flags_only() {
    let a = short('a').switch().into_rc();

    assert_eq!(
        run_arranged(arranged!((a * a) * (a * a)), &["-a"]),
        [true, false, false, false]
    );

    assert_eq!(
        run_arranged(arranged!((a + a) + (a + a)), &["-a"]),
        [true, true, true, true]
    );

    assert_eq!(
        run_arranged(arranged!((a * a) + (a + a)), &["-a"]),
        [true, false, true, true]
    );

    assert_eq!(
        run_arranged(arranged!((a + a) + (a * a)), &["-a"]),
        [true, true, true, false]
    );

    assert_eq!(
        run_arranged(arranged!((a + a) * (a + a)), &["-a"]),
        [true, true, false, false]
    );
}

fn arrange<T: 'static>(
    ps: [Bp<RcParser<T>>; 4],
    kind_1234: Kind,
    kind_12: Kind,
    kind_34: Kind,
) -> impl Parser<[Result<T, Error>; 4]> {
    #![allow(clippy::type_complexity)]

    let run: Box<dyn Fn(Ctx) -> Box<dyn FnOnce() -> Result<[Result<T, Error>; 4], Error>>> =
        Box::new(move |ctx: Ctx| {
            let cur_kind = ctx.current_task.borrow().parent_kind;
            let info1234 = ctx.make_child_info(cur_kind);
            let info12 = ctx.make_child_info(kind_1234);
            let info34 = ctx.make_child_info(kind_1234);

            let (ha, act) = ctx.make_raw_task(ps[0].clone());
            let mut info = ctx.make_child_info(kind_12);
            info.parent_id = info12.id.as_parent();
            ctx.add_task(Task { act, info });

            let (hb, act) = ctx.make_raw_task(ps[1].clone());
            let mut info = ctx.make_child_info(kind_12);
            info.parent_id = info12.id.as_parent();
            ctx.add_task(Task { act, info });

            let (hc, act) = ctx.make_raw_task(ps[2].clone());
            let mut info = ctx.make_child_info(kind_34);
            info.parent_id = info34.id.as_parent();
            ctx.add_task(Task { act, info });

            let (hd, act) = ctx.make_raw_task(ps[3].clone());
            let mut info = ctx.make_child_info(kind_34);
            info.parent_id = info34.id.as_parent();
            ctx.add_task(Task { act, info });

            ctx.add_task(Task {
                act: Box::pin(async { r#yield().await }),
                info: TaskInfo {
                    pending: 2,
                    parent_id: info1234.id.as_parent(),
                    ..info12
                },
            });

            ctx.add_task(Task {
                act: Box::pin(async { r#yield().await }),
                info: TaskInfo {
                    pending: 2,
                    parent_id: info1234.id.as_parent(),
                    ..info34
                },
            });

            ctx.add_task(Task {
                act: Box::pin(async { r#yield().await }),
                info: TaskInfo {
                    pending: 2,
                    ..info1234
                },
            });
            ctx.current_task.borrow_mut().pending -= 6;

            Box::new(|| Ok([ha.take(), hb.take(), hc.take(), hd.take()]))
        });
    Con { run }
}
