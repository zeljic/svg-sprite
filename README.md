# SVG Sprite
Takes SVG files and create SVG sprite.

```
svg-sprite 0.1.0
Takes SVG files and create SVG sprite

USAGE:
    svg-sprite [OPTIONS] <INPUT> <OUTPUT>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --clear-attribute <clear-attribute>...    Remove attributes from SVG file
    -s, --separator <separator>
            String placed between each directory in generated id for every SVG file [default: -]

    -t, --tag <tag>
            Tag for every generated child of new created SVG file [default: symbol]  [possible values: g, symbol]


ARGS:
    <INPUT>     Source directory where svg files are located
    <OUTPUT>    Output file
```
