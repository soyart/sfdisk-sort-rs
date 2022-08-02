use crate::linux::block;

use super::partition::Partition;

use lazy_static::lazy_static;
use regex::Regex;

const SFDISK_DEVICE_NAME_PATTERN: &str = r"(?:device:\s+)(?P<device_name>(?:/dev/).*)";

lazy_static! {
    static ref SFDISK_DEVICE_NAME_REGEX: Regex = Regex::new(SFDISK_DEVICE_NAME_PATTERN).unwrap();
}

pub fn is_sfdisk_device_name_line(s: &str) -> bool {
    SFDISK_DEVICE_NAME_REGEX.is_match(s)
}

pub fn parse_sfdisk_device_name_line(s: &str) -> Result<String, String> {
    let caps = SFDISK_DEVICE_NAME_REGEX.captures(s);
    if caps.is_none() {
        return Err(format!(
            "failed to parse device name from 'device' line: {}",
            s
        ));
    }
    let caps = caps.unwrap();
    if let Some(device_name) = caps.name("device_name") {
        return Ok(String::from(device_name.as_str()));
    }
    return Err(format!(
        "failed to parse device name from 'device' line: {}",
        s
    ));
}

#[derive(Default, Debug, PartialEq)]
pub struct Disk {
    pub name: String,
    pub linux_block_device: block::LinuxBlockDevice,
    pub header_lines: Vec<String>,
    pub partitions: Vec<Partition>,
}

impl Disk {
    pub fn new(
        disk_name: &str,
        header_lines: Vec<String>,
        partitions: Vec<Partition>,
    ) -> Result<Self, String> {
        if let Some(correct_linux_device) = block::linux_blk_name(disk_name) {
            return Ok(Disk {
                name: String::from(disk_name),
                linux_block_device: correct_linux_device,
                header_lines,
                partitions,
            });
        }
        Err(format!(
            "disk name does match known Linux block device name (e.g. sdX, vdX, or nvmeXnY)"
        ))
    }

    /// Sorts and reassigns partition name and designation. It assumes first partition starts at 1.
    pub fn rearrange(&mut self) -> Result<(), String> {
        // Sort partition by start_block
        self.partitions
            .sort_by(|a, b| a.start_block.cmp(&b.start_block));
        let l = self.partitions.len();
        // TODO: fix this iteration
        for i in 0..l {
            let part = self.partitions.get(i);
            let mut part = part.unwrap().clone();

            if let Some(re) = block::BLK_REGEX.get(&self.linux_block_device) {
                let caps = re.captures(&part.name);
                if caps.is_none() {
                    return Err(String::from(format!(
                        "failed to get partition number for {}",
                        &part.name
                    )));
                }
                match part.redesignate(self.linux_block_device, i + 1) {
                    Ok(_) => {
                        // Overwrite with redesignated partition
                        self.partitions[i] = part;
                    }
                    Err(err) => {
                        return Err(format!("error redesignating partition {}", err));
                    }
                }
            } else {
                return Err(String::from(
                    "missing regex for parsing partition prefix and number",
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod disk_test {
    use super::{block, Disk, SFDISK_DEVICE_NAME_REGEX};
    use crate::partition::Partition;

    use std::collections::HashMap;

    #[test]
    fn test_device_name() {
        let names = vec![
            "device: /dev/sda",
            "device: /dev/vdz",
            "device: /dev/mmcblk2",
            "device: /dev/nvme17n1",
        ];
        for name in names {
            assert!(SFDISK_DEVICE_NAME_REGEX.is_match(name));
        }
    }

    #[test]
    fn test_rearrange() {
        let p2048 = Partition::new_from_start_block(2048, block::LinuxBlockDevice::SCSI);
        let p2022 = Partition::new_from_start_block(2022, block::LinuxBlockDevice::SCSI);
        let p1969 = Partition::new_from_start_block(1969, block::LinuxBlockDevice::SCSI);
        let p2069 = Partition::new_from_start_block(2069, block::LinuxBlockDevice::SCSI);

        let mut expecteds = HashMap::new();
        expecteds.insert(0, p1969.clone());
        expecteds.insert(1, p2022.clone());
        expecteds.insert(2, p2048.clone());
        expecteds.insert(3, p2069.clone());

        let mut sda = Disk {
            name: String::from("/dev/sda"),
            linux_block_device: super::block::LinuxBlockDevice::SCSI,
            header_lines: Vec::new(),
            partitions: Vec::new(),
        };
        sda.partitions = vec![p2048, p2069, p2022, p1969];
        match sda.rearrange() {
            Ok(_) => {}
            Err(err) => {
                panic!("rearrange failed: {}", err);
            }
        }

        for (i, sorted) in sda.partitions.iter().enumerate() {
            let _expected = expecteds.get(&i).unwrap();
            assert_eq!(sorted.designation, i + 1);
        }
    }
}
