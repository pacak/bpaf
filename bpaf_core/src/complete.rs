use std::{borrow::Cow, ffi::OsString, rc::Rc};

use crate::{Error, Metavar, Name, Named, utils::Vec1};
impl From<CompleteReply> for Error {
    fn from(value: CompleteReply) -> Self {
        Error::CompleteReply(Vec1::new(value))
    }
}

// this could be a method on `Command<T>`, but will it monomorphise?
pub(crate) fn complete_command(names: &[Cow<'static, str>], err: Error) -> Error {
    let Error::CompleteRequest(ref comp) = err else {
        return err;
    };
    let prefix = match comp {
        CompleteReq::Anything => None,
        CompleteReq::Literal { prefix } => Some(prefix.as_ref()),
        CompleteReq::Name { .. } | CompleteReq::Value(..) => return err,
    };
    for name in names {
        if prefix.is_none_or(|p| name.starts_with(p)) {
            return CompleteReply::Command {
                name: name.clone(),
                help: None,
            }
            .into();
        }
    }
    Error::Killed
}

pub(crate) fn complete_value(
    err: Error,
    group: Option<&str>,
    completer: &Box<dyn Fn(&str) -> Vec<(String, Option<String>)>>,
) -> Error {
    let Error::CompleteRequest(ref comp) = err else {
        return err;
    };
    let key = match comp {
        CompleteReq::Anything => "",
        CompleteReq::Literal { prefix } => prefix.as_ref(),
        CompleteReq::Value(..) | CompleteReq::Name { .. } => return err,
    };

    let group = group.map(|s| s.into());
    let values = completer(key)
        .into_iter()
        .map(|(value, hint)| {
            let group = group.clone();
            CompleteReply::Value { group, value, hint }
        })
        .collect::<Vec<_>>();
    Error::CompleteReply(values.into())
}

impl Named {
    pub(crate) fn complete_name(&self, err: Error, meta: Option<Metavar>) -> Error {
        let Error::CompleteRequest(ref comp) = err else {
            return err;
        };
        let mut name = None;
        match comp {
            CompleteReq::Name { prefix: None } | CompleteReq::Anything => {
                // any name with a preference to long
                for n in self.names.iter() {
                    match n {
                        Name::Short(_) => {
                            if name.is_none() {
                                name = Some(n);
                            }
                        }
                        Name::Long(_) => {
                            name = Some(n);
                            break;
                        }
                    }
                }
            }
            CompleteReq::Name {
                prefix: Some(prefix),
            } => {
                // long that starts with a given prefix
                for n in self.names.iter() {
                    if let Name::Long(s) = n
                        && s.starts_with(prefix.as_ref())
                    {
                        name = Some(n);
                        break;
                    }
                }
            }

            CompleteReq::Literal { .. } | CompleteReq::Value(..) => return err,
        };
        if let Some(name) = name.cloned() {
            let help = self.help.clone();
            CompleteReply::Named { name, meta, help }.into()
        } else {
            Error::Killed
        }
    }
}

#[derive(Debug, Clone, Ord, Eq, PartialEq, PartialOrd)]
pub(crate) enum CompleteReply {
    Command {
        name: Cow<'static, str>,
        help: Option<String>,
    },
    Named {
        name: Name<'static>,
        meta: Option<Metavar>,
        help: Option<String>,
    },
    Value {
        group: Option<Rc<str>>,
        value: String,
        hint: Option<String>,
    },
}

#[derive(Debug, Clone, Ord, Eq, PartialEq, PartialOrd)]
pub(crate) enum CompleteReq {
    /// We are trying to complete a name, `Some` indicates a long name, `None` - a short name
    /// where user typed just `-`
    Name {
        prefix: Option<Rc<str>>,
    },

    //
    Anything,
    Literal {
        prefix: Rc<str>,
    },
    Value(OsString),
}

pub(crate) fn render_completions(items: Vec1<CompleteReply>) -> String {
    render_completions_int(items).unwrap()
}
pub(crate) fn render_completions_int(
    mut items: Vec1<CompleteReply>,
) -> Result<String, std::fmt::Error> {
    use std::fmt::Write;
    items.sort();
    let mut out = String::new();
    for item in items.as_slice() {
        match item {
            CompleteReply::Named { name, meta, help } => match meta {
                Some(m) => write!(&mut out, "{name} {m:?} ({help:?})")?,
                None => write!(&mut out, "{name} ({help:?})")?,
            },
            CompleteReply::Value { value, group, hint } => write!(&mut out, "{value} ({hint:?}")?,
            CompleteReply::Command { name, help } => write!(&mut out, "{name} ({help:?})")?,
        }
    }
    Ok(out)
}
