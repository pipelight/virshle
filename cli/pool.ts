import { Command } from "https://deno.land/x/cliffy/command/mod.ts";

import { convert, exec, verbosity } from "../utils/mod.ts";
import { Status } from "../utils/mod.ts";
import { map } from "./map.ts";

/**
 * A sub-command for every domain/vm related operationsal
 */
export const pool = new Command()
  .name("virshle pool")
  .description(
    "storage pool manipulation commands",
  )
  .command("list", "list storage pools")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, ...args: string[]) => {
    await exec.raw({ cmd: "virsh", args: [map.pool.list, ...args] });
  })
  /**
   * Create a persistent pool
   * do not make use of virsh create but rather
   * virshe define and virshe autostart
   */
  .command("create", "create a pool from a file")
  .arguments("<path:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, path: string, ...args: string[]) => {
    const file = await convert.any2xml(path);
    file.read();

    // Ensure pool path
    await Deno.mkdir(file.data.pool.target.path, { recursive: true });

    await exec.raw({
      cmd: "virsh",
      args: [map.pool.define, file.path!, ...args],
    });

    await exec.raw({
      cmd: "virsh",
      args: [map.pool.start, file.data.pool.name!, ...args],
    });
    await exec.raw({
      cmd: "virsh",
      args: [map.pool.autostart, file.data.pool.name!, ...args],
    });

    // Clean up
    await Deno.remove(file.path!);
  })
  .command("delete", "delete a previously defined pool")
  .arguments("<name:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, name: string, ...args: string[]) => {
    await exec.raw({
      cmd: "virsh",
      args: [map.pool.stop, name, ...args],
    });
    await exec.raw({
      cmd: "virsh",
      args: [map.pool.delete, name, ...args],
    });
    await exec.raw({
      cmd: "virsh",
      args: [map.pool.undefine, name, ...args],
    });
  })
  .command("info", "pool information")
  .arguments("<path:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, name: string, ...args: string[]) => {
    await exec.raw({
      cmd: "virsh",
      args: [map.pool.info, name, ...args],
    });
  })
  // Edit a configuration with nvim
  .command("edit", "edit a domain(vm) configuration")
  .arguments("<name:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, name: string) => {
    // Try destroy vm
    const dumpAction = await exec.pipe({
      cmd: "virsh",
      args: [
        map.pool.dump,
        name,
      ],
    });
    if (dumpAction.status == Status.Success) {
      const file = await convert.any2toml(dumpAction.stdout);
      await exec.simple({
        cmd: Deno.env.has("EDITOR") ? Deno.env.get("EDITOR")! : "nvim",
        args: [
          file.path!,
        ],
      });

      const res = await convert.any2xml(file.path!);
      const defineAction = await exec.pipe({
        cmd: "virsh",
        args: [
          map.pool.define,
          res.path!,
        ],
      });
      if (defineAction.status == Status.Success) {
        console.log(defineAction.stdout);
      } else if (defineAction.status == Status.Fail) {
        console.log(defineAction.stderr);
      }
    } else if (dumpAction.status == Status.Fail) {
      console.log(dumpAction.stderr);
    }
  });
