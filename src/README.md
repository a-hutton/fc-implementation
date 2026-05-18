# An Implementation of FC

> Note: The main branch is identical to the branch `groundup-lcp-join-nc-heuristics`.

This repository contains all the implementations created for the Part D Project "Implementing the Text Querying Language
FC".
Each implementation was created on a separate branch, so they can be compiled individually and tested against
each-other, while keeping the code organised and in one place.

# Usage

The compiled executable file can be run `$ ./fc <options>`.

The available options:

| Full Option      | Shorthand | argument       | Description                                                                                                                                                                                                                                                                                             |
|------------------|-----------|----------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--help`         | `-h`      |                | Print the help text, showing program usage                                                                                                                                                                                                                                                              |
| `--command`      | `-c`      | `<COMMAND>`    | Change the operation of the program. The available options are `find-solutions` (default), which finds all satisfying assignments for the given formula and word; `check-assignment`, which checks if a given assignment is satisfying; `generate-factors`, which generates all factors of a given word |
| `--pattern`      | `-p`      | `<FORMULA>`    | The formula passed to the solver                                                                                                                                                                                                                                                                        |
| `--text`         | `-t`      | `<TEXT>`       | The word used by the solver. Mutually exclusive with `--file`                                                                                                                                                                                                                                           |
| `--file`         | `-f`      | `<FILE>`       | The path to a text file, whose contents will be used as the universe word. Mutually exclusive with `--text`                                                                                                                                                                                             |
| `--assignment`   | `-a`      | `<ASSIGNMENT>` | A string representing a variable assignment, used when `--command`=`check-assignment`. In the format `<VAR>:<WORD>`. This should be passed multiple times to assign values to multiple variables (e.g. `-a 'x:abc' -a 'y:def'`)                                                                         |
| `--quiet`        | `-q`      |                | Operates the program in 'quiet' mode, where limited outputs are printed to the console (e.g. the number of assignments, rather than the whole list)                                                                                                                                                     |
| `--column-width` |           | `<VALUE>`      | Limits the width of each column of assignments when printing to the console (default = 30)                                                                                                                                                                                                              |

## Example

The program can be used to find if the word `abcdabcd` is a square:

```bash
$ ./fc --pattern '$U = x x' --text 'abcdabcd'
Found 1 solutions
x                             
abcd   
```

And to find that the word `abcdef` is not a square:

```bash
$ ./fc --pattern '$U = x x' --text 'abcdef'
Found 0 solutions
No satisfying assignments found
```

s

# Formula Syntax

## Variables

A variable name can have upper- and lower-case letters and digits, and must start with a letter

## String Literals / Words

A string literal in a formula is a string surrounded by double quotes `"`.
These quotes can be escaped in literals by prefixing them with a backslash `\"`, but **the universe word must also have
the quotes expected escaped for them to match**.

## Word Equations

A word equation is a single variable, then an ASCII 'equals' character `=`, then a pattern of variables and string
literals.
Variables must be separated by spaces (otherwise the parser will consider them as a single variable).
Other than that case, spaces are optional (i.e. `x = y "ab"  z` and `x=y"ab"z` are equivalent, but `x = y z` and `x=yz`
are not).

## Conjunctions and Disjunctions

Sub-formulas are combined using logical 'And' and 'Or' operations, using either typical programming notation `&&` and
`||`, or the Unicode characters representing the logical symbols `∧` (U+2227) and `∨` (U+2228).

E.g. `Z = X "ab" Y ∧ Z = Y "ab" X`

## Negations

Sub-formulas are negated by using the symbol `¬` (U+00AC).
This binds to the nearest single sub-formula, and therefore parentheses should be used to apply negation to more than a
single equation or quantified expression.

E.g. `¬x="aa" || x="a"` can assign `x` the value `a`, but `¬(x="aa" || x="a")` cannot.

## Quantifiers

To bind a variable to an existential or universal quantifier, use either the keywords `exists` and `forall` or, similar
to conjunctions and disjunctions, the Unicode characters `∀`(U+2200) `∃` (U+2203).
The syntax is `<"forall" | ∀ | "exists" | ∃> <VAR> <SUB FORMULA>`.
Again, only the nearest single sub-formula is applied to the quantifier, so parentheses are required if applying to more
than a single equation, negation, or quantified expression.

# Building the Project

Assuming that Rustup and Cargo are installed on your system (https://rust-lang.org/learn/get-started/), the project can
be built using the `cargo` tool.

```bash
cargo build --release
```

The executable is in the `./target/release/` directory.