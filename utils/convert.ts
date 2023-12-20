// Uuid
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

// Colors
import { colors, tty } from "https://deno.land/x/cliffy/ansi/mod.ts";

import type { Args } from "../types.ts";
import { verbosity } from "../actions/mod.ts";

const success = colors.bold.green;
const xml2toml = async ({
  file,
}: Args) => {
  // const text = await Deno.readTextFile(file!);
};

const toml2xml = async ({
  file,
}: Args): Promise<
  | Args
  | undefined
> => {
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

  const encoder = new TextEncoder();
  const data = encoder.encode(xml);
  await Deno.mkdir(tmp.dir, { recursive: true });
  await Deno.writeFile(`${tmp.file}`, data);

  if (!!verbosity.get()) {
    console.debug(success(`-------------input:${format}--------------`));
    console.debug(text);
    console.debug(success(`------------------------------------------`));
    console.debug(success("-------------output:xml-------------------"));
    console.debug(xml);
    console.debug(success(`------------------------------------------`));
  }

  return { file: tmp.file };
};

export const convert = {
  xml2toml,
  toml2xml,
};
