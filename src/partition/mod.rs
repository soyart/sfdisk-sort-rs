pub mod parse;

use crate::linux::block;

/// Represents what matters for sfdisk-sort to reassign the names in the partition table.
/// Fields `designation` and `start_block` are used for sorting.
// Trait Clone is now only used for testing - TODO: remove?
#[derive(Default, Debug, PartialEq, Clone)]
pub struct Partition {
    // For sorting
    pub(crate) designation: usize,
    pub(crate) start_block: usize,

    // For reconstructing sfdisk dump output
    pub(crate) name: String, // This will be full path, e.g. /dev/sda1
    pub(crate) extras: Vec<String>,
}

/// sfdisk-sort-rs uses this Display impl to reconstruct sfdisk output
/// in the form `/dev/sda1 : start= 2048, size= 409600, type=C12A7328-F81F-11D2-BA4B-00A0C93EC93B, uuid=AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE`
impl std::fmt::Display for Partition {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let joined_extras: String = self.extras.join(" ");
        write!(
            f,
            "{0} : start= {1}, {2}",
            self.name, self.start_block, joined_extras
        )
    }
}

impl Partition {
    pub fn redesignate(
        &mut self,
        blk_dev: block::LinuxBlockDevice,
        new_designation: usize,
    ) -> Result<(), String> {
        let (prefix, _part_num) =
            match block::linux_part_prefix_and_part_num(blk_dev, &self.name) {
                Ok((pref, partn)) => (pref, partn),
                Err(err) => return Err(err),
            };

        self.name = format!("{}{}", prefix, new_designation);
        self.designation = new_designation;

        Ok(())
    }
}

#[cfg(test)]
pub mod partition_tests {
    use super::parse;
    use super::Partition;
    use crate::disk::Disk;
    use crate::linux::block;

    use std::collections::HashMap;

    impl Partition {
        pub(crate) fn new_from_start_block(
            start_block: usize,
            blk: block::LinuxBlockDevice,
        ) -> Self {
            let part_name = match blk {
                block::LinuxBlockDevice::NVME => String::from("/dev/nvme0n1p69"),
                block::LinuxBlockDevice::SCSI => String::from("/dev/sda69"),
                block::LinuxBlockDevice::VIRT => String::from("/dev/vda69"),
                block::LinuxBlockDevice::MMCBLK => String::from("/dev/mmcblk69"),
            };
            Partition {
                designation: 69,
                start_block: start_block,
                name: part_name,
                extras: Vec::new(),
            }
        }
    }

    impl Disk {
        pub fn new_disk_without_parts(
            disk_name: &str,
        ) -> Result<(Self, block::LinuxBlockDevice), String> {
            if let Some(correct_linux_device) = block::linux_blk_name(disk_name) {
                let mut this_disk = Disk::default();
                this_disk.name = String::from(disk_name);
                this_disk.linux_block_device = correct_linux_device;
                return Ok((this_disk, correct_linux_device));
            }

            Err(format!(
                "disk name does match known Linux block device name (e.g. sdX, vdX, or nvmeXnY)"
            ))
        }
    }

    // Test if the Display formatted text is indeed parsable.
    #[test]
    fn test_display() {
        let part = Partition {
            designation: 1,
            name: String::from("/dev/sda1"),
            start_block: 69,
            extras: vec![
                String::from("size="),
                String::from("60086239,"),
                String::from("type=0FC63DAF-8483-4772-8E79-3D69D8477DE4,"),
                String::from("uuid=AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE,"),
                String::from("it"),
                String::from("ain't"),
                String::from("me"),
                String::from("babe"),
            ],
        };

        assert!(parse::is_sfdisk_partition_line(&format!("{}", part)));
    }

    // Test if Vec<Partition> can actually be sorted by start_block
    // and that we can actually sort them for a disk
    #[test]
    fn test_sort_by_start_block() {
        let p2048 = Partition::new_from_start_block(2048, block::LinuxBlockDevice::SCSI);
        let p2022 = Partition::new_from_start_block(2022, block::LinuxBlockDevice::SCSI);
        let p1969 = Partition::new_from_start_block(1969, block::LinuxBlockDevice::SCSI);
        let p2069 = Partition::new_from_start_block(2069, block::LinuxBlockDevice::SCSI);

        let mut expecteds = HashMap::new();
        expecteds.insert(0, p1969.clone());
        expecteds.insert(1, p2022.clone());
        expecteds.insert(2, p2048.clone());
        expecteds.insert(3, p2069.clone());

        let (mut sda, _linux_blk) =
            crate::disk::Disk::new_disk_without_parts("/dev/sda").unwrap();

        sda.partitions = vec![p2048, p2069, p2022, p1969];
        sda.partitions.sort_by(|a, b| a.start_block.cmp(&b.start_block));

        for (i, sorted) in sda.partitions.iter().enumerate() {
            let expected = expecteds.get(&i).unwrap();
            assert_eq!(sorted, expected);
        }
    }

    #[test]
    fn test_rename_part() {
        let mut m1 = Partition {
            name: String::from("/dev/mmcblk11p2"),
            designation: 2,
            start_block: 2048,
            extras: vec![String::from("")],
        };

        match m1.redesignate(block::LinuxBlockDevice::MMCBLK, 1) {
            Err(err) => {
                panic!("error redesignating partition: {}", err)
            }
            _ => {}
        }

        assert_eq!(m1.name, "/dev/mmcblk11p1");
        assert_eq!(m1.designation, 1);

        let mut n1 = Partition {
            name: String::from("/dev/nvme0n75p2"),
            designation: 2,
            start_block: 2048,
            extras: vec![String::from("")],
        };

        match n1.redesignate(block::LinuxBlockDevice::NVME, 1) {
            Err(err) => {
                panic!("error redesignating partition: {}", err)
            }
            _ => {}
        }

        assert_eq!(n1.name, "/dev/nvme0n75p1");
        assert_eq!(n1.designation, 1);
    }
}
