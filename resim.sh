#!/bin/bash

set -e

OUTPUTFILE=$(mktemp)

resim reset

echo
resim new-account >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export account=$(grep 'Account component address:' $OUTPUTFILE | cut -d ' ' -f 4)
export owner_badge=$(grep 'Owner badge:' $OUTPUTFILE | cut -d ':' -f 2)
echo Account address: $account, Owner badge: $owner_badge

resim new-token-fixed 5000 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export fomo=$(grep 'Resource:' $OUTPUTFILE | cut -d ' ' -f 3)
echo FOMO ResourceAddress: $fomo

resim publish . >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export package=$(grep 'Success! New Package:' $OUTPUTFILE | cut -d ' ' -f 4)
echo Package: $package

resim call-function ${package} FomoStaking new ${owner_badge} ${fomo} 2592000 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export component=$(grep 'Component:' $OUTPUTFILE | cut -d ' ' -f 3)
export staked_fomo=$(grep 'Resource:' $OUTPUTFILE | cut -d ' ' -f 3)
echo Component address: $component, Staked FOMO NFT address: $staked_fomo

echo
resim set-current-time 2024-06-02T12:00:00Z

echo
resim call-method ${component} add_stake $fomo:100 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
nft_id1=$(grep -A 1 "ResAddr: $staked_fomo" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)
echo "100 FOMO staked, NFT #${nft_id1}# received (should be 1)"

resim call-method ${component} add_stake $fomo:50 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
nft_id2=$(grep -A 1 "ResAddr: $staked_fomo" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)
echo "50 FOMO staked, NFT #${nft_id2}# received (should be 2)"

echo
resim call-method ${component} deposit_rewards $fomo:750 --proofs "${owner_badge}:#1#" >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo 750 FOMO deposited as future rewards
resim call-method ${component} airdrop_deposited_amount 100 --proofs "${owner_badge}:#1#" >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo 100 FOMO airdropped

echo
resim new-token-fixed 5000 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export coin2=$(grep 'Resource:' $OUTPUTFILE | cut -d ' ' -f 3)
echo COIN2 ResourceAddress: $coin2

for i in $(seq 1 250)
do
	resim call-method ${component} airdrop ${coin2}:1 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
	echo -n .
done
echo 250 airdrops of 1 COIN2 each

echo
resim call-method ${component} add_stake $fomo:250 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
nft_id3=$(grep -A 1 "ResAddr: $staked_fomo" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)
echo "250 FOMO staked, NFT #${nft_id3}# received (should be 3)"

echo
resim new-token-fixed 5000 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export coin3=$(grep 'Resource:' $OUTPUTFILE | cut -d ' ' -f 3)
echo COIN3 ResourceAddress: $coin3
resim call-method ${component} airdrop ${coin3}:1000 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo 1000 COIN3 airdropped

echo
resim set-current-time 2024-08-02T12:00:00Z

echo
export nft_id=${nft_id1}
export ignore_coins="Address(\"$coin3\")"
resim run manifests/remove_stake.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export withdraw_events=$(grep WithdrawEvent $OUTPUTFILE | wc -l)
export deposit_events=$(grep DepositEvent $OUTPUTFILE | wc -l)
export fee_paid=$(grep -A 1 resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3 $OUTPUTFILE | grep 'Change: -' | cut -d - -f 2)
echo "NFT #${nft_id}# unstaked, ignoring COIN3, ${withdraw_events} WithdrawEvents, ${deposit_events} DepositEvents, XRD ${fee_paid} fee paid "
export fomo_received=$(grep -A 1 "ResAddr: $fomo" $OUTPUTFILE | tail -n 1 | cut -d : -f 2)
export coin2_received=$(grep -A 1 "ResAddr: $coin2" $OUTPUTFILE | tail -n 1 | cut -d : -f 2)
export coin3_received=$(grep -A 1 "ResAddr: $coin3" $OUTPUTFILE | tail -n 1 | cut -d : -f 2)
echo "FOMO received: ${fomo_received} (should be 166), COIN2 received: ${coin2_received} (should be 166), COIN3 received: ${coin3_received} (should be zero)"

echo
for i in $(seq 1 80)
do
	resim new-token-fixed 5000 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
	export coin=$(grep 'Resource:' $OUTPUTFILE | cut -d ' ' -f 3)
        resim call-method ${component} airdrop ${coin}:1 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
        echo -n .
done
echo 80 new coins created, 80 airdrops done

echo
export nft_id=${nft_id2}
export ignore_coins=""
resim run manifests/remove_stake.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export withdraw_events=$(grep WithdrawEvent $OUTPUTFILE | wc -l)
export deposit_events=$(grep DepositEvent $OUTPUTFILE | wc -l)
export fee_paid=$(grep -A 1 resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3 $OUTPUTFILE | grep 'Change: -' | cut -d - -f 2)
echo "NFT #${nft_id}# unstaked, ${withdraw_events} WithdrawEvents, ${deposit_events} DepositEvents, XRD ${fee_paid} fee paid "
export fomo_received=$(grep -A 1 "ResAddr: $fomo" $OUTPUTFILE | tail -n 1 | cut -d : -f 2)
export coin2_received=$(grep -A 1 "ResAddr: $coin2" $OUTPUTFILE | tail -n 1 | cut -d : -f 2)
export coin3_received=$(grep -A 1 "ResAddr: $coin3" $OUTPUTFILE | tail -n 1 | cut -d : -f 2)
echo "FOMO received: ${fomo_received} (should be 83), COIN2 received: ${coin2_received} (should be 83), COIN3 received: ${coin3_received} (should be 166)"

echo
export nft_id=${nft_id3}
export ignore_coins=""
resim run manifests/remove_stake.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export withdraw_events=$(grep WithdrawEvent $OUTPUTFILE | wc -l)
export deposit_events=$(grep DepositEvent $OUTPUTFILE | wc -l)
export fee_paid=$(grep -A 1 resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3 $OUTPUTFILE | grep 'Change: -' | cut -d - -f 2)
echo "NFT #${nft_id}# unstaked, ${withdraw_events} WithdrawEvents, ${deposit_events} DepositEvents, XRD ${fee_paid} fee paid "
export fomo_received=$(grep -A 1 "ResAddr: $fomo" $OUTPUTFILE | tail -n 1 | cut -d : -f 2)
export coin2_received=$(grep -A 1 "ResAddr: $coin2" $OUTPUTFILE | tail -n 1 | cut -d : -f 2)
export coin3_received=$(grep -A 1 "ResAddr: $coin3" $OUTPUTFILE | tail -n 1 | cut -d : -f 2)
echo "FOMO received: ${fomo_received} (should be 250), COIN2 received: ${coin2_received} (should be zero), COIN3 received: ${coin3_received} (should be 500)"
