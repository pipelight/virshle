# Jechiale

## What it is!

A cli made of small utilities written in typescript/deno. It convert between
file formats:

- TOML,
- YAML,
- JSON,
- and XML.

## Why another file converter

I need it to use libvirt with TOML instead of XML. I spent my time crying while
looking for one I could installe simply until I bumped into Deno internals.

```ts
Deno.readTextFile();
```

And the one line **to_toml** **from_toml** parsers and the many others.

This cli is only a few lines long and makes usage of Deno internals for file
format conversion.

## Usage

```sh
jechiale --from_xml <file> --to_toml <file>
```
```sh
virsh create < $(jechiale --from_xml <file> --to_toml <file>)
```

## S/O

Inpired by mario and nushell.
