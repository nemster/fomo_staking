use scrypto::prelude::*;

#[derive(ScryptoSbor, ScryptoEvent)]
struct AirdropEvent {
    coin: ResourceAddress,
    amount: Decimal,
}

#[derive(Debug, ScryptoSbor, NonFungibleData)]
struct StakedFomoData {
    stake_date: Instant,
    minimum_unstake_date: Instant,
    amount_staked: Decimal,
    stake_share: PreciseDecimal,
    last_airdrop_id: u64,
}

#[derive(Debug, ScryptoSbor)]
struct Airdrop {
    vault: Vault,
    amount_per_share: PreciseDecimal,
}

static STAKED_FOMO_NAME: &str = "Staked FOMO";
static STAKED_FOMO_ICON: &str = "https://pbs.twimg.com/media/GEfnpcUbIAAoEY8?format=jpg&name=small";
static MAX_AIRDROPS_PER_USER: u64 = 100;

#[blueprint]
#[events(AirdropEvent)]
mod fomo_staking {

    enable_method_auth! {
        methods {
            add_stake => PUBLIC;
            remove_stake => PUBLIC;
            airdrop => PUBLIC;
            deposit_rewards => restrict_to: [OWNER];
            airdrop_deposited_amount => restrict_to: [OWNER];
        }
    }

    struct FomoStaking {
        fomo_vault: Vault,
        airdrops: KeyValueStore<u64, Airdrop>,
        minimum_stake_period: i64,
        staked_fomo_resource_manager: ResourceManager,
        next_staked_fomo_id: u64,
        total_stake_share: PreciseDecimal,
        future_rewards: Vault,
        last_airdrop_id: u64,
    }

    impl FomoStaking {

        pub fn new(
            owner_badge_address: ResourceAddress,
            fomo_address: ResourceAddress,
            minimum_stake_period: i64,
        ) -> Global<FomoStaking> {

            assert!(
                minimum_stake_period >= 0,
                "Invalid minimum_stake_period",
            );

            // Reserve a ComponentAddress for setting rules on resources
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(FomoStaking::blueprint_id());

            // Create a ResourceManager for minting transient NFTs used by flash loans
            let staked_fomo_resource_manager = ResourceBuilder::new_integer_non_fungible::<StakedFomoData>(
                OwnerRole::Updatable(rule!(require(owner_badge_address)))
            )
            .metadata(metadata!(
                roles {
                    metadata_setter => rule!(require(owner_badge_address));
                    metadata_setter_updater => rule!(require(owner_badge_address));
                    metadata_locker => rule!(require(owner_badge_address));
                    metadata_locker_updater => rule!(require(owner_badge_address));
                },
                init {
                    "name" => STAKED_FOMO_NAME, updatable;
                    "icon_url" => MetadataValue::Url(UncheckedUrl::of(STAKED_FOMO_ICON.to_string())), updatable;
                }
            ))
            .mint_roles(mint_roles!(
                minter => rule!(require(global_caller(component_address)));
                minter_updater => rule!(deny_all);
            ))
            .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                non_fungible_data_updater => rule!(deny_all);
                non_fungible_data_updater_updater => rule!(require(owner_badge_address));
            ))
            .burn_roles(burn_roles!(
                burner => rule!(require(global_caller(component_address)));
                burner_updater => rule!(deny_all);
            ))
            .create_with_no_initial_supply();

            Self {
                fomo_vault: Vault::new(fomo_address),
                airdrops: KeyValueStore::new(),
                minimum_stake_period: minimum_stake_period,
                staked_fomo_resource_manager: staked_fomo_resource_manager,
                next_staked_fomo_id: 1,
                total_stake_share: PreciseDecimal::ZERO,
                future_rewards: Vault::new(fomo_address),
                last_airdrop_id: 0,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .with_address(address_reservation)
            .globalize()
        }

