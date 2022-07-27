
#[cfg(test)]
mod parser_tests {
    use crate::partition::parser::{ParseMap, LineParser, SfdiskPartitionInfo as info};
    use crate::partition::Partition;
    
    #[test]
    fn test_parse() {
        let sfdisk_output_disk_line = "/dev/sda1 : start=  2048, size= 60086239, type=0FC63DAF-8483-4772-8E79-3D69D8477DE4, uuid=AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE,    it ain't me    babe";
        let test_disk = Partition {
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
        };
        let mut parse_map = ParseMap::new();
        parse_map.insert(1, (info::PartName, Some("/dev/sda")));
        parse_map.insert(2, (info::PartNameSep, Some(":")));
        parse_map.insert(3, (info::StartText, Some("start=")));
        parse_map.insert(4, (info::StartBlock, None));

        let line_parser = LineParser::new(sfdisk_output_disk_line, parse_map);
        let parsed = match line_parser.foo("/dev/sda") {
            Ok(part) => part,
            Err(err) => {
                println!("error parsing: {}", err);
                Partition::default()
            }
        };

        if parsed != test_disk {
            assert_eq!(test_disk.designation, parsed.designation);
            assert_eq!(test_disk.start_block, parsed.start_block);
            assert_eq!(test_disk.name, parsed.name);
            assert_eq!(test_disk.extras, parsed.extras);
        }
    }

    #[test]
    fn test_header() {
        let sfdisk_output_header = "label: gpt
label-id: 12345678-F226-1234-5678-E55555555555
device: /dev/nvme0n1
unit: sectors
first-lba: 2048
last-lba: 60088286
sector-size: 512";
        let parse_map: ParseMap = ParseMap::new();
        let line_parser = LineParser::new(sfdisk_output_header, parse_map);
        let parsed = line_parser.foo("/dev/sda");
        println!("{:?}", parsed);
    }
}
