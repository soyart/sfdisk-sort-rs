pub mod parse;

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

impl std::fmt::Display for Partition {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let joined_extras: String = self.extras.join(" ");
        write!(f, "{0} : start= {1}, {2}", self.name, self.start_block, joined_extras)
        // /dev/sda1 : start= 2048, size= 409600, type=C12A7328-F81F-11D2-BA4B-00A0C93EC93B, uuid=AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE
    }
}

#[cfg(test)]
pub mod partition_tests {
    use super::Partition;
    use super::parse;
    use std::collections::HashMap;
    impl Partition {
        fn new_from_start_block(start_block: usize) -> Self {
            let mut this = Self::default();
            this.start_block = start_block;
            this
        }
    }

    #[test]
    fn test_display() {
        let part = Partition{
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
            ]
        };

        assert!(parse::is_sfdisk_partition_line(&format!("{}", part)));
    }

    #[test]
    fn test_sort_by_start_block() {
        let p2048 = Partition::new_from_start_block(2048);
        let p2022 = Partition::new_from_start_block(2022);
        let p1969 = Partition::new_from_start_block(1969);
        let p2069 = Partition::new_from_start_block(2069);

        let mut expecteds = HashMap::new();
        expecteds.insert(0, p1969.clone());
        expecteds.insert(1, p2022.clone());
        expecteds.insert(2, p2048.clone());
        expecteds.insert(3, p2069.clone());

        let mut partitions = vec![p2048, p2069, p2022, p1969];
        partitions.sort_by(|a, b| a.start_block.cmp(&b.start_block));

        for (i, sorted) in partitions.iter().enumerate() {
            let expected = expecteds.get(&i).unwrap();
            assert_eq!(sorted, expected);
        }
    }
}
