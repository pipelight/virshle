// An intuitive remap of virsh commands
import { Command } from "https://deno.land/x/cliffy/command/mod.ts";

import { domain } from "./domain.ts";
import { network } from "./network.ts";

export const cli = new Command()
  // Main command.
  .name("virshle")
  .version("0.2.0")
  .description("libvirt TOML/YAML wrapper")
  .globalOption(
    "-v , --verbosity <level:number>",
    "Set verbosity level",
    { default: 0 },
  )
  .command("vm", domain)
  .command("net", network);
