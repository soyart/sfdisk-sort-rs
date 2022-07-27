use super::Partition;
use std::collections::HashMap;

pub struct LineParser<'a> {
    line: &'a str,
    map: ParseMap<'a>,
}

#[derive(Debug)]
pub enum SfdiskPartitionInfo {
    PartName,
    PartNameSep,
    StartText,
    StartBlock,
}

pub type ParseMap<'a> = HashMap<usize, (SfdiskPartitionInfo, Option<&'a str>)>;

impl<'a> LineParser<'a> {
    pub fn new(line: &'a str, map: ParseMap<'a>) -> Self {
        LineParser { line, map }
    }

    pub fn foo(&self, base_name: &'a str) -> Result<Partition, String> {
        let line_fields: std::str::SplitWhitespace = self.line.split_whitespace();
        let fields = line_fields.into_iter().collect::<Vec<&str>>();

        let mut partition = Partition::default();
        let mut extras: Vec<String> = Vec::new();

        for (i, field) in fields.iter().enumerate() {
            let expected = self.map.get(&(i + 1));
            match expected {
                None => {
                    extras.push(String::from(*field));
                }
                Some((data_key, expected_value)) => match data_key {
                    SfdiskPartitionInfo::PartName => {
                        if let Some(part_num_str) =
                            field.split(base_name).collect::<Vec<&str>>().get(1)
                        {
                            match str::parse::<usize>(part_num_str) {
                                Ok(part_num) => {
                                    partition.designation = part_num;
                                    partition.name = String::from(*field);
                                    continue;
                                }
                                Err(err) => {
                                    return Err(format!(
                                        "failed to parse partition number {} to usize: {}",
                                        part_num_str, err
                                    ))
                                }
                            }
                        }
                        return Err(format!(
                            "bad partition name {}: missing partition number",
                            field
                        ));
                    }
                    SfdiskPartitionInfo::PartNameSep => {
                        let separator = &expected_value.unwrap();
                        if field != separator {
                            return Err(format!("expecting {} for the Separator", separator));
                        }
                    }
                    SfdiskPartitionInfo::StartText => {
                        let start_text = &expected_value.unwrap();
                        if field != start_text {
                            return Err(format!("expecting {} for the StartText", start_text));
                        }
                    }
                    SfdiskPartitionInfo::StartBlock => {
                        if let Some(start_block_str) =
                            field.split(",").into_iter().collect::<Vec<&str>>().get(0)
                        {
                            match str::parse::<u64>(start_block_str) {
                                Ok(start_block) => partition.start_block = start_block,
                                Err(err) => {
                                    return Err(format!(
                                        "failed to parse {} to u64: {}",
                                        start_block_str, err
                                    ))
                                }
                            }
                        }
                    }
                },
            }
        }

        partition.extras = extras;
        Ok(partition)
    }
}
