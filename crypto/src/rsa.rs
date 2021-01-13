// Copyright 2019 Parity Technologies (UK) Ltd.
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

//! RSA keys.

use asn1_der::{Asn1Der, FromDerObject, IntoDerObject, DerObject, DerTag, DerValue, Asn1DerError};
use lazy_static::lazy_static;
use super::error::*;
use ring::rand::SystemRandom;
use ring::signature::{self, RsaKeyPair, RSA_PKCS1_SHA256, RSA_PKCS1_2048_8192_SHA256};
use ring::signature::KeyPair;
use std::{fmt::{self, Write}, sync::Arc};
use zeroize::Zeroize;

/// An RSA keypair.
#[derive(Clone)]
pub struct Keypair(Arc<RsaKeyPair>);

impl Keypair {
    /// Decode an RSA keypair from a DER-encoded private key in PKCS#8 PrivateKeyInfo
    /// format (i.e. unencrypted) as defined in [RFC5208].
    ///
    /// [RFC5208]: https://tools.ietf.org/html/rfc5208#section-5
    pub fn from_pkcs8(der: &mut [u8]) -> Result<Keypair, DecodingError> {
        let kp = RsaKeyPair::from_pkcs8(&der)
            .map_err(|e| DecodingError::new("RSA PKCS#8 PrivateKeyInfo").source(e))?;
        der.zeroize();
        Ok(Keypair(Arc::new(kp)))
    }

    /// Get the public key from the keypair.
    pub fn public(&self) -> PublicKey {
        PublicKey(self.0.public_key().as_ref().to_vec())
    }

    /// Sign a message with this keypair.
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SigningError> {
        let mut signature = vec![0; self.0.public_modulus_len()];
        let rng = SystemRandom::new();
        match self.0.sign(&RSA_PKCS1_SHA256, &rng, &data, &mut signature) {
            Ok(()) => Ok(signature),
            Err(e) => Err(SigningError::new("RSA").source(e))
        }
    }
}

/// An RSA public key.
#[derive(Clone, PartialEq, Eq)]
pub struct PublicKey(Vec<u8>);

impl PublicKey {
    /// Verify an RSA signature on a message using the public key.
    pub fn verify(&self, msg: &[u8], sig: &[u8]) -> bool {
        let key = signature::UnparsedPublicKey::new(&RSA_PKCS1_2048_8192_SHA256, &self.0);
        key.verify(msg, sig).is_ok()
    }

    /// Encode the RSA public key in DER as a PKCS#1 RSAPublicKey structure,
    /// as defined in [RFC3447].
    ///
    /// [RFC3447]: https://tools.ietf.org/html/rfc3447#appendix-A.1.1
    pub fn encode_pkcs1(&self) -> Vec<u8> {
        // This is the encoding currently used in-memory, so it is trivial.
        self.0.clone()
    }

    /// Encode the RSA public key in DER as a X.509 SubjectPublicKeyInfo structure,
    /// as defined in [RFC5280].
    ///
    /// [RFC5280]: https://tools.ietf.org/html/rfc5280#section-4.1
    pub fn encode_x509(&self) -> Vec<u8> {
        let spki = Asn1SubjectPublicKeyInfo {
            algorithmIdentifier: Asn1RsaEncryption {
                algorithm: Asn1OidRsaEncryption(),
                parameters: ()
            },
            subjectPublicKey: Asn1SubjectPublicKey(self.clone())
        };
        let mut buf = vec![0u8; spki.serialized_len()];
        spki.serialize(buf.iter_mut()).map(|_| buf)
            .expect("RSA X.509 public key encoding failed.")
    }

    /// Decode an RSA public key from a DER-encoded X.509 SubjectPublicKeyInfo
    /// structure. See also `encode_x509`.
    pub fn decode_x509(pk: &[u8]) -> Result<PublicKey, DecodingError> {
        Asn1SubjectPublicKeyInfo::deserialize(pk.iter())
            .map_err(|e| DecodingError::new("RSA X.509").source(e))
            .map(|spki| spki.subjectPublicKey.0)
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = &self.0;
        let mut hex = String::with_capacity(bytes.len() * 2);

        for byte in bytes {
            write!(hex, "{:02x}", byte).expect("Can't fail on writing to string");
        }

        f.debug_struct("PublicKey")
            .field("pkcs1", &hex)
            .finish()
    }
}

