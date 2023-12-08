use clap::Parser;
use std::fs;
use std::io::{Read, Write};

#[derive(Parser)]
#[command(version)]
struct Args {
    /// merged hex file
    out: String,
    /// hex files to merge
    ins: Vec<String>,
}

fn main() {
    let cli = Args::parse();

    // store start linear address
    let mut start_linear_address = None;
    // prepare tree of records
    let mut in_records = vec![];

    for inhex in cli.ins {
        // open whole file and dump it into string
        let mut file = fs::File::open(inhex).expect("unable to open input file");
        let mut data = String::new();
        file.read_to_string(&mut data)
            .expect("unable to read the whole file");

        // wrap ihex::Reader on string
        let ihex = ihex::Reader::new(&data);

        let mut upper_address = 0u32;
        // iterate through ihex records
        for record in ihex {
            let record = record.expect("error while parsing ihex file");
            match record {
                ihex::Record::ExtendedLinearAddress(addr) => {
                    upper_address = (addr as u32) << 16;
                }
                ihex::Record::Data { offset, value } => {
                    let address = upper_address | (offset as u32);
                    in_records.push((address, value));
                }
                ihex::Record::StartLinearAddress(addr) => {
                    if let Some(paddr) = start_linear_address {
                        if addr != paddr {
                            eprintln!("warning: Record StartLinearAddress differs between files, using first one");
                        }
                    } else {
                        start_linear_address = Some(addr);
                    }
                }
                ihex::Record::EndOfFile => {
                    // nothing to do
                }
                _ => {
                    panic!("unmanaged ihex record : {:?}", record)
                }
            }
        }
    }

    if in_records.len() == 0 {
        eprintln!("no records to write in output ihex file");
        return;
    }

    // sort all records by addresses
    in_records.sort_by_key(|(addr, _)| *addr);

    // prepare records for output file
    let mut out_records = vec![];
    out_records.push(ihex::Record::StartLinearAddress(0));

    // get first record to store starting upper address
    let (addr, _) = in_records[0];
    let mut segment_upper_address = addr >> 16;

    // iterate through records and push them to output vector
    for (addr, value) in in_records.iter() {
        let upper = addr >> 16;

        // write extend linear address record if it has changed
        if upper != segment_upper_address {
            out_records.push(ihex::Record::ExtendedLinearAddress(upper as u16));
        }

        let offset = addr & 0xffff;
        out_records.push(ihex::Record::Data {
            offset: offset as u16,
            value: value.clone(),
        });
        segment_upper_address = upper;
    }

    // push EOF record
    out_records.push(ihex::Record::EndOfFile);

    // create ihex file
    let data = ihex::create_object_file_representation(&out_records)
        .expect("error while create ihex object");

    // write output file
    let mut file = fs::File::create(cli.out).expect("unable to create output file");
    file.write_all(data.as_bytes())
        .expect("unable to write ihex object to file");
}
