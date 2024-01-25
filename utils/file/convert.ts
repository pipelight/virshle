import { Extension, File } from "./file.ts";

const any2toml = async (input: string): Promise<File> => {
  const infile = new File({
    raw: input,
  });
  await infile.read();

  const outfile = await infile.convert(Extension.toml);
  return outfile;
};

const any2xml = async (
  path: string,
): Promise<
  File
> => {
  // Get file
  const infile = new File({
    path,
  });
  await infile.read();
  // Convert
  const outfile = await infile.convert(Extension.xml);
  return outfile;
};

export const convert = {
  any2toml,
  any2xml,
};
