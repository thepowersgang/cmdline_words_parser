A rust crate to do inline parsing of literal command line strings (handling escape codes and splitting at whitespace)

Performs the parsing in the source string, to avoid intermediate allocations (suitable for memory-constrained environments).
