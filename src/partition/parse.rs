use super::Partition;
use lazy_static::lazy_static;
use regex::Regex;

const SFDISK_PARTITION_LINE_PATTERN: &str = r"(?P<full_path>/dev/(\w+(?P<part_num>\d+)))\s+:\s+(:?start=\s+)(?P<start_block>\d+)[,](?P<rest>.*)";

lazy_static! {
    static ref PARTITION_LINE_REGEX: Regex = Regex::new(SFDISK_PARTITION_LINE_PATTERN).unwrap();
}

pub fn is_sfdisk_partition_line<'a>(line: &'a str) -> bool {
    PARTITION_LINE_REGEX.is_match(line)
}

pub fn parse_sfdisk_partition_line<'a>(line: &'a str) -> Result<Partition, String> {
    let caps = PARTITION_LINE_REGEX.captures(line);
    if caps.is_none() {
        return Err(String::from("bad line"));
    }

    let mut part = Partition::default();
    let mut extras: Vec<String> = Vec::new();
    let caps = caps.unwrap();

    if let Some(full_path) = caps.name("full_path") {
        part.name = String::from(full_path.as_str());
    } else {
        return Err(String::from("missing full partition path"));
    }

    if let Some(part_num) = caps.name("part_num") {
        let part_num = part_num.as_str();
        match str::parse::<usize>(part_num) {
            Ok(num) => {
                part.designation = num;
            }
            Err(err) => {
                return Err(format!(
                    "error parsing partition number {} to usize: {}",
                    part_num, err
                ))
            }
        }
    } else {
        return Err(String::from("missing partition number"));
    }

    if let Some(start_block) = caps.name("start_block") {
        let start_block = start_block.as_str();
        match str::parse::<usize>(start_block) {
            Ok(num) => {
                part.start_block = num;
            }
            Err(err) => {
                return Err(format!(
                    "error parsing partition start block {} to usize: {}",
                    start_block, err
                ))
            }
        }
    } else {
        return Err(String::from("missing full partition start block"));
    }

    if let Some(rest) = caps.name("rest") {
        let splited: std::str::SplitWhitespace = rest.as_str().split_whitespace();
        for extra in splited.into_iter().collect::<Vec<&str>>().iter() {
            extras.push(String::from(*extra));
        }
    } else {
        return Err(String::from("missing the rest of the line"));
    }

    part.extras = extras;

    Ok(part)
}

#[cfg(test)]
mod test_parse {
    use super::Partition;
    use super::{parse_sfdisk_partition_line, SFDISK_PARTITION_LINE_PATTERN};
    use crate::linux::block;
    use crate::partition::parse::is_sfdisk_partition_line;

    use lazy_static::lazy_static;

    use std::collections::HashMap;

    lazy_static! {
        static ref KEYS: Vec<&'static str> = vec!["full_path", "part_num", "start_block"];
    }

    #[test]
    fn test_regex() {
        let s = "/dev/nvme0n1p1 : start=        2048, size=      409600, type=C12A7328-F81F-11D2-BA4B-00A0C93EC93B, uuid=AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE";
        let re = regex::Regex::new(SFDISK_PARTITION_LINE_PATTERN).unwrap();
        let caps = re.captures(s);
        assert!(caps.is_some());

        let caps = caps.unwrap();
        for key in KEYS.iter() {
            assert!(caps.name(key).is_some());
        }
    }

    #[test]
    fn test_parse() {
        let m: HashMap<(&str, &str, block::LinuxBlockDevice), Partition> = HashMap::from([
            (
                (
                    "/dev/sda",
                    "/dev/sda1 : start=  2048, size= 60086239, type=0FC63DAF-8483-4772-8E79-3D69D8477DE4, uuid=AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE,    it ain't me    babe",
                    block::LinuxBlockDevice::SCSI,
                ),
                    Partition {
                        designation: 1,
                        start_block: 2048,
                        name: String::from("/dev/sda1"),
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
                },
            ),
            (
                (
                    "/dev/nvme0n1",
                    "/dev/nvme0n1p1 : start=  2048, size= 60086239, type=0FC63DAF-8483-4772-8E79-3D69D8477DE4, uuid=AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE,    it ain't me    babe",
                    block::LinuxBlockDevice::NVME,
                ),
                    Partition {
                        designation: 1,
                        start_block: 2048,
                        name: String::from("/dev/nvme0n1p1"),
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
                },
            ),
        ]);

        for (test_tuple, expected_part) in m
            .iter()
            .collect::<Vec<(&(&str, &str, block::LinuxBlockDevice), &Partition)>>()
        {
            // let disk_name = test_tuple.0;
            let part_line = test_tuple.1;
            assert!(is_sfdisk_partition_line(part_line));
            // let this_linux_name = test_tuple.2;
            let parsed = match parse_sfdisk_partition_line(part_line) {
                Ok(part) => part,
                Err(err) => {
                    println!("error parsing: {}", err);
                    Partition::default()
                }
            };

            if parsed != *expected_part {
                assert_eq!(expected_part.designation, parsed.designation);
                assert_eq!(expected_part.start_block, parsed.start_block);
                assert_eq!(expected_part.name, parsed.name);
                assert_eq!(expected_part.extras, parsed.extras);
            }
        }
    }
}
