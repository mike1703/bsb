use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// the CSV format of a `Field`
#[derive(Deserialize)]
struct Field {
    id: u32,
    name: String,
    prognr: usize,
    data_type: String,
    path: String,
}

/// location of the bsb field definition field
const FIELD_DB_CSV: &'static str = "bsb-fields.csv";
/// location of the generated rust file
const FIELD_DB_RS: &'static str = "field_db.rs";

fn main() {
    // Use the csv crate to parse the field definition database.
    let mut rdr = csv::Reader::from_path(FIELD_DB_CSV)
        .expect(&format!("Failed to read CSV file {FIELD_DB_CSV}"));

    // Use phf to create a static map for the fields defined in `FIELD_DB_CSV`
    let mut builder = phf_codegen::Map::new();
    for field in rdr.deserialize() {
        let field: Field = field.expect("field in database could not be deserialized");

        builder.entry(
            field.id,
            &format!(
                "Field {{id: 0x{:08X}, name: \"{}\", prognr: {}, datatype: Datatype::{}, path: \"{}\"}}",
                field.id, field.name, field.prognr, field.data_type, field.path
            ),
        );
    }
    // Write the generated code to $OUT_DIR/<FIELD_DB_RS>
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not defined");
    let dest_path = Path::new(&out_dir).join(FIELD_DB_RS);
    let mut file = File::create(&dest_path).expect(&format!("Failed to create {}", FIELD_DB_RS));
    writeln!(file, "use crate::field;").unwrap();
    writeln!(file, "/// static field database").unwrap();
    writeln!(
        file,
        "static FIELDS: phf::Map<u32, field::Field> = {};",
        builder.build()
    )
    .unwrap();
}
