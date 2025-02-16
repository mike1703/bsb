use std::fmt::Display;

use serde::Serialize;

use crate::Datatype;
// include the bsb field definitions in a static map in `FIELDS`
include!(concat!(env!("OUT_DIR"), "/field_db.rs"));

/// the `name` and `datatype` of this `Field`
#[derive(Debug, PartialEq, Serialize)]
pub struct Field {
    id: u32,
    name: &'static str,
    prognr: usize,
    datatype: Datatype,
    path: &'static str,
}

impl Field {
    /// Try to get a `Field` definition from an field `id`
    pub fn by_id(id: u32) -> Option<&'static Field> {
        FIELDS.get(&id)
    }

    /// Try to get a `Field` definition from a field `name`
    pub fn by_name(name: &str) -> Option<&'static Field> {
        FIELDS.values().find(|field| field.name == name)
    }

    /// Access `Field.id`
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Access `Field.datatype`
    pub fn datatype(&self) -> Datatype {
        self.datatype
    }

    /// Access `Field.prognr`
    pub fn prognr(&self) -> usize {
        self.prognr
    }

    /// Access `Field.name`
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Access `Field.path`
    pub fn path(&self) -> &'static str {
        self.path
    }

    /// Iterator over the known fields
    pub fn iter<'a>() -> phf::map::Entries<'a, u32, Field> {
        FIELDS.entries()
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use crate::Datatype;

    use super::Field;

    const TESTFIELD: Field = Field {
        id: 0x313d052f,
        name: "warmwater_temperature",
        prognr: 8701,
        datatype: Datatype::Float(64),
        path: "temperature/warmwater",
    };

    #[test]
    fn test_field_db_by_id() {
        let testcase = Field::by_id(TESTFIELD.id).unwrap();
        let want = TESTFIELD;
        assert_eq!(testcase, &want)
    }

    #[test]
    fn test_field_db_by_name() {
        let testcase = Field::by_name(&TESTFIELD.name).unwrap();
        let want = TESTFIELD;
        assert_eq!(testcase, &want)
    }

    #[test]
    fn test_field_to_string() {
        let testcase = TESTFIELD.to_string();
        let want = TESTFIELD.name;
        assert_eq!(&testcase, want);
    }

    #[test]
    fn test_field_id() {
        let testcase = TESTFIELD.id();
        let want = 0x313d052f;
        assert_eq!(testcase, want);
    }

    #[test]
    fn test_field_datatype() {
        let testcase = TESTFIELD.datatype();
        let want = Datatype::Float(64);
        assert_eq!(testcase, want);
    }

    #[test]
    fn test_field_prognr() {
        let testcase = TESTFIELD.prognr();
        let want = 8701;
        assert_eq!(testcase, want);
    }

    #[test]
    fn test_field_name() {
        let testcase = TESTFIELD.name();
        let want = "warmwater_temperature";
        assert_eq!(testcase, want);
    }

    #[test]
    fn test_field_path() {
        let testcase = TESTFIELD.path();
        let want = "temperature/warmwater";
        assert_eq!(testcase, want);
    }

    #[test]
    fn test_field_iter() {
        let testcase = Field::iter().next();
        assert!(testcase.is_some())
    }
}
