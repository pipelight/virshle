/**
An object maping virshle commands to virsh commands
*/
export const map = {
  validate: "virt-xml-validate",
  pool: {
    create: "pool-create",
    delete: "pool-delete",
    info: "pool-info",
    list: "pool-list",
    dump: "pool-dumpxml",
    define: "pool-define",
    autostart: "pool-autostart",
    start: "pool-start",
    stop: "pool-destroy",
    undefine: "pool-undefine",
  },
  volume: {
    create: "vol-create",
    delete: "vol-delete",
    info: "vol-info",
    list: "vol-list",
    define: "vol-define",
  },
  domain: {
    dump: "dumpxml",
    create: "create",
    start: "start",
    stop: "shutdown",
    define: "define",
    list: "list",
    info: "dominfo",
    delete: "destroy",
    shutdown: "shutdown",
    undefine: "undefine",
    edit: "edit", // Deprecated
  },
  network: {
    dump: "net-dumpxml",
    create: "net-create",
    define: "net-define",
    list: "net-list",
    remove: "net-destroy",
    undefine: "net-undefine",
    edit: "net-edit", // Deprecated
    info: "net-info",
    dhcp: "net-dhcp-leases",
  },
};
