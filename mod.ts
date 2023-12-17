#!/usr/bin/env -S deno run -A

import { generate as uuid } from "https://deno.land/std/uuid/v1.ts";
// Xml
import { stringify as to_xml } from "https://deno.land/x/xml/mod.ts";
// Toml
import {
  parse as from_toml,
  stringify as to_toml,
} from "https://deno.land/std/toml/mod.ts";
// Yaml
import {
  parse as from_yaml,
  stringify as to_yaml,
} from "https://deno.land/std/yaml/mod.ts";

import { Command } from "https://deno.land/x/cliffy/command/mod.ts";
import { colors, tty } from "https://deno.land/x/cliffy/ansi/mod.ts";

const success = colors.bold.green;

const cli = new Command()
  .name("virshle")
  .version("0.1.0")
  .description("A virsh YAML/TOML wrapper");

// Subcommands - Getters
cli
  .arguments("[virsh_command] [file]")
  .action(async (_options, ...args: any) => {
    //Guards
    if (args.length < 2) {
      console.error("Please provide at least a command and a file");
      return;
    }
    //Args
    const command = args.shift();
    const file = args.shift();

    // Convert
    const text = await Deno.readTextFile(file!);
    let markup;
    let format;
    if (from_toml(text!)) {
      markup = from_toml(text!);
      format = "toml";
    } else if (from_yaml(text!)) {
      markup = from_yaml(text!);
      format = "yaml";
    } else if (JSON.parse(text!)) {
      markup = JSON.parse(text!);
      format = "json";
    } else {
      console.error("Could not convert the provided file");
      return;
    }
    const xml = to_xml(markup);

    const tmp = {
      dir: ".virshle/tmp",
      file: ".virshle/tmp" + "/" + uuid(),
    };

    await Deno.mkdir(tmp.dir, { recursive: true });

    const encoder = new TextEncoder();
    const data = encoder.encode(xml);
    await Deno.writeFile(`${tmp.file}`, data);

    console.debug(success(`-------------input:${format}--------------`));
    console.debug(text);
    console.debug(success(`------------------------------------------`));
    console.debug(success("-------------output:xml--------------"));
    console.debug(xml);
    console.debug(success(`------------------------------------------`));

    // Sub-process
    const cmd = new Deno.Command("virsh", { args: [command, tmp.file] });
    const child = cmd.spawn();
  });

await cli.parse(Deno.args);
