searchState.loadedDescShard("bpaf", 0, "Lightweight and flexible command line argument parser with …\nAll currently present command line parameters with some …\nDerive macro for bpaf command line parser\nThis also goes to stdout with exit code of 0, this cannot …\nSimilar to <code>File</code> but limited to directories only For bash …\nString with styled segments.\nA file or directory name with an optional file mask.\nDon’t produce anything at all from this parser - can be …\nReady to run <code>Parser</code> with additional information attached\nUnsuccessful command line parsing outcome, use it for unit …\nSimple or composed argument parser\nYou can also specify a raw value to use for each supported …\nShell specific completion\nPrint this to stderr and exit with failure code\nPrint this to stdout and exit with success code\nProject documentation\nParse a single arbitrary item from a command line\nBatteries included - helpful parsers that use only public …\nCreate a boxed representation for a parser\nCheck the invariants <code>bpaf</code> relies on for normal operations\nChoose between several parsers specified at runtime\nTransform parser into a collection parser\nParse a subcommand\nDynamic shell completion\nStatic shell completion\nCompose several parsers to produce a single result\nCount how many times the inner parser succeeds, and return …\nGet a list of command line arguments from OS\nCustomize how this parser looks like in the usage line\nSet the description field\nDocumentation generation system\nParse an environment variable\nReturns the exit code for the failure\nFail with a fixed error message\nUse this value as default if the value isn’t present on …\nPrint help if app was called with no parameters\nUse value produced by this function as default if the …\nSet the footer field\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nAttach a help message to a complex parser\nValidate or fail with a message\nSet the header field\nCustomize parser for <code>--help</code>\nIgnore this parser during any sort of help generation\nIgnore this parser when generating a usage line\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nApply the inner parser as many times as it succeeds, …\nA specialized version of <code>any</code> that consumes an arbitrary …\nParse a <code>flag</code>/<code>switch</code>/<code>argument</code> that has a long name\nConsume zero or more items from a command line and collect …\nApply a pure transformation to a contained value\nSet the width of the help message printed to the terminal …\nTurn a required argument into an optional one\nTools to define primitive parsers\nApply a failing transformation to a contained value\nThis module exposes parsers that accept further …\nParse a positional argument\nPrints a message to <code>stdout</code> or <code>stderr</code> appropriate to the …\nParser that produces a fixed value\nWrap a calculated value into a <code>Parser</code>\nRender command line documentation for the app into …\nRender command line documentation for the app into a …\nRender command line documentation for the app into Markdown\nExecute the <code>OptionParser</code>, extract a parsed value or print …\nFinalize and run the parser\nExecute the <code>OptionParser</code> and produce a values for unit …\nEnable completions with custom output revision style\nAdd an application name for args created from custom input\nParse a <code>flag</code>/<code>switch</code>/<code>argument</code> that has a short name\nConsume one or more items from a command line and collect …\nTransform <code>Parser</code> into <code>OptionParser</code> to get ready to <code>run</code> it\nExecute the <code>OptionParser</code>, extract a parsed value or return …\nReturns the contained <code>stderr</code> values - for unit tests\nReturns the contained <code>stdout</code> values - for unit tests\nSet custom usage field\nSet the version field.\nCustomize parser for <code>--version</code>\nMake a help message for a complex parser from its <code>MetaInfo</code>\nGenerate new usage line using automatically derived usage\nThis raw string will be used for <code>bash</code> shell …\nThis raw string will be used for <code>elvish</code> shell …\nThis raw string will be used for <code>fish</code> shell …\nOptional filemask to use, no spaces, no tabs\nOptional filemask to use, no spaces, no tabs\nThis raw string will be used for <code>zsh</code> shell …\n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \nStrip a command name if present at the front when used as …\nGet usage for a parser\nPick last passed value between two different flags\n<code>--verbose</code> and <code>--quiet</code> flags with results encoded as number\n<code>--verbose</code> and <code>--quiet</code> flags with results choosen from a …\nCustom section\nString with styled segments.\nWord with emphasis - things like “Usage”, “Available …\nFile formats and conventions\nGames and screensavers\nGeneral commands\nInvalid input given by user - used to display invalid …\nLibrary functions such as C standard library functions\nSomething user needs to type literally - command names, etc\nParser metainformation\nMetavavar placeholder - something user needs to replace …\nMiscellaneous\nManual page section\nSpecial files (usually devices in /dev) and drivers\nStyle of a text fragment inside of <code>Doc</code>\nSystem administration commands and daemons\nSystem calls\nPlain text, no decorations\nAppend a <code>Doc</code> to <code>Doc</code>\nAppend a <code>Doc</code> to <code>Doc</code> for plaintext documents try to format …\nAppend a fragment of text with emphasis to <code>Doc</code>\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nAppend a fragment of unexpected user input to <code>Doc</code>\nAppend a fragment of literal text to <code>Doc</code>\nAppend a fragment of parser metadata to <code>Doc</code>\nRender a monochrome version of the document\nRender doc into markdown document, used by documentation …\nAppend a fragment of plain text to <code>Doc</code>\nA named thing used to create <code>flag</code>, <code>switch</code> or <code>argument</code>\nConsume an arbitrary value that satisfies a condition, …\nParser for a named argument, created with <code>argument</code>.\nBuilder structure for the <code>command</code>\nParser for a named switch, created with <code>NamedArg::flag</code> or …\nParse a positional item, created with <code>positional</code>\nAllow for the command to succeed even if there are non …\nRestrict parsed arguments to have both flag and a value in …\nTry to apply the parser to each unconsumed element instead …\nArgument\nEnvironment variable fallback\nFlag with custom present/absent values\nAdd a brief description to a command\nAdd a help message to <code>any</code> parser. See examples in <code>any</code>\nAdd a help message to a <code>flag</code>/<code>switch</code>/<code>argument</code>\nAdd a help message to <code>flag</code>\nAdd a help message to a <code>positional</code> parser\nAdd a custom hidden long alias for a command\nAdd a long name to a flag/switch/argument\nReplace metavar with a custom value See examples in <code>any</code>\nChanges positional parser to be a “not strict” …\nRequired flag with custom value\nAdd a custom short alias for a command\nAdd a short name to a flag/switch/argument\nChanges positional parser to be a “strict” positional\nSimple boolean flag\nA named thing used to create <code>flag</code>, <code>switch</code> or <code>argument</code>\nConsume an arbitrary value that satisfies a condition, …\nParser for a named argument, created with <code>argument</code>.\nApply inner parser several times and collect results into …\nBuilder structure for the <code>command</code>\nParser that inserts static shell completion into bpaf’s …\nCreate parser from a function, <code>construct!</code> uses it …\nApply inner parser as many times as it succeeds while …\nParser that substitutes missing value but not parse …\nParser that substitutes missing value with a function …\nParser for a named switch, created with <code>NamedArg::flag</code> or …\nApply inner parser as many times as it succeeds while …\nApply inner parser several times and collect results into …\nApply inner parser, return a value in <code>Some</code> if items …\nParse a positional item, created with <code>positional</code>\nApply inner parser several times and collect results into …\nAutomagically restrict the inner parser scope to accept …\nHandle parse failures\nHandle parse failures\nHandle parse failures for optional parsers\nHandle parse failures\nShow <code>fallback_with</code> value in <code>--help</code> using <code>Debug</code> …\nShow <code>fallback</code> value in <code>--help</code> using <code>Debug</code> representation\nShow <code>fallback_with</code> value in <code>--help</code> using <code>Display</code> …\nShow <code>fallback</code> value in <code>--help</code> using <code>Display</code> representation\nTo produce a better error messages while parsing …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nAdd a help message to an <code>argument</code>\ninner parser closure\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nmetas for inner parsers")