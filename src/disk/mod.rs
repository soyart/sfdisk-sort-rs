use super::partition::{Partition, parse};
use crate::linux::block;
use crate::error::RegexCapturesError;

use lazy_static::lazy_static;
use regex::Regex;
use anyhow::{Error, Result, Context};

const SFDISK_DEVICE_NAME_PATTERN: &str = r"(?:device:\s+)(?P<device_name>(?:/dev/).*)";

lazy_static! {
    static ref SFDISK_DEVICE_NAME_REGEX: Regex =
        Regex::new(SFDISK_DEVICE_NAME_PATTERN).unwrap();
}

pub fn is_sfdisk_device_name_line(s: &str) -> bool {
    SFDISK_DEVICE_NAME_REGEX.is_match(s)
}

pub fn parse_sfdisk_device_name_line(s: &str) -> Result<String> {
    let caps = SFDISK_DEVICE_NAME_REGEX.captures(s);
    if caps.is_none() {
        return Err(Error::from(RegexCapturesError)).with_context(|| {
            format!("failed to parse device name from 'device' line: {}", s)
        });
    }
    let caps = caps.unwrap();

    let device_name = caps.name("device_name");
    if device_name.is_none() {
        return Err(Error::from(RegexCapturesError)).with_context(|| {
            format!("failed to parse device name from 'device' line: {}", s)
        });
    }

    Ok(String::from(device_name.unwrap().as_str()))
}

/// Parses the `sfdisk -d` text output into Disk.
pub fn parse_sfdisk_full_disk(prog_input: String) -> Result<Disk> {
    let mut device_name: Option<String> = None;
    let mut header_lines: Vec<String> = Vec::new();
    let mut partitions: Vec<Partition> = Vec::new();

    for (c, input_line) in prog_input.lines().collect::<Vec<&str>>().iter().enumerate() {
        // Parse partition line (will continue)
        if parse::is_sfdisk_partition_line(input_line) {
            let part = match parse::parse_sfdisk_partition_line(input_line) {
                Ok(valid_partition) => valid_partition,
                Err(err) => {
                    return Err(err).with_context(|| {
                        format!("error parsing partition on line {}", c + 1)
                    });
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
                    return Err(err).with_context(|| {
                        format!("error parsing device name on line {}", c + 1)
                    });
                }
            }
        }
        header_lines.push(String::from(*input_line));
    }

    if device_name.is_none() {
        return Err(Error::from(RegexCapturesError))
            .with_context(|| String::from("missing device_name"));
    }
    if header_lines.is_empty() {
        return Err(Error::from(RegexCapturesError))
            .with_context(|| String::from("missing header lines"));
    }

    let device_name = device_name.unwrap();
    let this_disk = match Disk::new(&device_name, header_lines, partitions) {
        Ok(d) => d,
        Err(err) => {
            return Err(err).with_context(|| String::from("error creating new disk"));
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
    ) -> Result<Self> {
        if let Some(correct_linux_device) = block::linux_blk_name(disk_name) {
            return Ok(Disk {
                name: String::from(disk_name),
                linux_block_device: correct_linux_device,
                header_lines,
                partitions,
            });
        }

        Err(Error::from(RegexCapturesError)).with_context(|| format!(
            "disk name does match known Linux block device name (e.g. sdX, vdX, or nvmeXnY)"
        ))
    }

    /// Sorts and reassigns partition name and designation. It assumes first partition starts at 1.
    pub fn rearrange(&mut self) -> Result<(), String> {
        // Sort partition by start_block
        self.partitions
            .sort_by(|a, b| a.start_block.cmp(&b.start_block));

        // Redesignate all partitions based on sorted indices
        for (i, part) in self.partitions.iter_mut().enumerate() {
            if let Some(re) = block::BLK_REGEX.get(&self.linux_block_device) {
                // Check if part.name is a valid Regex.
                if re.captures(&part.name).is_none() {
                    return Err(String::from(format!(
                        "failed to get partition number for {}",
                        &part.name
                    )));
                }

                // Redesignate (update) partition fields to reflect the new sorted name.
                if let Err(err) = part.redesignate(self.linux_block_device, i + 1) {
                    return Err(format!(
                        "error redesignating partition {}: {}",
                        part.name, err
                    ));
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
        let p2048 =
            Partition::new_from_start_block(1, 2048, block::LinuxBlockDevice::SCSI);
        let p2022 =
            Partition::new_from_start_block(2, 2022, block::LinuxBlockDevice::SCSI);
        let p1969 =
            Partition::new_from_start_block(3, 1969, block::LinuxBlockDevice::SCSI);
        let p2069 =
            Partition::new_from_start_block(4, 2069, block::LinuxBlockDevice::SCSI);

        let mut expecteds =
            vec![p1969.clone(), p2022.clone(), p2048.clone(), p2069.clone()];

        let mut sda = Disk {
            name: String::from("/dev/sda"),
            linux_block_device: super::block::LinuxBlockDevice::SCSI,
            header_lines: Vec::new(),
            partitions: vec![p2048, p2069, p2022, p1969],
        };

        if let Err(err) = sda.rearrange() {
            panic!("rearrange failed: {}", err);
        }

        for (i, sorted) in sda.partitions.iter().enumerate() {
            let expected = expecteds.get_mut(i).unwrap();
            // Update expected designation and names before asserting
            expected.designation = i + 1;
            expected.name = format!("{}{}", sda.name, expected.designation);

            assert_eq!(sorted, expected);
        }
    }
}
