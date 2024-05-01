# FOMO Staking contract
 
Below are the transaction manifests needed to use the contract:

## instantiate (stokenet)
```
CALL_FUNCTION
  Address("package_tdx_2_1p4prtgrq95e8vssmcg0jwsultu2dc39vt3j53zqmdums4mz8ydfhst")
  "FomoStaking"
  "new"
  Address("<OWNER_BADGE>")
  Address("<FOMO_RESOURCE_ADDRESS>")
  <MINIMUM_STAKE_PERIOD>i64
;
```

## add stake
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

## remove stake
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

## airdrop
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
