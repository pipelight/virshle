// An intuitive remap of virsh commands
//
import { Command } from "https://deno.land/x/cliffy/command/mod.ts";

import { convert, exec, verbosity } from "../utils/mod.ts";
import { Status } from "../utils/mod.ts";

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
    destroy: "destroy",
    shutdown: "shutdown",
    undefine: "undefine",
    edit: "edit", // Deprecated
  },
  network: {
    dump: "net-dumpxml",
    create: "net-create",
    define: "net-define",
    list: "net-list",
    remove: "net-destroy",
    undefine: "net-undefine",
    edit: "net-edit", // Deprecated
  },
};

/**
A sub-command for every domain/vm related operations
*/
const domain = new Command()
  .name("virshle vm")
  .description(
    "virtual machines(domains) manipulation command with TOML/YAML files",
  )
  .command("dump", "dump the core of a domain(vm) to stdout")
  .arguments("<name:string>")
  .action(async (options: any, name: string) => {
    const { status, stdout, stderr } = await exec.pipe({
      cmd: "virsh",
      args: [map.domain.dump, name],
    });
    if (status == Status.Success) {
      const res = await convert.xml2toml(stdout);
      console.log(res);
    } else if (status == Status.Fail) {
      console.log(stderr);
    }
  })
  /**
  The crunch is a hard unrecoverable deletion (for privacy).
  Libvirt has multiple methods to remove vms.
  The crunch action is here to bulk execute all those methods:

  - Destroy domain (successful only on running vms)
  - Undefine domain (successful only on non-running vms)

  - Delete volumes/storages

  - Delete hypervisor optionnal backups
  */
  .command("crunch", "hard delete the vm (hypervisor definition and storage)")
  .arguments("<name:string>")
  .action(async (options: any, name: string) => {
    // Try destroy vm
    let { status, stdout, stderr } = await exec.pipe({
      cmd: "virsh",
      args: [
        map.domain.destroy,
        name,
      ],
    });
    console.log(stderr);
    // Try Undefine vm
    if (status == Status.Success) {
      console.log(stdout);
    } else if (status == Status.Fail) {
      try {
        let { status, stdout, stderr } = await exec.pipe({
          cmd: "virsh",
          args: [map.domain.shutdown],
        });
      } catch (err) {
        console.error(err);
      }

      let { status, stdout, stderr } = await exec.pipe({
        cmd: "virsh",
        args: [
          map.domain.undefine,
          name,
          "--managed-save",
          "--remove-all-storage",
          "--delete-storage-volume-snapshots",
          "--wipe-storage",
          "--snapshots-metadata",
          "--nvram",
          "--tpm",
        ],
      });
      if (status == Status.Success) {
        console.log(stdout);
      } else if (status == Status.Fail) {
        console.log(stderr);
      }
    }
  })
  .command("define", "define (but don,t start) a domain(vm) from a file")
  .arguments("<file:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, file: string, ...args: string[]) => {
    const xmlfile = await convert.toml2xml(file);
    await exec.raw({
      cmd: "virsh",
      args: [map.domain.define, xmlfile, ...args],
    });
    await Deno.remove(xmlfile);
  })
  .command("create", "create a domain(vm) from a file")
  .arguments("<file:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, file: string, ...args: string[]) => {
    const xmlfile = await convert.toml2xml(file);
    await exec.raw({
      cmd: "virsh",
      args: [map.domain.create, xmlfile, ...args],
    });
    await Deno.remove(xmlfile);
  })
  .command("list", "list domains(vms)")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, ...args: string[]) => {
    await exec.raw({ cmd: "virsh", args: [map.domain.list, ...args] });
  })
  .command("edit", "edit a domain(vm) configuration");

/**
A sub-command for every network related operations
*/
const network = new Command()
  // Main command.
  .name("virshle network")
  .description(
    "virtual machines(domains) manipulation command with TOML/YAML files",
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
  .command("vm", domain)
  .command("net", network);
