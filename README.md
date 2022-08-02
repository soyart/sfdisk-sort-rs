# sfdisk-sort-rs
sfdisk-sort-rs is a text processing program for `sfdisk -d` dump output. It is a Rust clone of [`sfdisk-sort-go`](https://github.com/artnoi43/sfdisk-sort-go). Basically, it rearranges and renames your `sfdisk -d` partition output by start block, and prints the sorted (pretty) disk out for `sfdisk` to read the text and apply it back to the partition table.

This program does NOT alter or touch your disk partition table, instead it just outputs the text for `sfdisk` to do so.

Unlike the Go version, this Rust program gets its input only from stdin. It's also implemented differently than the Go version in that this program uses regex to parse text.

To rearrange an sfdisk output partitions for `/dev/sdb` by start block, you just pipe the `sfdisk -d` output to the program:

```
$ sudo sfdisk -d /dev/sdb | sfdisk-sort-rs > sdb.parttab.bkp;
$
$ # Read this file back to partition table.
$ # DO THIS AT YOUR OWN DISK.
$
$ sudo sfdisk /dev/sdb < sdb.parttab.bkp;
$
$ # If the above command does not work, try:
$
$ sudo sfdisk --no-reread -f /dev/sdb < sdb.new;
```