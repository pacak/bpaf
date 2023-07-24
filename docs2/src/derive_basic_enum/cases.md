Help message reflects mutually exclusive parts

> --help

At least one branch needs to succeed

>

And in this example only one branch can succeed

> --name Cargo.toml

> --url https://crates.io --auth-method digest

While both branches can succeed at once - only one will actually succeed and afetr that
parsing fails since there are unconsumed items

> --url https://crates.io --auth-method digest --name Cargo.toml
