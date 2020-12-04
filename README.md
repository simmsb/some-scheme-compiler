# some-scheme-compiler

A scheme compiler that uses a Continuation Passing Style transformation.

It's similar to cyclone and chicken scheme in that we use the
C stack as a nursery 'heap', when the stack is exhausted we
migrate everything to the actual heap, then reset the stack.

# Compiling and running

```
somescheme 0.1.0

USAGE:
    some-scheme-compiler [FLAGS] [OPTIONS] <SUBCOMMAND>

FLAGS:
    -d, --debug       
    -h, --help        Prints help information
    -k, --keep-tmp    
    -V, --version     Prints version information

OPTIONS:
    -i, --input <input>    

SUBCOMMANDS:
    compile    Compile the program
    help       Prints this message or the help of the given subcommand(s)
    run        Run the progam
```
