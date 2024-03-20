/**
An object maping virshle commands to virsh commands
*/
export const map = {
  validate: "virt-xml-validate",
  domain: {
    dump: "dumpxml",
    create: "create",
    start: "start",
    stop: "shutdown",
    define: "define",
    list: "list",
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
