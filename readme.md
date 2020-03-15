# marksman

Make markdown articles into static web pages.

```
USAGE:
    marksman.exe [FLAGS] --input <input> --output <output> --template <template>

FLAGS:
    -h, --help         Prints help information
    -D, --no-digest    Disable JSON digests for listing articles on pages.
    -M, --no-media     Disable copying files in source directory.
    -V, --version      Prints version information

OPTIONS:
    -i, --input <input>          Input directory with markdown articles and media.
    -o, --output <output>        Output directory.
    -t, --template <template>    HTML template.
```

The setup is pretty straightforward. Just have a directory of markdown and images like:

```
articles/
    index.md
    hello-world.md
    media/
        image.png
src/
    template.html
```

It compiles to

```
out/
    digest0.json
    index.html
    hello-world.html
    media/
        image.png
```

More specifics below.

### Metadata

Metadata is given at the top at the top of articles as [YAML](https://en.wikipedia.org/wiki/YAML). Its used for information like the author, dates, tags, and so on.

```
---
title: Hello world
author: John Doe
tags: [ example, test ]
---

# Hello world

Lorem ipsum incididunt dolor dolor sit exercitation anim, nostrud ipsum laboris officia consectetur.
```

The fields can be anything, but they're rendered in one template so keep it consistent.

### Template

All articles are rendered in a [Handlebars](https://github.com/sunng87/handlebars-rust) HTML template. The variables available are `article`, `url`, and the article's metadata.

The rendered HTML is displayed with `{{{article}}}`.

For distinguished pages like the home page, use the `url` variable.

```
<h1>{{title}} by {{author}}</h1>
<div>{{{article}}}</div>
```

### Digest

Digests are JSON files available to other pages for listing the articles. Each digest item contains a summary and metadata. It can be disabled with `-D`, `--no-digest`.