import { Command } from "https://deno.land/x/cliffy/command/mod.ts";

import { convert, exec, verbosity } from "../utils/mod.ts";
import { Status } from "../utils/mod.ts";
import { map } from "./map.ts";

/**
 * A sub-command for every volume related operationsal
 */
export const volume = new Command()
  .name("virshle vol")
  .description(
    "virtual machines(domains) manipulation commands",
  )
  .command("list", "list storage pools")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, ...args: string[]) => {
    await exec.raw({ cmd: "virsh", args: [map.volume.list, ...args] });
  })
  /**
   * Create a volume
   * Usage:
   *
   * Put the flags after file path
   *
   * ```sh
   * virshle vol create ./base/volumes/default.toml --pool default
   * ```
   */
  .command("create", "create a volume from a file")
  .arguments("<path:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, path: string, ...args: string[]) => {
    const file = await convert.any2xml(path);
    await exec.raw({
      cmd: "virsh",
      args: [map.volume.create, ...args, file.path!],
    });
    // Clean up
    await Deno.remove(file.path!);
  })
  .command("delete", "delete a previously defined volume")
  .arguments("<name:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, name: string, ...args: string[]) => {
    await exec.raw({
      cmd: "virsh",
      args: [map.volume.delete, name, ...args],
    });
  })
  .command("info", "volume information")
  .arguments("<path:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, name: string, ...args: string[]) => {
    await exec.raw({
      cmd: "virsh",
      args: [map.volume.info, name, ...args],
    });
  });
