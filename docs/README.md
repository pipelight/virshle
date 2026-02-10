# Virshle documentation

You can browse documentation as is through the markdown files.
Or you can build the website with Zola.

## Zola suprecharged.

This documentation website uses Zola
plus some other preprocessors to ease file readability:

- pug -> html.
- tailwindcss -> css.

## Development

A few pipelines ease the installation of dependencies.

```sh
p run install
```

Serve the website with hot-reload.

```sh
p enable watcher && zola serve
```
