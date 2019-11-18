use sawtooth_sdk::signing;
use crate::{Result, Error};

static ALG_NAME: &'static str = "secp256k1";

// ----------------------------------------------------------------------------

pub struct Signer {
    context: Box<dyn signing::Context>,
    key: Box<dyn signing::PrivateKey>,
}


impl Signer {
   
    pub fn from_hex(key: &str) -> Result<Self> {
        let context = create_context()?;

        signing::secp256k1::Secp256k1PrivateKey::from_hex(key)
            .map(|key| {
                Signer { 
                    context: context,
                    key: Box::new(key),
                }
            })
            .map_err(|e| Error::Signing(e))
    }

    pub fn get_public_key(&self ) -> Result<String> {
        self.context.get_public_key(self.key.as_ref()).map(|key| key.as_hex()).map_err(|e| Error::Signing(e))
    }

    pub fn sign(&self, message: &[u8]) -> Result<String> {
        self.context.sign(message, self.key.as_ref()).map_err(|e| Error::Signing(e))
    }

    pub fn new() -> Result<Self> {
        create_context().and_then(|context| {
            new_random_private_key().map(|k| {
                Signer {
                    context: context,
                    key: k,
                }
            })
        })
    }
}


// ----------------------------------------------------------------------------

pub fn create_context() -> Result<Box<dyn signing::Context>> {
    signing::create_context(ALG_NAME).map_err(|e| Error::Signing(e))
}

pub fn new_random_private_key() -> Result<Box<dyn signing::PrivateKey>> {
    let context = create_context()?;
    context.new_random_private_key().map_err(|e| Error::Signing(e))
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    static KEY1_PRIV_HEX: &'static str = "2f1e7b7a130d7ba9da0068b3bb0ba1d79e7e77110302c9f746c3c2a63fe40088";
    static KEY1_PUB_HEX: &'static str = "026a2c795a9776f75464aa3bda3534c3154a6e91b357b1181d3f515110f84b67c5";

    static KEY2_PRIV_HEX: &'static str = "51b845c2cdde22fe646148f0b51eaf5feec8c82ee921d5e0cbe7619f3bb9c62d";
    static KEY2_PUB_HEX: &'static str = "039c20a66b4ec7995391dbec1d8bb0e2c6e6fd63cd259ed5b877cb4ea98858cf6d";

    static MSG1: &'static str = "test";
    static MSG1_KEY1_SIG: &'static str = "5195115d9be2547b720ee74c23dd841842875db6eae1f5da8605b050a49e702b4aa83be72ab7e3cb20f17c657011b49f4c8632be2745ba4de79e6aa05da57b35";

    static MSG2: &'static str = "test2";
    static MSG2_KEY2_SIG: &'static str = "d589c7b1fa5f8a4c5a389de80ae9582c2f7f2a5e21bab5450b670214e5b1c1235e9eb8102fd0ca690a8b42e2c406a682bd57f6daf6e142e5fa4b2c26ef40a490";


    #[test]
    fn test_signer() {
        
        let signer1 = Signer::from_hex(KEY1_PRIV_HEX).unwrap();
        assert_eq!(signer1.get_public_key().unwrap(), KEY1_PUB_HEX);

        let signer2 = Signer::from_hex(KEY2_PRIV_HEX).unwrap();
        assert_eq!(signer2.get_public_key().unwrap(), KEY2_PUB_HEX);


        let sign1 = signer1.sign(&String::from(MSG1).into_bytes()).unwrap();
        assert_eq!(sign1, MSG1_KEY1_SIG);

        let sign2 = signer2.sign(&String::from(MSG2).into_bytes()).unwrap();
        assert_eq!(sign2, MSG2_KEY2_SIG);
    }

    
}