//////////////////////////////////////////////////////////////////////////////
// DER encoding / decoding of public keys
//
// Primer: http://luca.ntop.org/Teaching/Appunti/asn1.html
// Playground: https://lapo.it/asn1js/

lazy_static! {
    /// The DER encoding of the object identifier (OID) 'rsaEncryption' for
    /// RSA public keys defined for X.509 in [RFC-3279] and used in
    /// SubjectPublicKeyInfo structures defined in [RFC-5280].
    ///
    /// [RFC-3279]: https://tools.ietf.org/html/rfc3279#section-2.3.1
    /// [RFC-5280]: https://tools.ietf.org/html/rfc5280#section-4.1
    static ref OID_RSA_ENCRYPTION_DER: DerObject =
        DerObject {
            tag: DerTag::x06,
            value: DerValue {
                data: vec![ 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x01, 0x01 ]
            }
        };
}

/// The ASN.1 OID for "rsaEncryption".
#[derive(Clone)]
struct Asn1OidRsaEncryption();

impl IntoDerObject for Asn1OidRsaEncryption {
    fn into_der_object(self) -> DerObject {
        OID_RSA_ENCRYPTION_DER.clone()
    }
    fn serialized_len(&self) -> usize {
        OID_RSA_ENCRYPTION_DER.serialized_len()
    }
}

impl FromDerObject for Asn1OidRsaEncryption {
    fn from_der_object(o: DerObject) -> Result<Self, Asn1DerError> {
        if o.tag != DerTag::x06 {
            return Err(Asn1DerError::InvalidTag)
        }
        if o.value != OID_RSA_ENCRYPTION_DER.value {
            return Err(Asn1DerError::InvalidEncoding)
        }
        Ok(Asn1OidRsaEncryption())
    }
}

/// The ASN.1 AlgorithmIdentifier for "rsaEncryption".
#[derive(Asn1Der)]
struct Asn1RsaEncryption {
    algorithm: Asn1OidRsaEncryption,
    parameters: ()
}

/// The ASN.1 SubjectPublicKey inside a SubjectPublicKeyInfo,
/// i.e. encoded as a DER BIT STRING.
struct Asn1SubjectPublicKey(PublicKey);

impl IntoDerObject for Asn1SubjectPublicKey {
    fn into_der_object(self) -> DerObject {
        let pk_der = (self.0).0;
        let mut bit_string = Vec::with_capacity(pk_der.len() + 1);
        // The number of bits in pk_der is trivially always a multiple of 8,
        // so there are always 0 "unused bits" signaled by the first byte.
        bit_string.push(0u8);
        bit_string.extend(pk_der);
        DerObject::new(DerTag::x03, bit_string.into())
    }
    fn serialized_len(&self) -> usize {
        DerObject::compute_serialized_len((self.0).0.len() + 1)
    }
}

impl FromDerObject for Asn1SubjectPublicKey {
    fn from_der_object(o: DerObject) -> Result<Self, Asn1DerError> {
        if o.tag != DerTag::x03 {
            return Err(Asn1DerError::InvalidTag)
        }
        let pk_der: Vec<u8> = o.value.data.into_iter().skip(1).collect();
        // We don't parse pk_der further as an ASN.1 RsaPublicKey, since
        // we only need the DER encoding for `verify`.
        Ok(Asn1SubjectPublicKey(PublicKey(pk_der)))
    }
}

/// ASN.1 SubjectPublicKeyInfo
#[derive(Asn1Der)]
#[allow(non_snake_case)]
struct Asn1SubjectPublicKeyInfo {
    algorithmIdentifier: Asn1RsaEncryption,
    subjectPublicKey: Asn1SubjectPublicKey
}