use sysinfo::Disks;

#[derive(Clone, Debug)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space_gb: f64,
    pub available_space_gb: f64,
    pub file_system: String,
    pub is_removable: bool,
}

pub fn detect_disks() -> Vec<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    let mut list = Vec::new();

    for disk in &disks {
        let mount_point = disk.mount_point().to_string_lossy().to_string();
        let name = if disk.name().is_empty() {
            "Storage Device".to_string()
        } else {
            disk.name().to_string_lossy().to_string()
        };

        let total_space_gb = disk.total_space() as f64 / (1024.0 * 1024.0 * 1024.0);
        let available_space_gb = disk.available_space() as f64 / (1024.0 * 1024.0 * 1024.0);
        let file_system = disk.file_system().to_string_lossy().to_string();
        let is_removable = disk.is_removable();

        list.push(DiskInfo {
            name,
            mount_point,
            total_space_gb,
            available_space_gb,
            file_system,
            is_removable,
        });
    }

    // Sort by mount point
    list.sort_by(|a, b| a.mount_point.cmp(&b.mount_point));
    list
}
