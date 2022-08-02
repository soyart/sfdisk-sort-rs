use lazy_static::lazy_static;
use regex::Regex;

use std::collections::HashMap;

/// Represents my commonly used block device names.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum LinuxBlockDevice {
    /// SCSI, ATA, and SATA
    SCSI,
    /// Virtual devices, e.g. '/dev/vdX' and '/dev/xvd'
    VIRT,
    /// eMMC storage, SD cards, and other NAND/NOR flash
    MMCBLK,
    /// NVMe devices
    NVME,
}

const SCSI_REGEX: &str = r"sd[a-z]";
const VIRT_REGEX: &str = r"vd[a-z]";
const NVME_REGEX: &str = r"nvme\d+[n]\d+";
const MMBCLK_REGEX: &str = r"mmcblk\d+";

const SCSI_PART_REGEX: &str = r"(?P<prefix>/dev/sd[a-z])(?P<part_num>\d+)";
const VIRT_PART_REGEX: &str = r"(?P<prefix>/dev/vd[a-z])(?P<part_num>\d+)";
const NVME_PART_REGEX: &str = r"(?P<prefix>/dev/nvme\d+[n]\d+[p])(?P<part_num>\d+)";
const MMBCLK_PART_REGEX: &str = r"(?P<prefix>/dev/mmcblk\d+[p])(?P<part_num>\d+)";

lazy_static! {
    pub static ref BLK_REGEX: HashMap<LinuxBlockDevice, Regex> = HashMap::from([
        (LinuxBlockDevice::SCSI, Regex::new(SCSI_REGEX).unwrap()),
        (LinuxBlockDevice::VIRT, Regex::new(VIRT_REGEX).unwrap()),
        (LinuxBlockDevice::MMCBLK, Regex::new(MMBCLK_REGEX).unwrap()),
        (LinuxBlockDevice::NVME, Regex::new(NVME_REGEX).unwrap()),
    ]);

    /// These Regexes are used during rearranging/redesignation
    /// by extracting all the text in the device name before the partition number,
    /// in `nvme` and `mmcblk` cases, the `prefix` also includes the 'p'.
    pub static ref BLK_PART_REGEX: HashMap<LinuxBlockDevice, Regex> = HashMap::from([
        (LinuxBlockDevice::SCSI, Regex::new(SCSI_PART_REGEX).unwrap()),
        (LinuxBlockDevice::VIRT, Regex::new(VIRT_PART_REGEX).unwrap()),
        (LinuxBlockDevice::MMCBLK, Regex::new(MMBCLK_PART_REGEX).unwrap()),
        (LinuxBlockDevice::NVME, Regex::new(NVME_PART_REGEX).unwrap()),
    ]);
}

pub fn linux_blk_name(device_name: &str) -> Option<LinuxBlockDevice> {
    for (disk_type, re) in BLK_REGEX
        .clone()
        .into_iter()
        .collect::<Vec<(LinuxBlockDevice, Regex)>>()
    {
        if re.is_match(device_name) {
            return Some(disk_type);
        }
    }

    None
}

/// This function extracts the partition prefix and the partition number.
/// e.g. `/dev/sda1000` will be extracted as tuple `("sda", "1")`,
/// while `/dev/mmcblk10p2` will be extracted as `("mmcblk10", "2")`.
pub fn linux_part_prefix_and_part_num(
    blk_dev: LinuxBlockDevice,
    part_name: &str,
) -> Result<(&str, &str), String> {
    let re = BLK_PART_REGEX.get(&blk_dev).unwrap();
    let caps = re.captures(part_name);
    if caps.is_none() {
        return Err(format!(
            "failed to match {:?} prefix and partition number for {}",
            blk_dev, part_name
        ));
    }

    let caps = caps.unwrap();
    let prefix = caps.name("prefix");
    if prefix.is_none() {
        return Err(format!(
            "missing prefix for {:?} (partition name {})",
            blk_dev, part_name
        ));
    }
    let part_num = caps.name("part_num");
    if part_num.is_none() {
        return Err(format!(
            "missing partition number for {:?} (partition name {})",
            blk_dev, part_name
        ));
    }

    Ok((prefix.unwrap().as_str(), part_num.unwrap().as_str()))
}

impl core::fmt::Debug for LinuxBlockDevice {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            self::LinuxBlockDevice::SCSI => {
                write!(f, "{}", "SCSI")
            }
            self::LinuxBlockDevice::VIRT => {
                write!(f, "{}", "VIRT")
            }
            self::LinuxBlockDevice::MMCBLK => {
                write!(f, "{}", "MMCBLK")
            }
            self::LinuxBlockDevice::NVME => {
                write!(f, "{}", "NVME")
            }
        }
    }
}

impl Default for LinuxBlockDevice {
    fn default() -> Self {
        Self::SCSI
    }
}

#[cfg(test)]
mod disk_tests {
    use super::{linux_blk_name, linux_part_prefix_and_part_num, LinuxBlockDevice as ns};
    use std::collections::HashMap;

    #[test]
    fn test_diskname() {
        let sda1000 = "/dev/sda1000";
        let sdc1 = "/dev/sdc1";
        let vda1 = "/dev/vda1";
        let mmcblk10p20 = "/dev/mmcblk10p20";
        let nvme0n1p1 = "/dev/nvme0n1p1";

        let expected_linux_blk_name = HashMap::from([
            (sda1000, Some(ns::SCSI)),
            (sdc1, Some(ns::SCSI)),
            (nvme0n1p1, Some(ns::NVME)),
            (mmcblk10p20, Some(ns::MMCBLK)),
            (vda1, Some(ns::VIRT)),
            ("/dev/nvme1", None),
            ("/dev/sd1", None),
        ]);

        for device in expected_linux_blk_name.iter() {
            let result_ns = linux_blk_name(*device.0);
            assert_eq!(result_ns, *expected_linux_blk_name.get(*device.0).unwrap());
        }

        let expected_captures: HashMap<(&str, ns), (&str, &str)> = HashMap::from([
            ((sda1000, ns::SCSI), ("/dev/sda", "1000")),
            ((sdc1, ns::SCSI), ("/dev/sdc", "1")),
            ((vda1, ns::VIRT), ("/dev/vda", "1")),
            ((mmcblk10p20, ns::MMCBLK), ("/dev/mmcblk10p", "20")),
            ((nvme0n1p1, ns::NVME), ("/dev/nvme0n1p", "1")),
        ]);

        for (test_tuple, expected_tuple) in expected_captures.iter() {
            let result_tuple =
                linux_part_prefix_and_part_num(test_tuple.1, test_tuple.0).unwrap();
            assert_eq!(result_tuple, *expected_tuple);
        }
    }
}
