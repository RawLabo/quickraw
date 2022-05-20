use super::*;

macro_rules! gen_collector_impls_for_num {
    ($t:tt) => {
        pub fn $t(&self, name: &str) -> Result<$t, RawInfoError> {
            match self.content.get(name) {
                Some(v) => Ok(v.$t()?),
                None => {
                    Err(RawInfoError::FieldNotFound(name.to_owned()))
                }
            }
        }
    };
}

impl ParsedRawInfo {
    gen_collector_impls_for_num!(u16);
    gen_collector_impls_for_num!(u32);
    gen_collector_impls_for_num!(i32);
    gen_collector_impls_for_num!(f64);
    gen_collector_impls_for_num!(usize);

    pub fn str<'a>(&'a self, name: &str) -> Result<&'a str, RawInfoError> {
        match self.content.get(name) {
            Some(v) => Ok(v.str()?),
            None => {
                Err(RawInfoError::FieldNotFound(name.to_owned()))
            }
        }
    }

    pub fn u8a4(&self, name: &str) -> Result<[u8; 4], RawInfoError> {
        match self.content.get(name) {
            Some(v) => Ok(v.u8a4(self.is_le)?),
            None => {
                Err(RawInfoError::FieldNotFound(name.to_owned()))
            }
        }
    }

    pub fn stringify_all(&self) -> Result<String, ValueError> {
        let mut result = format!(
            "{:>22}:  {}-endian\n",
            "endianness",
            if self.is_le { "little" } else { "big" }
        );
        let mut names = self.content.iter().map(|x| x).collect::<Vec<_>>();
        names.sort_by(|(a, _), (b, _)| a.cmp(b));

        for (name, value) in names.iter() {
            let value_str = match value {
                Value::U16(x) => format!("{} / {:#x?}", x, x),
                Value::U32(x) => format!("{} / {:#x?} / {:?}", x, x, value.u8a4(self.is_le)?),
                Value::R64(x) => x.to_string(),
                Value::Str(_) => value.str()?.to_owned(),
            };
            result.push_str(format!("{:>22}:  {}\n", name, value_str).as_str());
        }

        Ok(result)
    }
}
