// SPDX-FileCopyrightText: 2022 Profian Inc. <opensource@profian.com>
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg(feature = "rcrypto-rsa")]

use rsa::{
    traits::{PrivateKeyParts, PublicKeyParts},
    BigUint, RsaPrivateKey, RsaPublicKey,
};

use jose_jwa::{Algorithm, Algorithm::Signing, Signing::*};

use super::super::KeyInfo;
use super::Error;
use crate::{Rsa, RsaPrivate};

#[cfg_attr(docsrs, doc(cfg(feature = "rcrypto-rsa")))]
impl KeyInfo for RsaPublicKey {
    fn strength(&self) -> usize {
        self.size() / 16
    }

    fn is_supported(&self, algo: &Algorithm) -> bool {
        // RFC 7518 Section 3.3
        //
        // I would actually prefer stronger requirements here based on the
        // algorithm below. However, the RFCs actually specify examples that
        // this would break. Note that we generate stronger keys by default.
        if self.strength() < 16 {
            return false;
        }

        #[allow(clippy::match_like_matches_macro)]
        match algo {
            Signing(Rs256) => true,
            Signing(Rs384) => true,
            Signing(Rs512) => true,
            Signing(Ps256) => true,
            Signing(Ps384) => true,
            Signing(Ps512) => true,
            _ => false,
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "rcrypto-rsa")))]
impl KeyInfo for RsaPrivateKey {
    fn strength(&self) -> usize {
        self.size() / 16
    }

    fn is_supported(&self, algo: &Algorithm) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match (algo, self.strength()) {
            (Signing(Rs256), 16..) => true,
            (Signing(Rs384), 24..) => true,
            (Signing(Rs512), 32..) => true,
            (Signing(Ps256), 16..) => true,
            (Signing(Ps384), 24..) => true,
            (Signing(Ps512), 32..) => true,
            _ => false,
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "rcrypto-rsa")))]
impl From<&RsaPublicKey> for Rsa {
    fn from(pk: &RsaPublicKey) -> Self {
        Self {
            n: pk.n().to_bytes_be().into(),
            e: pk.e().to_bytes_be().into(),
            prv: None,
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "rcrypto-rsa")))]
impl From<RsaPublicKey> for Rsa {
    fn from(sk: RsaPublicKey) -> Self {
        (&sk).into()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "rcrypto-rsa")))]
impl TryFrom<&Rsa> for RsaPublicKey {
    type Error = Error;

    fn try_from(value: &Rsa) -> Result<Self, Self::Error> {
        let n = BigUint::from_bytes_be(&value.n);
        let e = BigUint::from_bytes_be(&value.e);
        RsaPublicKey::new(n, e).map_err(|_| Error::Invalid)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "rcrypto-rsa")))]
impl TryFrom<Rsa> for RsaPublicKey {
    type Error = Error;

    fn try_from(value: Rsa) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

// TODO: patch rsa crate to export the optional values
#[cfg_attr(docsrs, doc(cfg(feature = "rcrypto-rsa")))]
impl From<&RsaPrivateKey> for Rsa {
    fn from(pk: &RsaPrivateKey) -> Self {
        Self {
            n: pk.n().to_bytes_be().into(),
            e: pk.e().to_bytes_be().into(),
            prv: Some(RsaPrivate {
                d: pk.d().to_bytes_be().into(),
                opt: None,
            }),
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "rcrypto-rsa")))]
impl From<RsaPrivateKey> for Rsa {
    fn from(sk: RsaPrivateKey) -> Self {
        (&sk).into()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "rcrypto-rsa")))]
impl TryFrom<&Rsa> for RsaPrivateKey {
    type Error = Error;

    fn try_from(value: &Rsa) -> Result<Self, Self::Error> {
        if let Some(prv) = value.prv.as_ref() {
            if let Some(opt) = prv.opt.as_ref() {
                let n = BigUint::from_bytes_be(&value.n);
                let e = BigUint::from_bytes_be(&value.e);
                let d = BigUint::from_bytes_be(&prv.d);
                let p = BigUint::from_bytes_be(&opt.p);
                let q = BigUint::from_bytes_be(&opt.q);

                let mut primes = alloc::vec![p, q];
                primes.extend(opt.oth.iter().map(|x| BigUint::from_bytes_be(&x.r)));

                return Self::from_components(n, e, d, primes).map_err(|_| Error::Invalid);
            }

            return Err(Error::Unsupported);
        }

        Err(Error::NotPrivate)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "rcrypto-rsa")))]
impl TryFrom<Rsa> for RsaPrivateKey {
    type Error = Error;

    fn try_from(value: Rsa) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}
