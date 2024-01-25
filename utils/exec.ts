export type Command = {
  cmd: string;
  args: string[];
};
export enum Status {
  Success,
  Fail,
}
export type Result = {
  status: Status;
  stdout: string;
  stderr: string;
};
export const simple = async ({ cmd, args }: Command): Promise<Partial<Result>> => {
  // Sub-process
  const str = Object.values(args);
  const command = new Deno.Command(cmd, {
    args: str,
  });
  const child = command.spawn();
  const output = await child.output();

  const res = {
    status: output.success ? Status.Success : Status.Fail,
  };

  return res;
};
export const raw = async ({ cmd, args }: Command): Promise<Result> => {
  // Sub-process
  const str = Object.values(args);
  const command = new Deno.Command(cmd, {
    stdout: "piped",
    stderr: "piped",
    args: str,
  });
  const child = command.spawn();
  const output = await child.output();

  const res = {
    stdout: new TextDecoder().decode(output.stdout),
    stderr: new TextDecoder().decode(output.stderr),
    status: output.success ? Status.Success : Status.Fail,
  };

  if (output.success) {
    loggy.log(res.stdout);
  } else {
    loggy.log(res.stderr);
  }
  return res;
};

export const pipe = async ({ cmd, args }: Command): Promise<Result> => {
  // Sub-process
  const command = new Deno.Command(cmd, {
    stdout: "piped",
    stderr: "piped",
    args: args,
  });
  const child = command.spawn();
  const output = await child.output();

  const res = {
    stdout: new TextDecoder().decode(output.stdout),
    stderr: new TextDecoder().decode(output.stderr),
    status: output.success ? Status.Success : Status.Fail,
  };
  return res;
};

export const exec = {
  raw,
  pipe,
  simple,
};
