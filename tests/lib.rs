use radix_engine_interface::prelude::*;
use scrypto::this_package;
use scrypto_test::prelude::*;

use fomo_staking::test_bindings::*;

#[test]
fn test_fomo_staking() -> Result<(), RuntimeError> {
    let mut env = TestEnvironment::new();
    env.disable_auth_module();
    let package_address = Package::compile_and_publish(this_package!(), &mut env)?;

    // Create FOMO owner badge
    let badge_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(0)
        .mint_initial_supply(1, &mut env)?;
    let badge_address = badge_bucket.resource_address(&mut env)?;

    // Create FOMO coin
    let fomo_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(17000000, &mut env)?;
    let fomo_address = fomo_bucket.resource_address(&mut env)?;

    // Instantiate a FomoStaking component
    let mut fomo_staking = FomoStaking::new(
        badge_address,
        fomo_address,
        0,
        package_address,
        &mut env
    )?;

    // Stake 100 FOMO
    let staked_fomo_bucket1 = fomo_staking.add_stake(
        fomo_bucket.take(dec!(100), &mut env)?,
        &mut env
    )?;

    // Stake 50 FOMO
    let staked_fomo_bucket2 = fomo_staking.add_stake(
        fomo_bucket.take(dec!(50), &mut env)?,
        &mut env
    )?;

    // Deposit 1000 FOMO as future staking rewards and distribute 30 of them
    fomo_staking.deposit_rewards(
        fomo_bucket.take(dec!(1000), &mut env)?,
        &mut env
    )?;
    fomo_staking.airdrop_deposited_amount(
        dec!(30),
        &mut env
    )?;

    // Stake 100 FOMO
    let staked_fomo_bucket3 = fomo_staking.add_stake(
        fomo_bucket.take(dec!(100), &mut env)?,
        &mut env
    )?;

    // Create TEST coin
    let test_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(17000000, &mut env)?;

    // Make two airdrops of total 280 TEST coins
    fomo_staking.airdrop(
        test_bucket.take(dec!(130), &mut env)?,
        &mut env
    )?;
    fomo_staking.airdrop(
        test_bucket.take(dec!(150), &mut env)?,
        &mut env
    )?;

    // Stake 50 FOMO
    let staked_fomo_bucket4 = fomo_staking.add_stake(
        fomo_bucket.take(dec!(50), &mut env)?,
        &mut env
    )?;

    // Unstake staked FOMO and check the amounts (a small rounding error is acceptable)
    let vec_of_buckets1 = fomo_staking.remove_stake(
        staked_fomo_bucket1,
        &mut env
    )?;
    let fomo_received1 = vec_of_buckets1[0].amount(&mut env)?;
    assert!(
        fomo_received1 > dec!("119.99") && fomo_received1 < dec!("120.01"),
        "Wrong FOMO received 1: {}",
        fomo_received1
    );
    let test_received1 = vec_of_buckets1[1].amount(&mut env)?;
    assert!(
        test_received1 > dec!("119.99") && test_received1 < dec!("120.01"),
        "Wrong TEST received 1: {}",
        test_received1
    );

    // Unstake staked FOMO and check the amounts (a small rounding error is acceptable)
    let vec_of_buckets2 = fomo_staking.remove_stake(
        staked_fomo_bucket2,
        &mut env
    )?;
    let fomo_received2 = vec_of_buckets2[0].amount(&mut env)?;
    assert!(
        fomo_received2 > dec!("59.99") && fomo_received2 < dec!("60.01"),
        "Wrong FOMO received 2: {}",
        fomo_received2
    );
    let test_received2 = vec_of_buckets2[1].amount(&mut env)?;
    assert!(
        test_received2 > dec!("59.99") && test_received2 < dec!("60.01"),
        "Wrong TEST received 2: {}",
        test_received2
    );

    // Unstake staked FOMO and check the amounts (a small rounding error is acceptable)
    let vec_of_buckets3 = fomo_staking.remove_stake(
        staked_fomo_bucket3,
        &mut env
    )?;
    let fomo_received3 = vec_of_buckets3[0].amount(&mut env)?;
    assert!(
        fomo_received3 > dec!("99.99") && fomo_received3 < dec!("100.01"),
        "Wrong FOMO received 3: {}",
        fomo_received3
    );
    let test_received3 = vec_of_buckets3[1].amount(&mut env)?;
    assert!(
        test_received3 > dec!("99.99") && test_received3 < dec!("100.01"),
        "Wrong TEST received 3: {}",
        test_received3
    );

    // Unstake staked FOMO and check the amounts (a small rounding error is acceptable)
    let vec_of_buckets4 = fomo_staking.remove_stake(
        staked_fomo_bucket4,
        &mut env
    )?;
    let fomo_received4 = vec_of_buckets4[0].amount(&mut env)?;
    assert!(
        fomo_received4 > dec!("49.99") && fomo_received4 < dec!("50.01"),
        "Wrong FOMO received 4: {}",
        fomo_received4
    );
    assert!(
        vec_of_buckets4.len() == 1,
        "Received an undue bucket"
    );

    Ok(())
}
