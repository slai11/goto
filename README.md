# goto

gt is a simple and user-friendly way to jump to your indexed directories.

gt is short for "goto", which is basically what you want to do with minimal
keystrokes.

## Features

* Convenient syntax `gt XXX` to jump to XXX's path
* Easy indexing of all directories by using 


## Installation 

Paste `eval $(goto init)` in your bashrc or zshrc.

The binary's name is `goto` while the command you should be using is `gt`.
A shell-based workaround inspired by https://github.com/ajeetdsouza/zoxide and
https://github.com/gsamokovarov/jump is used as it is not possible to change the
working directory of your shell programmatically.

## Command-Line Options

```

```


## Tutorial

Indexing a directory
```
gt add
gt add -r n
```

Jumping to an indexed directory
`gt PATH` to jump to your path

Cleaning up index to ensure all paths are valid
`gt prune` to update and remove non-existent directories


## todo
1. resolve repeated names in the hmap
2. recursive listing
3. list indexes
4. set up travis & github actions

