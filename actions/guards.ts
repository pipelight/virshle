import { parseFlags } from "https://deno.land/x/cliffy/flags/mod.ts";
import { verbosity, virsh } from "./mod.ts";
import { convert } from "../utils/mod.ts";
import { run } from "../utils/mod.ts";
//Guards
export const optionGuardSwitch = async (deno_args: any) => {
  // Igmore flags
  const { flags, unknown } = parseFlags(deno_args, {
    stopEarly: false,
    stopOnUnknown: false,
    flags: [
      {
        name: "verbose",
        aliases: ["v"],
        collect: true,
        value: (_: any, verbose = 0) => ++verbose,
      },
    ],
  });
  if (flags.verbose) {
    verbosity.set(flags.verbose);
  }
  if (!unknown.length) {
    console.debug("Please provide a command: virsh --help");
    const cmd = new Deno.Command("virsh", { args: ["--help"] });
    // const child = cmd.spawn();
    return;
  }

  const is_define = unknown.some((e: string) => virsh.cmds.define.includes(e));
  const is_dump = unknown.some((e: string) => virsh.cmds.dump.includes(e));
  const is_edit = unknown.some((e: string) => virsh.cmds.edit.includes(e));

  let args = {
    command: unknown.shift(),
    file: unknown.shift(),
  };

  if (is_define) {
    args = {
      ...args,
      ...await convert.toml2xml(args),
    };
    await run(args);
  } else if (is_dump) {
    //Args
  } else if (is_edit) {
    // convertFile()
  }
};
