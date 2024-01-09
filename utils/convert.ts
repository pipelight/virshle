// Uuid
import { generate as uuid } from "https://deno.land/std/uuid/v1.ts";
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

// Colors
import { colors, tty } from "https://deno.land/x/cliffy/ansi/mod.ts";

import { removeEmpty, replaceRelativePaths, verbosity } from "./mod.ts";

type ConvertionResult = {
  origin_format: string;
  output: string;
};

const to_XML = async (input: string): Promise<ConvertionResult> => {
  let data;
  let origin_format;
  if (from_toml(input!)) {
    data = from_toml(input!);
    origin_format = "toml";
  } else if (from_yaml(input!)) {
    data = from_yaml(input!);
    origin_format = "yaml";
  } else if (JSON.parse(input!)) {
    data = JSON.parse(input!);
    origin_format = "json";
  } else {
    const msg = "Could not convert the provided file";
    throw new Error(msg);
  }
  data = await replaceRelativePaths(removeEmpty(data));
  const output = to_xml(data);
  return {
    origin_format,
    output,
  };
};
const to_TOML = async (input: string): Promise<ConvertionResult> => {
  let data;
  let origin_format;
  if (from_xml(input!)) {
    data = from_xml(input!);
    data = await replaceRelativePaths(removeEmpty(data));
    origin_format = "xml";
  } else {
    const msg = "Could not convert the provided file";
    throw new Error(msg);
  }
  const output = to_toml(data);
  return {
    origin_format,
    output,
  };
};

const xml2toml = async (input: string): Promise<string> => {
  const { columns, rows } = Deno.consoleSize();
  let indent = "-".repeat(columns / 3);

  const { origin_format, output } = await to_TOML(input);

  const success = colors.bold.green;
  loggy.info(
    success(
      indent + `input:${origin_format}` + indent,
    ),
  );
  loggy.info(input);
  loggy.info(success(indent.repeat(2)));

  loggy.info(success(indent + `output:toml` + indent));
  loggy.info(output);
  loggy.info(success(indent.repeat(2)));
  return output;
};

const toml2xml = async (
  file: string,
): Promise<
  string
> => {
  // Convert
  const text = await Deno.readTextFile(file!);
  const { origin_format, output: xml } = await to_XML(text);

  const tmp = {
    dir: ".virshle/tmp",
    file: ".virshle/tmp" + "/" + uuid(),
  };

  const encoder = new TextEncoder();
  const data = encoder.encode(xml);

  await Deno.mkdir(tmp.dir, { recursive: true });
  await Deno.writeFile(`${tmp.file}`, data);

  const success = colors.bold.green;
  loggy.info(success(`-------------input:${origin_format}--------------`));
  loggy.info(text);
  loggy.info(success(`------------------------------------------`));
  loggy.info(success("-------------output:xml-------------------"));
  loggy.info(xml);
  loggy.info(success(`------------------------------------------`));

  return tmp.file;
};

export const convert = {
  to_XML,
  to_TOML,
  xml2toml,
  toml2xml,
};
