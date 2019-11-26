use std::fmt;
use super::util;

static EMPTY_HASH: &'static str = "e3b0c44298fc1c14";

// -----------------------------------------------------------------------------

pub fn prefix(name: &str) -> String {
    String::from(&util::sha512(name)[..6])
    
}

pub fn address(prefix: &str, key: &str) -> String {
    let mut ret = String::new();
    ret.push_str(prefix);
    ret.push_str(&util::sha512(key)[..64]);
    ret
}

pub fn is_address(test: &str) -> bool {
    test.len() == 70
}

fn sawtooth_build_family(name: &str) -> &str {
    match name {
        "000000" | "settings" => "000000",
        "00001d" | "identity" => "00001d",
        x => x,
    }
}

pub fn sawtooth_address(prefix_or_family: &str, key: &str) -> String {
    let prefix = sawtooth_build_family(prefix_or_family);

    let mut ret = String::new();
        ret.push_str(prefix);
        
        let tmp: Vec<&str> = key.splitn(4, ".").collect();
        for x in tmp.iter() {
            ret.push_str(&util::sha256(x)[..16]);
        }

        if tmp.len() < 4 {
            //for _ in 0..(4-tmp.len()) {
            //    ret.push_str(EMPTY_HASH);
            //}
            ret.push_str(&EMPTY_HASH.repeat(4-tmp.len()));
        }
        ret
}

// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ns() {
        let ns1 = "intkey";
        assert_eq!("1cf126", prefix(ns1));
        
        assert_eq!("000000a87cb5eafdcca6a8b79606fb3afea5bdab274474a6aa82c1c0cbf0fbcaf64c0b", sawtooth_address("settings", "sawtooth.config.vote.proposals"));
        assert_eq!("0000005e50f405ace6cbdfe3b0c44298fc1c14e3b0c44298fc1c14e3b0c44298fc1c14", sawtooth_address("settings", "mykey"));
        assert_eq!("0000008923f4638a4a5030ab27b729d9cc4cb1e3b0c44298fc1c14e3b0c44298fc1c14", sawtooth_address("settings", "diviner.exchange"));

        let ns3 = prefix("df.bigbang");
        assert_eq!("534d8f", prefix("df.bigbang"));
        assert_eq!("534d8ffcfc87efb413bc331581b60745073e3fa69e96ada01beca2e4c1aebca1a1f892", address(&ns3, "Brahmā"));
        assert_eq!("534d8fb42a9d85eebee9a4b8613bb399e3f3345e73535acac74ca700ef7e9e2f848a0a", address(&ns3, "Viṣṇu"));
        assert_eq!("534d8fd1d32f37d7463d420fb6bea4c0c7cdb838a2d812942745659637f3eb502e4206", address(&ns3, "Śiva"));

        assert_eq!("cc207f", prefix("df.citizen.citizen"));
        assert_eq!("69e807", prefix("df.citizen.service"));
        assert_eq!("12515f", prefix("df.citizen.transfer"));
        assert_eq!("c6cd3c", prefix("df.citizen.samsara"));
    }
}

// -----------------------------------------------------------------------------

pub trait Namespace: fmt::Display {
    fn make_address(&self, input: &str) -> String;
    fn name(&self) -> &str;
    fn prefix(&self) -> &str;
}

// -----------------------------------------------------------------------------

pub struct GeneralNS {
    name: String,
    prefix: String,
}

impl Namespace for GeneralNS {
    
    fn name(&self) -> &str {
        &self.name
    }

    fn prefix(&self) -> &str {
        &self.prefix
    }

    fn make_address(&self, input: &str) -> String {
        address(&self.prefix, input)
    }
}

impl fmt::Display for GeneralNS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "name: {}, prefix: {}", self.name, self.prefix)
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct SawtoothNS {
    name: &'static str,
    prefix: &'static str,
}

impl Namespace for SawtoothNS {
    fn name(&self) -> &str {
        self.name
    }

    fn prefix(&self) -> &str {
        self.prefix
    }

    fn make_address(&self, input: &str) -> String {
        sawtooth_address(self.prefix, input)
    }
}

impl fmt::Display for SawtoothNS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "sawtooth: {}, prefix: {}", self.name, self.prefix)
    }
}

// -----------------------------------------------------------------------------

pub fn new(name: &str) -> Box<dyn Namespace> {
    Box::new(GeneralNS {
        name: name.to_string(),
        prefix: prefix(name),
    })
}

pub fn sawtooth(family: &str) -> Box<dyn Namespace> {
    match family {
        "000000" | "settings" => {
            Box::new(SawtoothNS {
                name: "settings",
                prefix: "000000",
            })
        },
        "00001d" | "identity" => {
            Box::new(SawtoothNS {
                name: "identity",
                prefix: "00001d",
            })
        },
        _ => new(family)
    }
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod test_namespace {
    use super::*;

    #[test]
    fn test_ns() {
        let ns1 = new("intkey");
        assert_eq!("1cf126", ns1.prefix());
        assert_eq!("intkey", ns1.name());

        let ns2 = sawtooth("000000");

        assert_eq!("000000", ns2.prefix());
        assert_eq!("settings", ns2.name());
        
        assert_eq!("000000a87cb5eafdcca6a8b79606fb3afea5bdab274474a6aa82c1c0cbf0fbcaf64c0b", ns2.make_address("sawtooth.config.vote.proposals"));
        assert_eq!("0000005e50f405ace6cbdfe3b0c44298fc1c14e3b0c44298fc1c14e3b0c44298fc1c14", ns2.make_address("mykey"));
        assert_eq!("0000008923f4638a4a5030ab27b729d9cc4cb1e3b0c44298fc1c14e3b0c44298fc1c14", ns2.make_address("diviner.exchange"));


        let ns3 = new("df.bigbang");

        assert_eq!("534d8f", ns3.prefix());
        assert_eq!("df.bigbang", ns3.name());
        assert_eq!("534d8ffcfc87efb413bc331581b60745073e3fa69e96ada01beca2e4c1aebca1a1f892", ns3.make_address("Brahmā"));
        assert_eq!("534d8fb42a9d85eebee9a4b8613bb399e3f3345e73535acac74ca700ef7e9e2f848a0a", ns3.make_address("Viṣṇu"));
        assert_eq!("534d8fd1d32f37d7463d420fb6bea4c0c7cdb838a2d812942745659637f3eb502e4206", ns3.make_address("Śiva"));
    }
}
