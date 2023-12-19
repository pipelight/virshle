#!/usr/bin/env -S deno run -A


import { Command } from "https://deno.land/x/cliffy/command/mod.ts";
import { colors, tty } from "https://deno.land/x/cliffy/ansi/mod.ts";

const success = colors.bold.green;

const cli = new Command()
  .name("virshle")
  .version("0.1.0")
  .description("A virsh YAML/TOML wrapper");

// Subcommands - Getters
cli
  .arguments("[virsh_command] [file]")
  .action(async (_options, ...args: any) => {
  });

await cli.parse(Deno.args);
