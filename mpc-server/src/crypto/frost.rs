use frost_ed25519 as frost;
use frost::keys::dkg;
use rand::rngs::OsRng;
use std::collections::BTreeMap;
use crate::error::MpcError;

pub const MAX_SIGNERS: u16 = 3;
pub const MIN_SIGNERS: u16 = 2;

pub fn dkg_part1(
    node_id: u16,
) -> Result<(dkg::round1::SecretPackage, dkg::round1::Package), MpcError> {
    let mut rng = OsRng;
    let identifier: frost::Identifier = node_id
        .try_into()
        .map_err(|_| MpcError::Crypto("Invalid node identifier".to_string()))?;

    let (secret_package, package) = dkg::part1(
        identifier,
        MAX_SIGNERS,
        MIN_SIGNERS,
        &mut rng,
    )?;

    Ok((secret_package, package))
}

pub fn dkg_part2(
    round1_secret: dkg::round1::SecretPackage,
    received_round1_packages: &BTreeMap<frost::Identifier, dkg::round1::Package>,
) -> Result<(dkg::round2::SecretPackage, BTreeMap<frost::Identifier, dkg::round2::Package>), MpcError> {
    let (round2_secret, round2_packages) = dkg::part2(
        round1_secret,
        received_round1_packages,
    )?;

    Ok((round2_secret, round2_packages))
}

pub fn dkg_part3(
    round2_secret: &dkg::round2::SecretPackage,
    received_round1_packages: &BTreeMap<frost::Identifier, dkg::round1::Package>,
    received_round2_packages: &BTreeMap<frost::Identifier, dkg::round2::Package>,
) -> Result<(frost::keys::KeyPackage, frost::keys::PublicKeyPackage), MpcError> {
    let (key_package, pubkey_package) = dkg::part3(
        round2_secret,
        received_round1_packages,
        received_round2_packages,
    )?;

    Ok((key_package, pubkey_package))
}

pub fn sign_round1(
    key_package: &frost::keys::KeyPackage,
) -> (frost::round1::SigningNonces, frost::round1::SigningCommitments) {
    let mut rng = OsRng;
    frost::round1::commit(key_package.signing_share(), &mut rng)
}

pub fn sign_round2(
    signing_package: &frost::SigningPackage,
    nonces: &frost::round1::SigningNonces,
    key_package: &frost::keys::KeyPackage,
) -> Result<frost::round2::SignatureShare, MpcError> {
    let share = frost::round2::sign(signing_package, nonces, key_package)?;
    Ok(share)
}
