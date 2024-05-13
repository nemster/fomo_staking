use scrypto::prelude::*;

// Event to emit at each airdrop.
#[derive(ScryptoSbor, ScryptoEvent)]
struct AirdropEvent {
    coin: ResourceAddress,
    amount: Decimal,
}

// NonFungibleData for the staking receipt
#[derive(Debug, ScryptoSbor, NonFungibleData)]
struct StakedFomoData {
    // When stake happened
    stake_date: Instant,

    // Can not be unstaked before this date
    minimum_unstake_date: Instant,

    // How many FOMO have been staked
    amount_staked: Decimal,

    // A number user to split rewards among the stakers
    stake_share: PreciseDecimal,

    // This receipt only allows receiving rewards for airdrops happening after this one
    last_airdrop_id: u64,
}

// A struct to store coins and information about non FOMO airdrops happened
#[derive(Debug, ScryptoSbor)]
struct Airdrop {
    coin: ResourceAddress,
    amount_per_share: PreciseDecimal,
}

// Metadata for the staking receipt
static STAKED_FOMO_NAME: &str = "Staked FOMO";
static STAKED_FOMO_ICON: &str = "https://pbs.twimg.com/media/GEfnpcUbIAAoEY8?format=jpg&name=small";

// Maximum number of buckets the remove_stake method will return.
// Raising this limit may cause transactions to fail
static MAX_BUCKETS: usize = 100;

#[blueprint]
#[events(AirdropEvent)]
mod fomo_staking {

    enable_method_auth! {
        methods {
            // Method to stake a bucket of FOMO, it returns a staking receipt
            add_stake => PUBLIC;

            remove_stake => PUBLIC;

            // Method to airdrop any fungible coin to current FOMO stakers
            airdrop => PUBLIC;

            // This method allows the owner to deposit future staking rewards in the component ahead of time
            deposit_rewards => restrict_to: [OWNER];

            // This method allows the owner to distribute the previously deposited rewards
            airdrop_deposited_amount => restrict_to: [OWNER];
        }
    }

    struct FomoStaking {
        // Vault to keep the staked FOMO
        fomo_vault: Vault,

        // KVS where the non FOMO coins are stored
        vaults: KeyValueStore<ResourceAddress, Vault>,

        // KVS where all non FOMO Airdrop objects are stored
        airdrops: KeyValueStore<u64, Airdrop>,

        // Optionally, stake can have a minimum length
        minimum_stake_period: i64,

        // ResourceManager to mint staking receipts
        staked_fomo_resource_manager: ResourceManager,

        // NonFungibleLocalId of the next staking receipt
        next_staked_fomo_id: u64,

        // The sum of all of the stake_share in the staking receipt
        total_stake_share: PreciseDecimal,

        // Optionally, the Owner can deposit the future FOMO rewards in this Vault ahead of time.
        future_rewards: Vault,

        // Numeric index of the last Airdrop object in the airdrops KVS
        last_airdrop_id: u64,
    }

    impl FomoStaking {

        // IFunction to instantiate a new FomoStaking component
        pub fn new(

            // The resource to set as owner badge for the component and the staking receipts it will mint
            owner_badge_address: ResourceAddress,

            // The resource address of FOMO
            fomo_address: ResourceAddress,

            // The minimum time has to pass from stake to unstake (seconds).
            // It can be zero to let the user unstake whenever he wants.
            minimum_stake_period: i64,

        ) -> Global<FomoStaking> {
            // Make sure minimum_stake_period isn't a negative number
            assert!(
                minimum_stake_period >= 0,
                "Invalid minimum_stake_period",
            );

            // Reserve a ComponentAddress for setting rules on resources
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(FomoStaking::blueprint_id());

            // Create a ResourceManager for minting staking receipts
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
                non_fungible_data_updater => rule!(require(global_caller(component_address)));
                non_fungible_data_updater_updater => rule!(require(owner_badge_address));
            ))
            .burn_roles(burn_roles!(
                burner => rule!(require(global_caller(component_address)));
                burner_updater => rule!(deny_all);
            ))
            .create_with_no_initial_supply();

