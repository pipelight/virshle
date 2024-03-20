// Xml
import {
  parse as from_xml,
  stringify as to_xml,
} from "https://deno.land/x/xml/mod.ts";
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
// Uuid
import { generate as uuid } from "https://deno.land/std/uuid/v1.ts";
// Colors
import { colors, tty } from "https://deno.land/x/cliffy/ansi/mod.ts";

import { basename, dirname, extname } from "https://deno.land/std/path/mod.ts";
import { removeEmpty, replaceRelativePaths } from "./clean.ts";
import { verbosity } from "../mod.ts";

import { assertThrows } from "https://deno.land/std/assert/mod.ts";

export enum Extension {
  json = "json",
  yaml = "yaml",
  xml = "xml",
  toml = "toml",
  unknown = "txt",
}

export interface Args {
  path?: string;
  raw?: string;
  extension?: Extension;
}

export class File {
  path?: string;
  extension: Extension = Extension.unknown;
  data: any;
  raw?: string;

  constructor(
    args: Args,
  ) {
    if (args.extension) {
      this.extension = args.extension;
    }

    if (args.path && args.raw) {
      this.path = args.path;
      this.raw = args.raw;
    } else if (args.path) {
      this.path = args.path;
    } else if (args.raw) {
      this.raw = args.raw;
    }
  }

  // Detect the file format
  private set_extension() {
    if (this.path) {
      switch (extname(this.path!)) {
        case ".toml" || ".tml":
          this.extension = Extension.toml;
          this.data = from_toml(this.raw!);
          break;
        case ".yaml" || ".yml":
          this.extension = Extension.yaml;
          this.data = from_yaml(this.raw!);
          break;
        case ".xml":
          this.extension = Extension.xml;
          this.data = from_xml(this.raw!);
          break;
        case ".json":
          this.extension = Extension.json;
          this.data = JSON.parse(this.raw!);
          break;
        default:
          const msg = "Couldn't determine the input format";
          throw new Error(msg);
      }
    } else {
      try {
        this.data = JSON.parse(this.raw!);
      } catch {
        try {
          this.data = from_toml(this.raw!);
        } catch {
          try {
            this.data = from_xml(this.raw!);
          } catch {
            try {
              this.data = from_yaml(this.raw!);
            } catch {}
          }
        }
      }
    }
  }
  // Extract raw data to Javascript Object
  private async set_data() {
    if (this.path && typeof this.raw === "undefined") {
      this.raw = await Deno.readTextFile(this.path!);
    }
    this.set_extension();

    switch (this.extension) {
      case Extension.toml:
        this.data = from_toml(this.raw!);
        break;
      case Extension.yaml:
        this.data = from_yaml(this.raw!);
        break;
      case Extension.xml:
        this.data = from_xml(this.raw!);
        break;
      case Extension.json:
        this.data = JSON.parse(this.raw!);
        break;
    }
    // Sanitize
    this.data = await replaceRelativePaths(removeEmpty(this.data));
  }
  // Write file to disk
  private async write() {
    await Deno.mkdir(dirname(this.path!), { recursive: true });
    await Deno.writeTextFile(this.path!, this.raw!);
  }

  async read() {
    await this.set_data();
  }

  async convert(extension: Extension): Promise<File> {
    const res = new File({
      path: ".virshle/tmp" + "/" + uuid() + "." + extension,
      extension,
    });
    // Convert output data based on file extension
    switch (extension) {
      case Extension.toml:
        res.raw = to_toml(this.data!);
        break;
      case Extension.yaml:
        res.raw = to_yaml(this.data!);
        break;
      case Extension.xml:
        res.raw = to_xml(this.data!);
        break;
      case Extension.json:
        res.raw = JSON.stringify(this.data);
        break;
    }

    // Write file
    await res.write();

    // Logs
    // Global indent values for logs printing
    // const { columns } = Deno.consoleSize();
    const columns = 30;
    const indent = "-".repeat(columns / 3);
    const success = colors.bold.green;
    loggy.info(
      success(
        indent + `input:${this.extension}` + indent,
      ),
    );
    loggy.info(this.raw);
    loggy.info(success(indent.repeat(2)));
    loggy.debug(success(indent + `output:${res.extension}` + indent));
    loggy.debug(res.raw);
    loggy.debug(success(indent.repeat(2)));

    return res;
  }
}
