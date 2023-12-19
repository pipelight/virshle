
  // Sub-process
  const cmd = new Deno.Command("virsh", { args: [command, tmp.file] });
  const child = cmd.spawn();

  // Clean up
  await Deno.remove(tmp.file);