        pub fn add_stake(&mut self, fomo: Bucket) -> Bucket {
            assert!(
                fomo.resource_address() == self.fomo_vault.resource_address(),
                "Wrong coin bro",
            );

            let amount = fomo.amount();
            assert!(
                amount > Decimal::ZERO,
                "No coin bro",
            );

            let vault_amount = self.fomo_vault.amount();
            self.fomo_vault.put(fomo);

            let mut stake_share = PreciseDecimal::ONE;
            if vault_amount > Decimal::ZERO {
                stake_share = (self.total_stake_share * amount) / vault_amount;
            }
            self.total_stake_share += stake_share;

            let now = Clock::current_time_rounded_to_seconds();

            let staked_fomo = self.staked_fomo_resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(self.next_staked_fomo_id.into()),
                StakedFomoData {
                    stake_date: now,
                    minimum_unstake_date: Instant {
                        seconds_since_unix_epoch: now.seconds_since_unix_epoch + self.minimum_stake_period,
                    },
                    amount_staked: amount,
                    stake_share: stake_share,
                    last_airdrop_id: self.last_airdrop_id,
                }
            );
            self.next_staked_fomo_id += 1;

            staked_fomo
        }

        pub fn remove_stake(&mut self, staked_fomo: Bucket) -> Vec<Bucket> {
            assert!(
                staked_fomo.resource_address() == self.staked_fomo_resource_manager.address(),
                "Wrong coin bro",
            );

            let now = Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch;
            
            let staked_fomo_data = staked_fomo.as_non_fungible().non_fungible::<StakedFomoData>().data();
            assert!(
                now >= staked_fomo_data.minimum_unstake_date.seconds_since_unix_epoch,
                "Can't unstake now bro",
            );

            staked_fomo.burn();

            let ratio = staked_fomo_data.stake_share / self.total_stake_share;
            self.total_stake_share -= staked_fomo_data.stake_share;

            let mut coins = vec![];
            let mut amount = ratio * PreciseDecimal::from(self.fomo_vault.amount());
            coins.push(
                self.fomo_vault.take_advanced(
                    amount.checked_truncate(RoundingMode::ToZero).unwrap(),
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                )
            );

            let first_airdrop_id = staked_fomo_data.last_airdrop_id + 1;
            let last_airdrop_id = match self.last_airdrop_id < first_airdrop_id + MAX_AIRDROPS_PER_USER {
                true => self.last_airdrop_id,
                false => first_airdrop_id + MAX_AIRDROPS_PER_USER - 1,
            };
            for airdrop_id in first_airdrop_id..=last_airdrop_id {
                let mut airdrop = self.airdrops.get_mut(&airdrop_id).unwrap();

                amount = staked_fomo_data.stake_share * airdrop.amount_per_share;

                coins.push(
                    airdrop.vault.take_advanced(
                        amount.checked_truncate(RoundingMode::ToZero).unwrap(),
                        WithdrawStrategy::Rounded(RoundingMode::ToZero),
                    )
                );
            }

            coins
        }

        pub fn airdrop(&mut self, coins: Bucket) {
            let resource_address = coins.resource_address();
            assert!(
                resource_address.is_fungible(),
                "Can't airdrop NFTs",
            );

            let amount = coins.amount();
            assert!(
                amount > Decimal::ZERO,
                "No coin bro",
            );

            assert!(
                self.staked_fomo_resource_manager.total_supply().unwrap() > Decimal::ZERO,
                "No one to airdrop to",
            );

            if resource_address == self.fomo_vault.resource_address() {
                self.fomo_vault.put(coins);
            } else {
                self.last_airdrop_id += 1;
                self.airdrops.insert(
                    self.last_airdrop_id,
                    Airdrop {
                        vault: Vault::with_bucket(coins),
                        amount_per_share: amount / self.total_stake_share,
                    }
                );
            }

            Runtime::emit_event(AirdropEvent {
                coin: resource_address,
                amount: amount,
            });
        }

        pub fn deposit_rewards(&mut self, fomo: Bucket) {
            self.future_rewards.put(fomo);
        }

        pub fn airdrop_deposited_amount(&mut self, amount: Decimal) {
            assert!(
                self.staked_fomo_resource_manager.total_supply().unwrap() > Decimal::ZERO,
                "No one to airdrop to",
            );

            self.fomo_vault.put(
                self.future_rewards.take(amount)
            );

            Runtime::emit_event(AirdropEvent {
                coin: self.fomo_vault.resource_address(),
                amount: amount,
            });
        }
    }
}
