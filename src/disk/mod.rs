pub mod block;

use super::partition::Partition;

#[derive(Default, Debug)]
pub struct Disk {
    pub name: String,
    pub partitions: Option<Vec<Partition>>,
    pub linux_block_device: block::LinuxBlockDevice,
}

impl Disk {
    pub fn new_disk_without_parts(
        disk_name: &str,
    ) -> Result<(Self, block::LinuxBlockDevice), String> {
        if let Some(correct_linux_device) = block::linux_blk_name(disk_name) {
            let mut this_disk = Disk::default();
            this_disk.name = String::from(disk_name);
            return Ok((this_disk, correct_linux_device));
        }
        Err(format!(
            "disk name does match known Linux block device name (e.g. sdX, vdX, or nvmeXnY)"
        ))
    }
}
