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
      const res = await convert.xml2toml(stdout);
      console.log(res);
    } else if (status == Status.Fail) {
      console.log(stderr);
    }
  })
  .command("define", "define (but don,t start) a network from a file")
  .arguments("<file:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, file: string, ...args: string[]) => {
    const xmlfile = await convert.toml2xml(file);
    await exec.raw({
      cmd: "virsh",
      args: [map.network.define, xmlfile, ...args],
    });
    await Deno.remove(xmlfile);
  })
  .command("create", "create a network from a file")
  .arguments("<file:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, file: string, ...args: string[]) => {
    const xmlfile = await convert.toml2xml(file);
    await exec.raw({
      cmd: "virsh",
      args: [map.network.create, xmlfile, ...args],
    });
    await Deno.remove(xmlfile);
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
  .command("list", "list networks")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, ...args: string[]) => {
    await exec.raw({ cmd: "virsh", args: [map.network.list, ...args] });
  })
  .command("edit", "edit a domain(vm) configuration");
