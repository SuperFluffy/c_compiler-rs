# Writing a C compiler in Rust

This is to learn how to a) write a C compiler, and b) how to do it completely in Rust.

This follows the [Write a Compiler] tutorial by [Nora Sandler].

[Write a Compiler]: https://norasandler.com/2017/11/29/Write-a-Compiler.html
[Nora Sandler]: https://norasandler.com

## Design choices

At the moment, only a lexer exists. It is implemented using a finite state machine, which
the `Tokenizer` threads through when tokenizing an input stream of characters.
