import type { Args } from "../types.ts";

export const run = async (args: Args) => {
  // Sub-process
  const cmd = new Deno.Command("virsh", { args: [args.cmd!, args.file] });
  const child = cmd.spawn();

  // Clean up
  // await Deno.remove(tmp.file);
};
export const validate = async (args: Args) => {
  // Sub-process
  const cmd = new Deno.Command("virt-xml-validate", { args: [args.file] });
  const child = cmd.spawn();

  // Clean up
  // await Deno.remove(tmp.file);
};
