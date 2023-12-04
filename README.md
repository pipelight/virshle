# Virshle - A modern libvirt wrapper.

A wrapper around libvirtd and virsh cli to use JSON,TOML and YAML over the
classic XML.

Plus some extra features

## How it works

It uses Deno internals.

```ts
Deno.readTextFile();
```

## Usage

Instead of

```sh
virsh domain create file.xml
```

Type

```sh
virshle domain create file.toml
```

## S/O

Inpired by mario and nushell.
