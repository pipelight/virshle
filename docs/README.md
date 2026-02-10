# Virshle documentation

You can browse documentation as is through the markdown files.
Or build the website with zola.

# Zola suprecharged.

Uses zola + some other preprocessors to ease file readability

- pug -> html.
- tailwindcss -> css.

## Development

```sh
p run install
```

```sh
p enable watcher && zola serve
```

## Start fresh

Init the zola blog

```sh
zola init
```

Install the required preprocessors.

```sh
# Cli tools
bun add -g pug-cli
bun add -g tailwindcss @tailwindcss/cli

# LSP
bun add -g tailwindcss @tailwindcss/language-server
go install github.com/opa-oz/pug-lsp@latest
```

Add vue js for simple js support.

# PWA (Progressive web app)

Generate assets from svg file width
[pwa-asset-generator]()
