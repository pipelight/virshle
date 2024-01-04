import $ from "https://deno.land/x/dax/mod.ts";
import type { DefineArgs, DumpArgs } from "../types.ts";

export const define = async (args: DefineArgs | DumpArgs): Promise<string> => {
  // Sub-process
  const { code, stdout, stderr } = await $`virsh ${
    Object.values(args).join(" ")
  }`
    .stdout("piped")
    .stderr("piped");

  if (code == 0) {
    console.log(stdout);
    return stdout;
  } else {
    console.log(stderr);
    return stderr;
  }
};
export const dump = async (args: DefineArgs | DumpArgs): Promise<string> => {
  // Sub-process
  const { code, stdout, stderr } = await $`virsh ${
    Object.values(args).join(" ")
  }`
    .stdout("piped")
    .stderr("piped");

  if (code == 0) {
    return stdout;
  } else {
    return stderr;
  }
};

/**
Pass raw arguments to virsh
*/
export const raw = async (args: any) => {
  // Sub-process
  const cmd = new Deno.Command("virsh", { args: args });
  const child = cmd.spawn();
};
export const validate = async (args: DefineArgs) => {
  // Sub-process
  const cmd = new Deno.Command("virt-xml-validate", { args: [args.file] });
  const child = cmd.spawn();
};
