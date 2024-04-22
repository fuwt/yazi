use std::{ffi::OsStr, ops::Range};

use anyhow::Result;
use regex::bytes::{Regex, RegexBuilder};
use yazi_shared::event::Cmd;

use yazi_config::USER_DIC;

pub struct Filter {
	raw:   String,
	regex: Regex,
    chars: Vec<char>,
}


fn get_char_vec(s: &[u8]) -> Vec<char> {
    let mut ret: Vec<char> = Vec::new();
    if let Ok(_str) = std::str::from_utf8(s) {
        ret = _str.chars().collect();
    }
    return ret
}

impl PartialEq for Filter {
	fn eq(&self, other: &Self) -> bool { self.raw == other.raw }
}

impl Filter {
	pub fn new(s: &str, case: FilterCase) -> Result<Self> {
		let regex = match case {
			FilterCase::Smart => {
				let uppercase = s.chars().any(|c| c.is_uppercase());
				RegexBuilder::new(s).case_insensitive(!uppercase).build()?
			}
			FilterCase::Sensitive => Regex::new(s)?,
			FilterCase::Insensitive => RegexBuilder::new(s).case_insensitive(true).build()?,
		};
        let chars: Vec<char> = if USER_DIC.exist { s.chars().collect()} else { Vec::new() };

		Ok(Self { raw: s.to_owned(), regex, chars})
	}

	#[inline]
	pub fn matches(&self, name: &OsStr) -> bool {
        if self.regex.is_match(name.as_encoded_bytes()) {
            return true
        } else if !USER_DIC.exist {
            return false
        } else {
            let name_bytes = name.as_encoded_bytes();
            let name_chars: Vec<char> = get_char_vec(name_bytes);
            if self.chars.len() > name_chars.len() { return false }
            for i in 0..=name_chars.len() - self.chars.len() {
                let mut found = true;
                for j in 0..self.chars.len() {
                    if name_chars[i + j] == self.chars[j] { continue; }
                    if let Some(value) = USER_DIC.table.get(&name_chars[i+j]) {
                        if !value.contains(&self.chars[j]) {
                            found = false;
                            break;
                        }
                    } else {
                        found = false;
                        break;
                    }
                }
                if found { return true}

            }
            false
        }
    }

	#[inline]
	pub fn highlighted(&self, name: &OsStr) -> Option<Vec<Range<usize>>> {
		let m = self.regex.find(name.as_encoded_bytes());
        match m {
            Some(r) => return vec![r.range()].into(),
            None =>  {
                let name_bytes = name.as_encoded_bytes();
                let name_chars: Vec<char> = get_char_vec(name_bytes);
                if self.chars.len() > name_chars.len() { return None }
                for i in 0..=name_chars.len() - self.chars.len() {
                    let mut found = true;
                    for j in 0..self.chars.len() {
                        if name_chars[i + j] == self.chars[j] { continue; }
                        if let Some(value) = USER_DIC.table.get(&name_chars[i+j]) {
                            if !value.contains(&self.chars[j]) {
                                found = false;
                                break;
                            }
                        } else {
                            found = false;
                            break;
                        }
                    }
                    if found {
                        let start:usize = name_chars[0..i].into_iter().collect::<String>().len();
                        let end:usize = name_chars[0..(i + self.chars.len())].into_iter().collect::<String>().len();
                        return vec![start..end].into()
                    }
                }
                return None
            },
        }
	}
}

#[derive(Default, PartialEq, Eq)]
pub enum FilterCase {
	Smart,
	#[default]
	Sensitive,
	Insensitive,
}

impl From<&Cmd> for FilterCase {
	fn from(c: &Cmd) -> Self {
		match (c.bool("smart"), c.bool("insensitive")) {
			(true, _) => Self::Smart,
			(_, false) => Self::Sensitive,
			(_, true) => Self::Insensitive,
		}
	}
}
