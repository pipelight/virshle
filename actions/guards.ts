import { parseFlags } from "https://deno.land/x/cliffy/flags/mod.ts";
import { virsh } from "./mod.ts";
import { convert, verbosity } from "../utils/mod.ts";
import { define, dump, raw, validate } from "../utils/mod.ts";
import type { DefineArgs, DumpArgs } from "../types.ts";
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
      {
        name: "help",
        aliases: ["h"],
      },
    ],
  });
  if (flags.verbose) {
    verbosity.set(flags.verbose);
  }
  if (flags.help) {
    // console.log("Virshle");
    // const cmd = new Deno.Command("virsh", { args: ["--help"] });
    // cmd.spawn();
  }
  if (!unknown.length) {
    console.debug("Please provide a command: virsh --help");
    return;
  }


  const is_define = unknown.some((e: string) => virsh.cmds.define.includes(e));
  const is_special = unknown.some((e: string) =>
    virsh.cmds.special.includes(e)
  );

  const is_dump = unknown.some((e: string) => virsh.cmds.dump.includes(e));
  const is_edit = unknown.some((e: string) => virsh.cmds.edit.includes(e));

  if (is_define) {
    let args: DefineArgs = {
      cmd: unknown.shift(),
      file: unknown.shift(),
    };
    args = {
      ...args,
      ...await convert.toml2xml(args),
    };
    await define(args);

    await Deno.remove(args.file);
  } else if (is_special) {
    let args = {
      cmd: unknown.shift(),
      file: unknown.shift(),
    };
    args = {
      ...args,
      ...await convert.toml2xml(args),
    };
    switch (args.cmd) {
      case "validate":
        await validate(args);
    }
  } else if (is_dump) {
    let args: DumpArgs = {
      cmd: unknown.shift(),
      item: unknown.shift(),
    };
    const data = await dump(args);
    const res = await convert.xml2toml(data);
    console.log(res);

    // convertFile()
  } else if (is_edit) {
    // convertFile()
  } else {
    const args = unknown;
    await raw(args);
  }
};
