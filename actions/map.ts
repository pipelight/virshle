// An intuitive remap of virsh commands
//
import { Command } from "https://deno.land/x/cliffy/command/mod.ts";

import { convert, exec, verbosity } from "../utils/mod.ts";

/**
An object maping virshle commands to virsh commands
*/
const map = {
  validate: "virt-xml-validate",
  domain: {
    dump: "dumpxml",
    create: "create",
    define: "define",
    list: "list",
    edit: "edit", // Deprecated
  },
  network: {
    dump: "net-dumpxml",
    create: "net-create",
    define: "net-define",
    edit: "net-edit", // Deprecated
    list: "net-list",
  },
};

const domain = new Command()
  .name("virshle domain")
  .description(
    "virtual machines(domains) manipulation command with TOML/YAML files",
  )
  .command("dump", "dump the core of a domain(vm) to stdout")
  .arguments("<name:string>")
  .action(async (options: any, name: string) => {
    const data = await exec.pipe({
      cmd: "virsh",
      args: [map.domain.dump, name],
    });
    const res = await convert.xml2toml(data);
    console.log(res);
  })
  .command("define", "define (but don,t start) a domain(vm) from a file")
  .arguments("<file:string>")
  .action(async (options: any, file: string) => {
    const xmlfile = await convert.toml2xml(file);
    await exec.raw({ cmd: "virsh", args: [map.domain.define, xmlfile] });
    await Deno.remove(xmlfile);
  })
  .command("list", "create and start a domain(vm) from a file")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, ...args: string[]) => {
    await exec.raw({ cmd: "virsh", args: [map.domain.list, ...args] });
  })
  .command("create", "create and start a domain(vm) from a file")
  .command("edit", "edit a domain(vm) configuration");

const network = new Command()
  // Main command.
  .name("virshle network")
  .description(
    "virtual machines(domains) manipulation command with TOML/YAML files",
  )
  .command("dump", "dump the core of a domain(vm) to stdout")
  .arguments("<name:string>")
  .action(async (options: any, name: string) => {
    // Set verbosity
    verbosity.set(options.verbosity.value);
    const data = await exec.pipe({
      cmd: "virsh",
      args: [map.network.dump, name],
    });
    const res = await convert.xml2toml(data);
    console.log(res);
  })
  .command("define", "define (but don,t start) a domain(vm) from a file")
  .arguments("<file:string>")
  .action(async (options: any, file: string) => {
    const xmlfile = await convert.toml2xml(file);
    await exec.raw({ cmd: "virsh", args: [map.network.define, xmlfile] });
    await Deno.remove(xmlfile);
  })
  .command("create", "create and start a domain(vm) from a file")
  .command("edit", "edit a domain(vm) configuration");

export const cli = new Command()
  // Main command.
  .name("virshle")
  .version("0.2.0")
  .description("libvirt TOML/YAML wrapper")
  .globalOption(
    "-v , --verbosity <level:number>",
    "Set verbosity level",
    { default: 0 },
  )
  .command("domain", domain)
  .command("net", network);
