# arthur

Make markdown articles into static web pages.

## Install

You can install with cargo:

```
cargo install arthur
```

## Usage

```
USAGE:
    arthur [FLAGS] --input <input> --output <output> --template <template>

FLAGS:
    -h, --help         Prints help information
    -D, --no-digest    Disable JSON digests for listing articles on pages.
    -G, --no-gfm       Disable Github-flavored markdown.
    -M, --no-media     Disable copying files in source directory.
    -V, --version      Prints version information

OPTIONS:
    -i, --input <input>          Input directory with markdown articles and media.
    -o, --output <output>        Output directory.
    -t, --template <template>    HTML template.
```

The setup is simple. Have a directory of markdown and images like:

```
articles/
    index.md
    hello-world.md
    media/
        image.png
src/
    template.html
```

And it compiles to:

```
out/
    digest0.json
    index.html
    hello-world.html
    media/
        image.png
```

By using the command

```
arthur --input articles --template src/template.html --output out
```

### Metadata

Metadata is given at the top at the top of articles in [YAML](https://en.wikipedia.org/wiki/YAML). Its used specifying the author, dates, tags, or whatever you want to use in your template.

```
---
title: Hello world
author: John Doe
tags: [ example, test ]
---

# Hello world

Lorem ipsum incididunt dolor dolor sit exercitation anim, nostrud ipsum laboris officia consectetur.
```

### Template

Articles are rendered with a Handlebars template ([1](https://github.com/sunng87/handlebars-rust), [2](https://handlebarsjs.com/)) into HTML.

The template has the variables `article`, `url`, and each article's metadata.

```
<h1>{{title}} by {{author}}</h1>
<div>{{{article}}}</div>
```

The rendered HTML is displayed with `{{{article}}}`.

Use `url` to create distinguished pages, like the home page.

### Digest

Digests are JSON files available to other pages for listing the articles. Each digest item contains a summary and metadata. It can be disabled with `-D`, `--no-digest`.