# Load completions shared by various ssh tools like ssh, scp and sftp.
__fish_complete_ssh ssh

#
# ssh specific completions
#
complete -c ssh -d Vm -xa "(__fish_complete_user_at_hosts)"

function __fish_complete_virshle_vm -d "Print list vm with user@"
      vm_list=/usr/bin/env virshle vm get-list-names --state running);
end