            // Instantiate the component and globalize it
            Self {
                fomo_vault: Vault::new(fomo_address),
                vaults: KeyValueStore::new(),
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

        // Method to stake a bucket of FOMO, it returns a staking receipt
        pub fn add_stake(&mut self, fomo: Bucket) -> Bucket {
            // Make sure the bucket contains FOMO
            assert!(
                fomo.resource_address() == self.fomo_vault.resource_address(),
                "Wrong coin bro",
            );

            // Count the coins in the bucket
            let amount = fomo.amount();
            assert!(
                amount > Decimal::ZERO,
                "No coin bro",
            );

            // Count the already staked coins
            let vault_amount = self.fomo_vault.amount();

            // Empty the bucket in the vault
            self.fomo_vault.put(fomo);

            // Set stake_share and increase total_stake_share so that stake_share/total_stake_share represents the
            // share of FOMO in the vault for this user.
            let mut stake_share = PreciseDecimal::ONE;
            if vault_amount > Decimal::ZERO {
                stake_share = (self.total_stake_share * amount) / vault_amount;
            }
            self.total_stake_share += stake_share;

            // Get current Instant
            let now = Clock::current_time_rounded_to_seconds();

            // Mint a staking receipt
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

            // Prepare for minting the next staking receipt
            self.next_staked_fomo_id += 1;

            // Return the staking receipt to the user
            staked_fomo
        }

        pub fn remove_stake(&mut self, staked_fomo: Bucket) -> Vec<Bucket> {
            // Make sure staked_fomo is a staking receipt
            assert!(
                staked_fomo.resource_address() == self.staked_fomo_resource_manager.address(),
                "Wrong coin bro",
            );

            // Read NonFungibleData from the bucket (it must contain exactly 1 staking receipt)
            let staked_fomo_data = staked_fomo.as_non_fungible().non_fungible::<StakedFomoData>().data();

            // Make sure the minimum stakeing time has passed
            let now = Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch;
            assert!(
                now >= staked_fomo_data.minimum_unstake_date.seconds_since_unix_epoch,
                "Can't unstake now bro",
            );

            // Burn the staking receipt
            staked_fomo.burn();

            // Compute the % of total FOMO to be unstaked and update total_stake_share accordingly
            let ratio = staked_fomo_data.stake_share / self.total_stake_share;
            self.total_stake_share -= staked_fomo_data.stake_share;

            // Prepare a vector for all of the buckets to return
            let mut coins: Vec<Bucket> = vec![];

            // Create the FOMO bucket and add it to the vector
            let mut amount = ratio * PreciseDecimal::from(self.fomo_vault.amount());
            coins.push(
                self.fomo_vault.take_advanced(
                    amount.checked_truncate(RoundingMode::ToZero).unwrap(),
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                )
            );

            // Prepare a HashMap to store the total the user must receive per each non FOMO coin
            let mut totals: HashMap<ResourceAddress, PreciseDecimal> = HashMap::with_capacity(MAX_BUCKETS);

            // For each non FOMO airdrop happened during the staking period
            for airdrop_id in staked_fomo_data.last_airdrop_id + 1 ..= self.last_airdrop_id {
                // Find the airdrop information
                let airdrop = self.airdrops.get_mut(&airdrop_id).unwrap();

                // Compute the amount this user must receive
                amount = staked_fomo_data.stake_share * airdrop.amount_per_share;

                // Is this coin already in the totals HashMap?
                if totals.get(&airdrop.coin).is_some() {
                    // If so, put the coins in this bucket
                    *totals.get_mut(&airdrop.coin).unwrap() += amount;
                } else {
                    // If not, can we create another one without exceeding MAX_BUCKETS?
                    if totals.len() < MAX_BUCKETS {
                        // If so, add a new element to totals
                        totals.insert(airdrop.coin, amount);
                    } // Otherwise those coins are lost forever :-(
                }
            }

            // For each coin the user has to receive
            for (resource_address, amount) in totals.iter() {
                // Create a new bucket with the number of coins he has to receve and add it to the coins vector
                coins.push(
                    self.vaults.get_mut(&resource_address).unwrap().take_advanced(
                        amount.checked_truncate(RoundingMode::ToZero).unwrap(),
                        WithdrawStrategy::Rounded(RoundingMode::ToZero),
                    )
                );
            }

            // Give the user all of his coins
            coins
        }

        // Method to airdrop any fungible coin to current FOMO stakers
        pub fn airdrop(&mut self, coins: Bucket) {
            // Get the resource_address of the coin to airdrop and make sure it is a fungible
            let resource_address = coins.resource_address();
            assert!(
                resource_address.is_fungible(),
                "Can't airdrop NFTs",
            );

            // Make sure there are actually coins in the bucket
            let amount = coins.amount();
            assert!(
                amount > Decimal::ZERO,
                "No coin bro",
            );

            // If no one is staking, the coins will be lost forever
            assert!(
                self.staked_fomo_resource_manager.total_supply().unwrap() > Decimal::ZERO,
                "No one to airdrop to",
            );

            // If the bucket contains some FOMO, just put them in the vault of the staked FOMO
            if resource_address == self.fomo_vault.resource_address() {
                self.fomo_vault.put(coins);
            } else {
                // If the bucket contains any fungible different from FOMO, create a new Airdrop object and add it to
                // the KVS
                self.last_airdrop_id += 1;
                self.airdrops.insert(
                    self.last_airdrop_id,
                    Airdrop {
                        coin: resource_address,
                        amount_per_share: amount / self.total_stake_share,
                    }
                );

                // Does a vault for this coins already exist?
                if self.vaults.get(&resource_address).is_some() {
                    // If so, put the coins in it
                    self.vaults.get_mut(&resource_address).unwrap().put(coins);
                } else {
                    // Else create the vault with the coins in it
                    self.vaults.insert(resource_address, Vault::with_bucket(coins));
                }
            }

            // Tell everyone the party is here
            Runtime::emit_event(AirdropEvent {
                coin: resource_address,
                amount: amount,
            });
        }

        // This method allows the owner to deposit future staking rewards in the component ahead of time
        pub fn deposit_rewards(&mut self, fomo: Bucket) {
            self.future_rewards.put(fomo);
        }

        // This method allows the owner to distribute the previously deposited rewards
        pub fn airdrop_deposited_amount(&mut self, amount: Decimal) {
            // If no one is staking, the coins will be lost forever
            assert!(
                self.staked_fomo_resource_manager.total_supply().unwrap() > Decimal::ZERO,
                "No one to airdrop to",
            );

            // Take the required amount from the future rewards and put it in the vault of the staked FOMO
            self.fomo_vault.put(
                self.future_rewards.take(amount)
            );

            // Tell everyone the party is here
            Runtime::emit_event(AirdropEvent {
                coin: self.fomo_vault.resource_address(),
                amount: amount,
            });
        }
    }
}
