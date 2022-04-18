//! Parsing for the `Records.json` file that Google sends you when
//! you want your Google Maps location history.
#![warn(
	unreachable_pub,
	// missing_debug_implementations,
	// missing_docs,
	clippy::pedantic
)]

use serde::{
    de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};

use crate::protocol::LocationEntry;

pub mod protocol;

struct Records<F> {
    locations: Locations<F>,
}

impl<'de, F> DeserializeSeed<'de> for Records<F>
where
    F: FnMut(LocationEntry),
{
    type Value = Self;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "camelCase")]
        enum Field {
            Locations,
        }

        struct RecordsVisitor<F>(Locations<F>);

        impl<'de, F> Visitor<'de> for RecordsVisitor<F>
        where
            F: FnMut(LocationEntry),
        {
            type Value = Records<F>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a map")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut locations = self.0;
                let mut visited_locations = false;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Locations => {
                            if visited_locations {
                                return Err(de::Error::duplicate_field("locations"));
                            }
                            locations = map.next_value_seed(locations)?;
                            visited_locations = true;
                        }
                    }
                }

                Ok(Records { locations })
            }
        }

        deserializer.deserialize_map(RecordsVisitor(self.locations))
    }
}

struct Locations<F>(F);

impl<'de, F> DeserializeSeed<'de> for Locations<F>
where
    F: FnMut(LocationEntry),
{
    type Value = Self;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LocationsVisitor<F: FnMut(LocationEntry)>(F);

        impl<'de, F> Visitor<'de> for LocationsVisitor<F>
        where
            F: FnMut(LocationEntry),
        {
            type Value = Locations<F>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of location entries")
            }

            fn visit_seq<S>(mut self, mut seq: S) -> Result<Self::Value, S::Error>
            where
                S: SeqAccess<'de>,
            {
                while let Some(entry) = seq.next_element::<LocationEntry>()? {
                    (self.0)(entry);
                }

                Ok(Locations(self.0))
            }
        }

        deserializer.deserialize_seq(LocationsVisitor(self.0))
    }
}

/// Lazily read entries using a custom serializer. The
/// source should have a structure corresponding to a
/// `Records.json` file.
///
/// Unless using a custom format, you are probably looking
/// for [`read_json_entries`].
///
/// # Errors
///
/// If the deserializer fails, an error is returned.
pub fn read_entries<'de, D, F>(deserializer: D, f: F) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
    F: FnMut(LocationEntry),
{
    let records = Records {
        locations: Locations(f),
    };
    records.deserialize(deserializer)?;

    Ok(())
}

/// Lazily read entries from a `Records.json` file reader.
///
/// # Errors
///
/// If the input data is invalid or Serde decides to fail,
/// an `Err` is returned.
pub fn read_json_entries<R, F>(reader: R, f: F) -> Result<(), serde_json::Error>
where
    R: std::io::Read,
    F: FnMut(LocationEntry),
{
    let mut deserializer = serde_json::Deserializer::from_reader(reader);
    read_entries(&mut deserializer, f)
}
