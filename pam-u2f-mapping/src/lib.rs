// Copyright © Amanda Graven 2022
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

//! Dead simple parser and formatter for mapping files generated by
//! `pamu2fcfg(1)`.

#![warn(missing_docs)]

use std::str::FromStr;

/// Represents the contents of a mapping file.
#[derive(Clone, Debug)]
pub struct MappingFile {
	/// The list of mapping entries in the file
	pub mappings: Vec<Mapping>,
}

/// The list of keys associated with a given username. Corresponds to one line in the mapping file
#[derive(Clone, Debug)]
pub struct Mapping {
	/// The username the mapping applies to
	pub user: String,
	/// The list of keys associated with the user
	pub keys: Vec<Key>,
}

/// A key entry in a mapping file. Corresponds to one colon (:) separated entry in a mapping line.
#[derive(Clone, Debug)]
pub struct Key {
	/// The key handle
	pub handle: String,
	/// The public key data
	pub public: String,
	/// The key algorithm
	pub kind: String,
	/// Flags for the key
	pub flags: Vec<String>,
}

impl FromStr for MappingFile {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mappings = s
			.lines()
			.map(Mapping::from_str)
			.collect::<Result<Vec<Mapping>, Error>>()?;
		Ok(MappingFile { mappings })
	}
}

impl std::str::FromStr for Mapping {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut fields = s.split(':');
		let user = fields.next().ok_or(Error::UserMissing)?;
		let mut keys = Vec::new();
		for field in fields {
			let mut subfields = field.split(',');
			// split will always yield at least one item
			let public = subfields.next().unwrap().to_owned();
			let handle = subfields.next().ok_or(Error::HandleMissing)?.to_owned();
			let kind = subfields.next().ok_or(Error::KindMissing)?.to_owned();
			let flags = subfields.next().ok_or(Error::FlagsMissing)?.to_owned();
			let mut flags = flags.split('+');
			if flags.next() != Some("") {
				return Err(Error::BadFlags);
			}
			let flags = flags.map(|s| s.to_owned()).collect::<Vec<_>>();
			keys.push(Key {
				public,
				handle,
				kind,
				flags,
			})
		}
		Ok(Mapping {
			user: user.to_owned(),
			keys,
		})
	}
}

impl std::fmt::Display for Mapping {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.user)?;
		for key in &self.keys {
			write!(f, ":{},{},{},", &key.handle, &key.public, &key.kind)?;
			for flag in &key.flags {
				write!(f, "+{flag}")?;
			}
		}
		Ok(())
	}
}

/// The key contained invalid data and failed to parse
#[derive(Debug, Clone, Copy)]
pub enum Error {
	/// User field was missing from a mapping
	UserMissing,
	/// Key handle was missing
	HandleMissing,
	/// Key kind was missing
	KindMissing,
	/// Key flags were missing
	FlagsMissing,
	/// Key flags were malformed
	BadFlags,
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use Error::*;
		let s = match *self {
			UserMissing => "Entry has no username",
			HandleMissing => "Missing second half of key data",
			KindMissing => "Entry has no key type",
			FlagsMissing => "Entry has no flags",
			BadFlags => "Entry has ill-formed flags",
		};
		f.write_str(s)
	}
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
	use super::Mapping;
	type BoxError = Box<dyn std::error::Error>;
	const TEST_MAPPING: &str = "alice:\
		owBYtYMabYlexEG10ildyDLNqwkpeIZyc4YwqP6yUnqlQ3DCxNMjPXoGcQOPiNXu2kFuGKs\
		LsN6am/UCjgUpwvr9G54GyY85i0zt/vHRsU+OayYoalSjsVjBvyRqFai3fZUdGEHVLdpw9Y\
		Z3MZeJiSWWEumF59CBdFWNLtq0Xi5M1katPXKIqOUSHLePlq1UfaGkh7R5y+Cv8jXtrhtak\
		ROcMXjrAfo+5Wq0hNe0JiQwxFPufHUJ8IMBTFw4Qv3TnPGcVFTXZgJQU1FguzVlQ6pU7FS6\
		37Dhdg==,\
		IiFyv2O8qSG517c2ghvHEbMb6xs5ToPaoOXdgGkkorH2ta/iYWtOhMB7wxaiS3BhOHSxcJU\
		JJkMLmfUWl8Uivw==,\
		es256,+presence";
	#[test]
	fn parse() -> Result<(), BoxError> {
		let mapping: Mapping = TEST_MAPPING.parse()?;
		assert_eq!(mapping.user, "alice", "user mismatch");
		let key = &mapping.keys[0];
		assert_eq!(
			key.handle,
			"owBYtYMabYlexEG10ildyDLNqwkpeIZyc4YwqP6yUnqlQ3DCxNMjPXoGcQOPiNXu2k\
			FuGKsLsN6am/UCjgUpwvr9G54GyY85i0zt/vHRsU+OayYoalSjsVjBvyRqFai3fZUdG\
			EHVLdpw9YZ3MZeJiSWWEumF59CBdFWNLtq0Xi5M1katPXKIqOUSHLePlq1UfaGkh7R5\
			y+Cv8jXtrhtakROcMXjrAfo+5Wq0hNe0JiQwxFPufHUJ8IMBTFw4Qv3TnPGcVFTXZgJ\
			QU1FguzVlQ6pU7FS637Dhdg==",
			"key handle mismatch"
		);
		assert_eq!(
			key.public,
			"IiFyv2O8qSG517c2ghvHEbMb6xs5ToPaoOXdgGkkorH2ta/iYWtOhMB7wxaiS3BhOHSxcJUJJkMLmfUWl8Uivw==",
			"public key mismatch"
		);
		assert_eq!(key.kind, "es256");
		assert_eq!(key.flags, &["presence"]);
		Ok(())
	}
	/// Asserts that a file gets parsed and formatted to the same data
	#[test]
	fn non_destructive() -> Result<(), BoxError> {
		assert_eq!(TEST_MAPPING.parse::<Mapping>()?.to_string(), TEST_MAPPING);
		Ok(())
	}
}
