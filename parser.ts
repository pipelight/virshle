#!/usr/bin/env -S deno run -A

// Command line args
import { parse } from "https://deno.land/std/flags/mod.ts";
// Xml
import { stringify as to_xml } from "https://deno.land/x/xml/mod.ts";
// Toml
import {
  parse as from_toml,
  stringify as to_toml,
} from "https://deno.land/std/toml/mod.ts";

console.log(Deno.args);

const flags = parse(Deno.args, {
  string: ["from_toml"],
});

const text = await Deno.readTextFile(flags.from_toml);
console.log(to_xml(from_toml(text)));
