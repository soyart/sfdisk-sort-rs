pub mod disk;
pub mod partition;

// Not implemented yet
fn main() {
    let part = partition::Partition::default();
    let disk = disk::Disk::default();
    println!("partition: {:?}, disk: {:?}", part, disk);
}
