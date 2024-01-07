#!/usr/bin/env -S deno run -A

import { cli } from "./cli/mod.ts";
import { verbosity } from "./utils/mod.ts";
import { parseFlags } from "https://deno.land/x/cliffy/flags/mod.ts";

// Set verbosity
const ctx = Deno.args.filter((e) => e.includes("-v"));
export const { flags } = parseFlags(ctx, {
  flags: [{
    name: "verbosity",
    aliases: ["v"],
    collect: true,
    value: (val: boolean, previous = 0) => val ? previous + 1 : 0,
  }],
});
const args = Deno.args.filter((e) => !e.includes("-v"));

verbosity.set(flags.verbosity);

// Run cli
cli.parse(args);
