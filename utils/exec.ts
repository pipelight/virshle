export const run = async (args: Args) => {
  // Sub-process
  const cmd = new Deno.Command("virsh", { args: [args.command, args.file] });
  const child = cmd.spawn();

  // Clean up
  // await Deno.remove(tmp.file);
};
