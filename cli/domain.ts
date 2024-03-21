import { Command } from "https://deno.land/x/cliffy/command/mod.ts";

import { convert, exec, verbosity } from "../utils/mod.ts";
import { Status } from "../utils/mod.ts";
import { map } from "./map.ts";

/**
A sub-command for every domain/vm related operations
*/
export const domain = new Command()
  .name("virshle vm")
  .description(
    "virtual machines(domains) manipulation commands",
  )
  .command("dump", "dump the core of a domain(vm) to stdout")
  .arguments("<name:string>")
  .action(async (options: any, name: string) => {
    const { status, stdout, stderr } = await exec.pipe({
      cmd: "virsh",
      args: [map.domain.dump, name],
    });
    if (status == Status.Success) {
      const file = await convert.any2toml(stdout);
      console.log(file.raw);
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
  .command("delete", "hard delete the vm (hypervisor definition and storage)")
  .arguments("<name:string>")
  .action(async (options: any, name: string) => {
    // Try destroy vm
    let { status, stdout, stderr } = await exec.pipe({
      cmd: "virsh",
      args: [
        map.domain.delete,
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
  .arguments("<path:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, path: string, ...args: string[]) => {
    const file = await convert.any2xml(path);
    await exec.raw({
      cmd: "virsh",
      args: [map.domain.define, file.path!, ...args],
    });
    // Clean uo
    await Deno.remove(file.path!);
  })
  .command("create", "create a domain(vm) from a file")
  .arguments("<path:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, path: string, ...args: string[]) => {
    const file = await convert.any2xml(path);
    await exec.raw({
      cmd: "virsh",
      args: [map.domain.create, file.path!, ...args],
    });
    // Clean up
    await Deno.remove(file.path!);
  })
  .command("start", "start a previously defined vm")
  .arguments("<name:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, name: string, ...args: string[]) => {
    await exec.raw({
      cmd: "virsh",
      args: [map.domain.start, name, ...args],
    });
  })
  .command("validate", "validate the domain file definition")
  .arguments("<path:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, path: string, ...args: string[]) => {
    const file = await convert.any2xml(path);
    await exec.raw({
      cmd: "virt-xml-validate",
      args: [file.path!, ...args],
    });
    // Clean up
    await Deno.remove(file.path!);
  })
  .command("list", "list domains(vms)")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, ...args: string[]) => {
    await exec.raw({ cmd: "virsh", args: [map.domain.list, ...args] });
  })
  .command("dominfo", "domain(vm) information")
  .arguments("<path:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, path: string, ...args: string[]) => {
    await exec.raw({
      cmd: "virsh",
      args: [map.domain.info, name, ...args],
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
        map.domain.dump,
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
          map.domain.define,
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
