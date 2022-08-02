pub mod disk;
pub mod partition;
pub mod linux;

use std::io::{self, Read};

fn main() {
    let prog_input = get_stdin_string();
    let mut this_disk = match parse_full_disk(prog_input) {
        Ok(the_disk) => the_disk,
        Err(err) => {
            eprintln!("failed to parse full disk: {}", err);
            panic!("{}", err)
        }
    };

    this_disk.rearrange().expect("failed to rearrange disk partitions");
    print_disk(this_disk);
}

fn print_disk(this_disk: disk::Disk) {
    for header_line in this_disk.header_lines {
        println!("{}", header_line);
    }
    for each_partition in this_disk.partitions {
        println!("{}", each_partition);
    }
}

fn get_stdin_string() -> String {
    let mut buf = String::new();
    let mut stdin = io::stdin();
    match stdin.read_to_string(&mut buf) {
        Err(err) => eprintln!("error reading from stdin {}", err),
        _ => {},
    }
    
    buf
}

fn parse_full_disk(prog_input: String) -> Result<disk::Disk, String> {
    let mut device_name: Option<String> = None;
    let mut header_lines: Vec<String> = Vec::new();
    let mut partitions: Vec<partition::Partition> = Vec::new();
    for (c, input_line) in prog_input.lines().collect::<Vec<&str>>().iter().enumerate() {
        // Parse partition line (will continue)
        if partition::parse::is_sfdisk_partition_line(input_line) {
            let part = match partition::parse::parse_sfdisk_partition_line(input_line) {
                Ok(valid_partition) => valid_partition,
                Err(err) => {
                    eprintln!("error parsing partition on line {}", c+1);
                    return Err(err);
                },
            };
            partitions.push(part);
            continue
        }

        // Parse device name (part of so-called 'header lines'), won't continue
        if disk::is_sfdisk_device_name_line(input_line) {
            match disk::parse_sfdisk_device_name_line(input_line) {
                Ok(text) => {
                    device_name = Some(text);
                }
                Err(err) => {
                    eprintln!("error parsing device name on line {}", c+1);
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
    let this_disk = match disk::Disk::new(&device_name, header_lines, partitions) {
        Ok(d) => d,
        Err(err) => {
            eprintln!("error creating new disk {}", err);
            return Err(err);
        },
    };

    Ok(this_disk)
}

#[cfg(test)]
mod test_main {
    use crate::{parse_full_disk};

    #[test]
    fn test_prog() {
        use std::fs;

        let ugly_disk_file = "./sfdisk_output_ugly.txt";
        let pretty_disk_file = "./sfdisk_output.txt";
        let ugly_disk_input = fs::read_to_string(ugly_disk_file).expect("failed to read ugly test text file");
        let pretty_disk_input = fs::read_to_string(pretty_disk_file).expect("failed to read pretty test text file");

        let mut ugly_disk = parse_full_disk(String::from(ugly_disk_input)).unwrap();
        ugly_disk.rearrange().expect("failed to rearrange");
        let pretty_disk = parse_full_disk(String::from(pretty_disk_input)).unwrap();

        assert_eq!(ugly_disk, pretty_disk);
    }
}
