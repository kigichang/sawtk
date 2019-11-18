use super::util;

static EMPTY_HASH: &'static str = "e3b0c44298fc1c14";

pub trait Namespace {
    fn make_address(&self, input: &str) -> String;
    fn name(&self) -> &str;
    fn prefix(&self) -> &str;
    //fn validate(&self, input: &str) -> bool;
}

// ----------------------------------------------------------------------------

struct GeneralNS {
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
        let mut ret = String::new();

        ret.push_str(&self.prefix);
        ret.push_str(&util::sha512(input)[..64]);

        ret
    }
}

// ----------------------------------------------------------------------------

struct SawtoothNS {
    name: String,
    prefix: String,
}

impl Namespace for SawtoothNS {
    fn name(&self) -> &str {
        &self.name
    }

    fn prefix(&self) -> &str {
        &self.prefix
    }

    fn make_address(&self, input: &str) -> String {
        let mut ret = String::new();
        ret.push_str(&self.prefix);
        
        let tmp: Vec<&str> = input.splitn(4, ".").collect();
        for x in tmp.iter() {
            ret.push_str(&util::sha256(x)[..16]);
        }

        if tmp.len() < 4 {
            for _ in 0..(4-tmp.len()) {
                ret.push_str(EMPTY_HASH);
            }
        }
        ret
    }
}

// ----------------------------------------------------------------------------

fn prefix(name: &str) -> String {
    let key = util::sha512(name);
    String::from(&key[..6])
}

pub fn new(name: &str) -> Box<dyn Namespace> {
    if name == "000000" {
        return Box::new(SawtoothNS {
            name: String::from("settings"),
            prefix: String::from("000000"),
        });
    }

    Box::new(GeneralNS {
        name: String::from(name),
        prefix: prefix(name),
    })
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ns() {
        let ns1 = new("intkey");
        assert_eq!("1cf126", ns1.prefix());
        assert_eq!("intkey", ns1.name());

        let ns2 = new("000000");

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