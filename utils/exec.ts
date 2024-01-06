
type Command = {
  cmd: string;
  args: string[];
};
export const raw = async ({ cmd, args }: Command): Promise<string> => {
  // Sub-process
  const str = Object.values(args);
  const command = new Deno.Command(cmd, {
    args: str,
    stdout: "piped",
    stderr: "piped",
  });
  const child = command.spawn();

  const output = await child.output();

  const stdout = new TextDecoder().decode(output.stdout);
  const stderr = new TextDecoder().decode(output.stderr);

  if (output.success) {
    console.log(stdout);
    return stdout;
  } else {
    console.log(stderr);
    return stderr;
  }
};

export const pipe = async ({ cmd, args }: Command): Promise<string> => {
  // Sub-process
  const command = new Deno.Command(cmd, {
    stdout: "piped",
    stderr: "piped",
    args: args,
  });
  const child = command.spawn();

  const output = await child.output();

  const stdout = new TextDecoder().decode(output.stdout);
  const stderr = new TextDecoder().decode(output.stderr);

  if (output.success) {
    return stdout;
  } else {
    return stderr;
  }
};

export const exec = {
  raw,
  pipe,
};
