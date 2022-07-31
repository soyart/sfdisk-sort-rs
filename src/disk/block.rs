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

const SCSI_REGEX: &str = r"(sd)[a-z]";
const VIRT_REGEX: &str = r"(vd)[a-z]";
const MMBCLK_REGEX: &str = r"(mmcblk)\d+[p]";
const NVME_REGEX: &str = r"(nvme)\d+[n]\d+[p]";

lazy_static! {
    static ref BLK_REGEX: HashMap<LinuxBlockDevice, Regex> = HashMap::from([
        (LinuxBlockDevice::SCSI, Regex::new(SCSI_REGEX).unwrap(),),
        (LinuxBlockDevice::VIRT, Regex::new(VIRT_REGEX).unwrap(),),
        (LinuxBlockDevice::NVME, Regex::new(NVME_REGEX).unwrap(),),
        (LinuxBlockDevice::MMCBLK, Regex::new(MMBCLK_REGEX).unwrap(),),
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

impl core::fmt::Debug for LinuxBlockDevice {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            self::LinuxBlockDevice::SCSI => write!(f, "{}", "SCSI"),
            self::LinuxBlockDevice::VIRT => write!(f, "{}", "VIRT"),
            self::LinuxBlockDevice::MMCBLK => write!(f, "{}", "MMCBLK"),
            self::LinuxBlockDevice::NVME => write!(f, "{}", "NVME"),
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
    use super::{linux_blk_name, LinuxBlockDevice as ns};
    use std::collections::HashMap;

    #[test]
    fn test_diskname() {
        let sda1 = "/dev/sda";
        let sdc1 = "/dev/sdc";
        let nvme0n1p1 = "/dev/nvme0n1";
        let vda1 = "/dev/vda";

        let expected_linux_blk_name = HashMap::from([
            (sda1, Some(ns::SCSI)),
            (sdc1, Some(ns::SCSI)),
            (nvme0n1p1, Some(ns::NVME)),
            (vda1, Some(ns::VIRT)),
            ("/dev/nvme1", None),
            ("/dev/sd1", None),
        ]);

        for disk in expected_linux_blk_name.iter() {
            let result_ns = linux_blk_name(*disk.0);
            assert_eq!(result_ns, *expected_linux_blk_name.get(*disk.0).unwrap());
        }
    }
}
