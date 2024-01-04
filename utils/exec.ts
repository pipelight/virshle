import $ from "https://deno.land/x/dax/mod.ts";
import type { DefineArgs, DumpArgs } from "../types.ts";

export const define = async (args: DefineArgs | DumpArgs): Promise<string> => {
  // Sub-process
  const str = Object.values(args);
  const cmd = new Deno.Command("virsh", {
    args: str,
    stdout: "piped",
    stderr: "piped",
  });
  const child = cmd.spawn();

  const output = await child.output();

  const stdout = new TextDecoder().decode(output.stdout);
  const stderr = new TextDecoder().decode(output.stderr);

  if (output.success) {
    console.log(stdout);
    return stdout;
  } else {
    console.log(stderr);
    return stderr;
  }
};

export const dump = async (args: DefineArgs | DumpArgs): Promise<string> => {
  // Sub-process
  const str = Object.values(args);
  const cmd = new Deno.Command("virsh", {
    args: str,
    stdout: "piped",
    stderr: "piped",
  });
  const child = cmd.spawn();

  const output = await child.output();

  const stdout = new TextDecoder().decode(output.stdout);
  const stderr = new TextDecoder().decode(output.stderr);

  if (output.success) {
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
