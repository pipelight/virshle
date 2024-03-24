import { Command } from "https://deno.land/x/cliffy/command/mod.ts";

import { convert, exec, verbosity } from "../utils/mod.ts";
import { Status } from "../utils/mod.ts";
import { map } from "./map.ts";

/**
A sub-command for every network related operations
*/
export const network = new Command()
  // Main command.
  .name("virshle network")
  .description(
    "virtual network manipulation commands",
  )
  .command("dump", "dump the core of a network to stdout")
  .arguments("<name:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, name: string) => {
    const { status, stdout, stderr } = await exec.pipe({
      cmd: "virsh",
      args: [map.network.dump, name],
    });
    if (status == Status.Success) {
      const file = await convert.any2toml(stdout);
      console.log(file.raw);
    } else if (status == Status.Fail) {
      console.log(stderr);
    }
  })
  .command("define", "define (but don,t start) a network from a file")
  .arguments("<path:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, path: string, ...args: string[]) => {
    const file = await convert.any2xml(path);
    await exec.raw({
      cmd: "virsh",
      args: [map.network.define, file.path!, ...args],
    });
    // Clean up
    await Deno.remove(file.path!);
  })
  .command("create", "create a network from a file")
  .arguments("<path:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, path: string, ...args: string[]) => {
    const file = await convert.any2xml(path);
    await exec.raw({
      cmd: "virsh",
      args: [map.network.create, file.path!, ...args],
    });
    // Clean up
    await Deno.remove(file.path!);
  })
  .command("remove", "remove a network")
  .arguments("<name:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, name: string, ...args: string[]) => {
    await exec.raw({
      cmd: "virsh",
      args: [map.network.remove, name, ...args],
    });
  })
  .command("leases", "get host addresses on a network (dhcp leases)")
  .arguments("<name:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, name: string, ...args: string[]) => {
    await exec.raw({
      cmd: "virsh",
      args: [map.network.dhcp, name, ...args],
    });
  })
  .command("info", "get basic informations about a network")
  .arguments("<name:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, name: string, ...args: string[]) => {
    await exec.raw({
      cmd: "virsh",
      args: [map.network.info, name, ...args],
    });
  })
  .command("list", "list networks")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, ...args: string[]) => {
    await exec.raw({ cmd: "virsh", args: [map.network.list, ...args] });
  })
  // Edit a configuration with editor
  .command("edit", "edit a network configuration")
  .arguments("<name:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, name: string) => {
    // Try destroy vm
    const dumpAction = await exec.pipe({
      cmd: "virsh",
      args: [
        map.network.dump,
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
          map.network.define,
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
