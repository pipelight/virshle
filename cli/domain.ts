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
  .command("validate", "validate the domain file definition")
  .arguments("<file:string>")
  .useRawArgs()
  .stopEarly()
  .action(async (options: any, file: string, ...args: string[]) => {
    const xmlfile = await convert.toml2xml(file);
    await exec.raw({
      cmd: "virt-xml-validate",
      args: [xmlfile, ...args],
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
