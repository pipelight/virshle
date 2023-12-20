#!/usr/bin/env -S deno run -A

import { Command } from "https://deno.land/x/cliffy/command/mod.ts";
import { convert } from "./utils/mod.ts";
import { optionGuardSwitch } from "./actions/mod.ts";

const cli = new Command()
  .name("virshle")
  .version("0.1.0")
  .description("A virsh YAML/TOML wrapper");

// Subcommands - Getters
await optionGuardSwitch(Deno.args);
