mod disk;
mod linux;
mod partition;

use std::io::{self, Read};

fn main() {
    let prog_input = match get_stdin_string() {
        Ok(s) => s,
        Err(err) => {
            eprintln!("{}", err);
            panic!("{}", err);
        }
    };
    let mut this_disk = match disk::parse_full_disk(prog_input) {
        Ok(the_disk) => the_disk,
        Err(err) => {
            eprintln!("failed to parse full disk: {}", err);
            panic!("{}", err);
        }
    };

    // Rearrange disk partitions by start_block
    this_disk
        .rearrange()
        .expect("failed to rearrange disk partitions");

    print_disk(this_disk);

    println!();
    println!("# See https://github.com/artnoi43/sfdisk-sort-rs/blob/main/README.md to see what to do whith this output");
}

/// Prints disk in `sfdisk -d` dump format. `disk::Disk` does not implements Display,
/// so this is how the program prints a `disk::Disk`
fn print_disk(this_disk: disk::Disk) {
    for header_line in this_disk.header_lines {
        println!("{}", header_line);
    }
    for each_partition in this_disk.partitions {
        println!("{}", each_partition);
    }
}

fn get_stdin_string() -> Result<String, String> {
    let mut buf = String::new();
    let mut stdin = io::stdin();

    match stdin.read_to_string(&mut buf) {
        Err(err) => {
            let err_msg = format!("error reading from stdin {}", err);
            return Err(err_msg);
        }
        _ => {
            return Ok(buf);
        }
    }
}

#[cfg(test)]
mod test_main {
    use crate::disk::parse_full_disk;

    #[test]
    fn test_prog() {
        use std::fs;

        let ugly_disk_file = "./assets/sfdisk_output_ugly.txt";
        let pretty_disk_file = "./assets/sfdisk_output.txt";

        let ugly_disk_input = fs::read_to_string(ugly_disk_file)
            .expect("failed to read ugly test text file");

        let pretty_disk_input = fs::read_to_string(pretty_disk_file)
            .expect("failed to read pretty test text file");

        let mut ugly_disk = parse_full_disk(String::from(ugly_disk_input))
            .unwrap();

        let pretty_disk = parse_full_disk(String::from(pretty_disk_input))
            .unwrap();

        ugly_disk.rearrange().expect("failed to rearrange");

        assert_eq!(ugly_disk, pretty_disk);
    }
}
