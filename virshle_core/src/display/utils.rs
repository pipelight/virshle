use crate::cloud_hypervisor::DiskTemplate;
use human_bytes::human_bytes;

pub fn display_vram(vram: &u64) -> String {
    let res = human_bytes((vram * u64::pow(1024, 3)) as f64);
    format!("{}", res)
}
pub fn display_disks(disks: &Option<Vec<DiskTemplate>>) -> String {
    let mut res = "".to_owned();
    if let Some(disks) = disks {
        let strs: Vec<String> = disks
            .iter()
            .map(|e| format!("{} -> {}", e.name, e.path))
            .collect();

        res = strs.join("\n");
    }
    res
}
pub fn display_ips(ips: &Vec<String>) -> String {
    let res = ips.join("\n");
    format!("{}\n", res)
}

pub fn display_id(id: &Option<u64>) -> String {
    if let Some(id) = id {
        format!("{}", id)
    } else {
        return "".to_owned();
    }
}
