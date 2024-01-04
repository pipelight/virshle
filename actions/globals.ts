let VERBOSITY = 0;

export const verbosity = {
  get: () => {
    return VERBOSITY;
  },
  set: (n: number) => {
    VERBOSITY = n;
  },
};
// A list of virsh commands that require an XML file as argument
const define = [
  "attach-device",
  "create",
  "checkpoint-create",
  "define",
  "detach-device",
  "update-device",
  "iface-define",
  "managedsave-define",
  "nwfilter-define",
  "nwfilter-binding-create",
  "net-create",
  "net-define",
  "net-port-create",
  "nodedev-create",
  "save-image-define",
  "save-image-define",
  "secret-define",
  "snapshot-create",
  "backup-dumpxml",
  "pool-create",
  "pool-define",
  "vol-create",
];

// A list of virsh commands that output XML
const dump = [
  "domxml-from-native",
  "domxml-to-native",
  "dumpxml",
  "managedsave-dumpxml",
  "metadata",
  "save-image-dumpxml",
  "cpu-compare",
  "checkpoint-dumpxml",
  "iface-dumpxml",
  "nwfilter-dumpxml",
  "net-dumpxml",
  "net-metadata",
  "net-port-dumpxml",
  "nodedev-dumpxml",
  "secret-dumpxml",
  "snapshot-dumpxml",
  "pool-dumpxml",
  "vol-dumpxml",
  "vol-dumpxml",
  "vol-dumpxml",
  "vol-dumpxml",
  "vol-dumpxml",
  "vol-dumpxml",
];

const edit = [
  "edit",
  "checkpoint-edit",
  "managedsave-edit",
  "net-edit",
  "save-image-edit",
  "snapshot-edit",
  "iface-edit",
  "pool-edit",
];

const special = [
  "validate",
];

const virsh: any = {};
virsh.cmds = {
  define,
  dump,
  edit,
  special,
};

export { virsh };
