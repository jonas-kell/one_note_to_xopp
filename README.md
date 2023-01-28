## OneNote to XOPP converter

This is an open source program, that converts the .one file format to .xopp.

I only implemented this, because my Microsoft Surface broke and my other laptop uses Linux, so I can no longer use OneNote.
As this is only for my own limited use, NOT everything (not even most things) can be successfully converted.

The converter ONLY supports `ink/pen drawings` and `images` at the moment, nothing more.
There is no plan from me to ever implement more, as I have never used other features of OneNote.

The implementation makes heavy use of [Markus Siemens](https://github.com/msiemens) work in the field of OneNote file parsing.
The code uses his package [Rust OneNote® File Parser](https://github.com/msiemens/onenote.rs) and my code is heavily "inspired" by his [one2html](https://github.com/msiemens/one2html) project, only that mine has way worse support for elements and converts to [Xournal++](https://xournalpp.github.io/) instead of html. I suggest looking at his project first, I only made this public for the case case it could benefit someone, so please do not expect support.

## Installation

At the moment only compiling from source is supported.
This requires the latest stable [Rust](https://www.rust-lang.org/) compiler. Once you've installed the Rust toolchain run:

```cmd
git clone https://github.com/jonas-kell/one_note_to_xopp
cd one_note_to_xopp
cargo build --release
```

## Usage

When the produced executable is run inside a folder it will take all the `.one` files from that folder as inputs and spit out the converted `.xopp` files into the same folder.

```cmd
 ./target/release/one_note_to_xopp
```

(Of course this is the execution syntax for linux, that assumes you are still in the git repository. You can move the compiled executable into any folder you like, even into folders in `Path`, to get systemwide access. Append `.exe` for Windows).

## CLI Arguments

The behavior can be adapted with CLI Arguments. They are documented when running

```cmd
 ./target/release/one_note_to_xopp --help
```

## Where to get my .one files

To acquire the .one files from cour OneDrive, take a look [here](https://github.com/msiemens/one2html#usage).
It is a good idea to backup them anyway, because who knows how reliable Cloud-Storage is.

## Disclaimer

This project is neither related to nor endorsed by Microsoft in any way. The author does not have any affiliation with Microsoft.

I have never used Rust before, so my code is probably shit, it is not commented and entirely inside one singular file, as this was all I needed to get it working and I do not have the time to learn how to use this language more proper.
