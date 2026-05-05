This example covers most of the available completions as well as how they are
being rendered in different shells.

First of all we get a list of all the commands - top level
```console
zsh% complete_tests <TAB>
zsh% complete_tests
arg-comp  -- Argument value completion
commands  -- A set of conflicting commands
flag-prod  -- A set of non-conflicting short and long flags
flag-sum  -- A set of conflicting short and long flags
pos-comp  -- Positional value completion
```

The top command should complete up to common prefix
```console
zsh% complete_tests fl<TAB><TAB>
zsh% complete_tests flag-
flag-prod  -- A set of non-conflicting short and long flags
flag-sum  -- A set of conflicting short and long flags
```

And once there's enough to identify it - to the end
```console
zsh% complete_tests flag-s<TAB>
zsh% complete_tests flag-sum
```

A list of flags. Double tab so it lists the descriptions and starts populating them
```console
zsh% complete_tests flag-sum -<TAB><TAB>
zsh% complete_tests flag-sum --alpha
--alpha  -- Alpha: α
--beta  -- Beta: β
--gamma  -- Gamma: γ
-z  -- Zetta: ζ
```

A list of flags. Double tab so it lists the descriptions and starts populating them, no single dash
```console
zsh% complete_tests flag-sum --<TAB><TAB><TAB>
zsh% complete_tests flag-sum --beta
--alpha  -- Alpha: α
--beta  -- Beta: β
--gamma  -- Gamma: γ
```


Should complete the flag name
```console
zsh% complete_tests flag-sum --a<TAB>
zsh% complete_tests flag-sum --alpha
```

And there's no other flags since it's a sum
```console
zsh% complete_tests flag-sum --alpha <TAB><TAB>
zsh% complete_tests flag-sum --alpha
```


Should complete the flag name
```console
zsh% complete_tests flag-prod --a<TAB>
zsh% complete_tests flag-prod --alpha
```

but there are other flags for product
```console
zsh% complete_tests flag-prod --alpha <TAB><TAB>
zsh% complete_tests flag-prod --alpha -
--beta  -- Beta: β
--gamma  -- Gamma: γ
-z  -- Zetta: ζ
```

Commands: list them by default
```console
zsh% complete_tests commands <TAB>
zsh% complete_tests commands
alpha  -- Alpha: α
bak-kut-teh  -- Bak Kut Teh: 肉骨茶
beta  -- Beta: β
gamma  -- Gamma: γ
```

Commands: show common prefix
```console
zsh% complete_tests commands b<TAB><TAB>
zsh% complete_tests commands bak-kut-teh
bak-kut-teh  -- Bak Kut Teh: 肉骨茶
beta  -- Beta: β
```

Commands: complete the only variant
```console
zsh% complete_tests commands a<TAB>
zsh% complete_tests commands alpha
```

Positional:
```console
zsh% complete_tests pos-comp <TAB>
zsh% complete_tests pos-comp
Alice  -- Sends a message
Bob  -- Receives a message
Carlos  -- A different unrelated third party
Carol  -- Unrelated third party
Grace  -- Government representative
```

Positional:
```console
zsh% complete_tests pos-comp C<TAB><TAB>
zsh% complete_tests pos-comp Car
Carlos  -- A different unrelated third party
Carol  -- Unrelated third party
```

Positional:
```console
zsh% complete_tests pos-comp A<TAB>
zsh% complete_tests pos-comp Alice
```
