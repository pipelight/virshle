import { parseFlags } from "https://deno.land/x/cliffy@v1.0.0-rc.3/flags/mod.ts";
import { virsh } from "./mod.ts";
//Guards
export const optionGuardSwitch = (deno_args: any) => {

  // Igmore flags
  const { flags } = parseFlags(deno_args);
  const args = flags.unknown;

  const is_define = args.some((e: string) => virsh.define.includes(e));
  const is_dump = args.some((e: string) => virsh.dump.includes(e));
  const is_edit = args.some((e: string) => virsh.edit.includes(e));

  if (is_define) {
    // convertFile()
  } else if (is_dump) {
    //Args
    const command = args.shift();
    const file = args.shift();

    convertFile(args.shift(), args.shift());
  } else if (is_edit) {
    // convertFile()
  }

  if (args.length < 2) {
    console.error("Please provide at least a command and a file");
    return;
  }
};
