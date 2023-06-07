use syn::{PathArguments, Type};

pub(crate) use crate::named_field::StructField;

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum Shape {
    /// Option<T>
    Optional(Type),
    /// Vec<T>,
    Multiple(Type),
    /// bool
    Bool,
    /// ()
    Unit,
    /// T
    Direct(Type),
}

pub(crate) fn split_type(ty: &Type) -> Shape {
    fn single_arg(x: &PathArguments) -> Option<Type> {
        match x {
            PathArguments::AngleBracketed(arg) => {
                if arg.args.len() == 1 {
                    match &arg.args[0] {
                        syn::GenericArgument::Type(ty) => Some(ty.clone()),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            PathArguments::None | PathArguments::Parenthesized(_) => None,
        }
    }

    fn try_split_type(ty: &Type) -> Option<Shape> {
        if let Type::Tuple(syn::TypeTuple { elems, .. }) = ty {
            if elems.is_empty() {
                return Some(Shape::Unit);
            }
        }

        let last = match ty {
            Type::Path(p) => p.path.segments.last()?,
            _ => return None,
        };
        if last.ident == "Vec" {
            Some(Shape::Multiple(single_arg(&last.arguments)?))
        } else if last.ident == "Option" {
            Some(Shape::Optional(single_arg(&last.arguments)?))
        } else if last.ident == "bool" {
            Some(Shape::Bool)
        } else {
            None
        }
    }
    try_split_type(ty).unwrap_or_else(|| Shape::Direct(ty.clone()))
}
