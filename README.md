# FOMO Staking blueprint

## This blueprint has the following characteristics

* It allows the owner to deposit future rewards ahead of time (just like the HIT rug-proof feature)
* It allows third parties to airdrop any fungible coin to all FOMO stakers
* It allows whoever instantiates the component to specify a minimum time that must elapse between stake and unstake
* Staking receipts are non fungible
* At each reward distribution an AirdropEvent is issued
* The add\_stake method is compatible with SelfiSocial guidelines

## Known limitations

When a user unstakes (remove\_stake) he can't receive more than MAX\_BUCKETS (100) different coins because of limitations on the number of events in a Radix transaction.  

## Below are the transaction manifests needed to use this contract:

### Instantiate (Stokenet)
```
CALL_FUNCTION
  Address("")
  "FomoStaking"
  "new"
  Address("<OWNER_BADGE>")
  Address("<FOMO_RESOURCE_ADDRESS>")
  <MINIMUM_STAKE_PERIOD>i64
;
```

### Add stake
```
CALL_METHOD
  Address("<ACCOUNT>")
  "withdraw"
  Address("<FOMO_RESOURCE_ADDRESS>")
  Decimal("<AMOUNT_TO_STAKE>")
;
TAKE_ALL_FROM_WORKTOP
  Address("<FOMO_RESOURCE_ADDRESS>")
  Bucket("tokens")
;
CALL_METHOD
	Address("<FOMO_STAKING_COMPONENT_ADDRESS>")
	"add_stake"
	Bucket("tokens")
;
CALL_METHOD
  Address("<ACCOUNT>")
  "deposit_batch"
  Expression("ENTIRE_WORKTOP")
;
```

### Remove stake
```
CALL_METHOD
  Address("<ACCOUNT>>")
  "withdraw_non_fungibles"
  Address("<STAKED_FOMO_RESOURCE_ADDRESS>")
  Array<NonFungibleLocalId>(NonFungibleLocalId("#<STAKED_FOMO_ID>#"))
;
TAKE_ALL_FROM_WORKTOP
  Address("<STAKED_FOMO_RESOURCE_ADDRESS>")
  Bucket("tokens")
;
CALL_METHOD
	Address("<FOMO_STAKING_COMPONENT_ADDRESS>")
	"remove_stake"
	Bucket("tokens")
;
CALL_METHOD
  Address("<ACCOUNT>")
  "deposit_batch"
  Expression("ENTIRE_WORKTOP")
;
```

### Airdrop any fungible
```
CALL_METHOD
  Address("<ACCOUNT>")
  "withdraw"
  Address("<RESOURCE_ADDRESS_TO_AIRDROP>")
  Decimal("<AMOUNT_TO_AIRDROP>")
;
TAKE_ALL_FROM_WORKTOP
  Address("<RESOURCE_ADDRESS_TO_AIRDROP>")
  Bucket("tokens")
;
CALL_METHOD
	Address("<FOMO_STAKING_COMPONENT_ADDRESS>")
	"airdrop"
	Bucket("tokens")
;
```

### Deposit future rewards (owner only)
```
CALL_METHOD
  Address("<ACCOUNT>")
  "create_proof_of_amount"
  Address("<OWNER_BADGE>")
  Decimal("1")
;
CALL_METHOD
  Address("<ACCOUNT>")
  "withdraw"
  Address("<FOMO_RESOURCE_ADDRESS>")
  Decimal("<AMOUNT>")
;
TAKE_ALL_FROM_WORKTOP
  Address("<FOMO_RESOURCE_ADDRESS>")
  Bucket("tokens")
;
CALL_METHOD
  Address("<FOMO_STAKING_COMPONENT_ADDRESS>")
  "deposit_rewards"
  Bucket("tokens")
;
```

### Distribute part of the previosly deposited rewards (owner only)
```
CALL_METHOD
  Address("<ACCOUNT>")
  "create_proof_of_amount"
  Address("<OWNER_BADGE>")
  Decimal("1")
;
CALL_METHOD
  Address("<FOMO_STAKING_COMPONENT_ADDRESS>")
  "airdrop_deposited_amount"
  Decimal("<AMOUNT>")
;
```
