use crate::linux::block;
use super::partition::{Partition, parse};

use lazy_static::lazy_static;
use regex::Regex;

const SFDISK_DEVICE_NAME_PATTERN: &str = r"(?:device:\s+)(?P<device_name>(?:/dev/).*)";

lazy_static! {
    static ref SFDISK_DEVICE_NAME_REGEX: Regex =
        Regex::new(SFDISK_DEVICE_NAME_PATTERN).unwrap();
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

/// Parses the `sfdisk -d` text output into Disk.
pub fn parse_full_disk(prog_input: String) -> Result<Disk, String> {
    let mut device_name: Option<String> = None;
    let mut header_lines: Vec<String> = Vec::new();
    let mut partitions: Vec<Partition> = Vec::new();

    for (c, input_line) in prog_input.lines().collect::<Vec<&str>>().iter().enumerate() {
        // Parse partition line (will continue)
        if parse::is_sfdisk_partition_line(input_line) {
            let part = match parse::parse_sfdisk_partition_line(input_line) {
                Ok(valid_partition) => valid_partition,
                Err(err) => {
                    eprintln!("error parsing partition on line {}", c + 1);
                    return Err(err);
                }
            };
            partitions.push(part);
            continue;
        }

        // Parse device name (part of so-called 'header lines'), won't continue
        if is_sfdisk_device_name_line(input_line) {
            match parse_sfdisk_device_name_line(input_line) {
                Ok(text) => {
                    device_name = Some(text);
                }
                Err(err) => {
                    eprintln!("error parsing device name on line {}", c + 1);
                    return Err(err);
                }
            }
        }
        header_lines.push(String::from(*input_line));
    }

    if device_name.is_none() {
        panic!("fatal: missing device name")
    }
    if header_lines.is_empty() {
        panic!("fatal: missing header lines")
    }

    let device_name = device_name.unwrap();
    let this_disk = match Disk::new(&device_name, header_lines, partitions) {
        Ok(d) => d,
        Err(err) => {
            eprintln!("error creating new disk {}", err);
            return Err(err);
        }
    };

    Ok(this_disk)
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

                // Redesignate (update) partition fields to reflect the new sorted name.
                match part.redesignate(self.linux_block_device, i + 1) {
                    Ok(_) => {
                        // Overwrite with redesignated partition
                        self.partitions[i] = part;
                    }
                    Err(err) => {
                        return Err(format!(
                            "error redesignating partition {}: {}",
                            part.name, err
                        ));
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
            partitions: vec![p2048, p2069, p2022, p1969],
        };

        match sda.rearrange() {
            Err(err) => {
                panic!("rearrange failed: {}", err);
            }
            _ => {}
        }

        for (i, sorted) in sda.partitions.iter().enumerate() {
            let _expected = expecteds.get(&i).unwrap();
            assert_eq!(sorted.designation, i + 1);
        }
    }
}
