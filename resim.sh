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

echo
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
export stake1=100
resim call-method ${component} add_stake $fomo:$stake1 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export nft_id1=$(grep -A 1 "ResAddr: $staked_fomo" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)
echo "$stake1 FOMO staked, NFT #${nft_id1}# received (should be 1)"

echo
export stake2=50
resim call-method ${component} add_stake $fomo:$stake2 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export nft_id2=$(grep -A 1 "ResAddr: $staked_fomo" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)
echo "$stake2 FOMO staked, NFT #${nft_id2}# received (should be 2)"

echo
export airdrop1=100
resim call-method ${component} deposit_rewards $fomo:$((2 * $airdrop1)) --proofs "${owner_badge}:#1#" >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo $((2 * $airdrop1)) FOMO deposited as future rewards
resim call-method ${component} airdrop_deposited_amount $airdrop1 --proofs "${owner_badge}:#1#" >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export total_stake=$(($stake1 + $stake2))
export stake1=$(($stake1 + ($airdrop1 * $stake1) / $total_stake))
export stake2=$(($stake2 + ($airdrop1 * $stake2) / $total_stake))
export total_stake=$(($stake1 + $stake2))
echo $airdrop1 FOMO airdropped

echo
resim new-token-fixed 5000 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export coin2=$(grep 'Resource:' $OUTPUTFILE | cut -d ' ' -f 3)
echo COIN2 ResourceAddress: $coin2

export iterations2=300
export airdrop2=1
for i in $(seq 1 $iterations2)
do
	resim call-method ${component} airdrop ${coin2}:$airdrop2 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
	echo -n .
done
export coin2_1=$((($iterations2 * $airdrop2 * $stake1) / ${total_stake}))
export coin2_2=$((($iterations2 * $airdrop2 * $stake2) / ${total_stake}))
echo $iterations2 airdrops of $airdrop2 COIN2 each

echo
export stake3=250
resim call-method ${component} add_stake $fomo:$stake3 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
nft_id3=$(grep -A 1 "ResAddr: $staked_fomo" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)
export total_stake=$(($stake1 + $stake2 + $stake3))
echo "$stake3 FOMO staked, NFT #${nft_id3}# received (should be 3)"

echo
resim new-token-fixed 5000 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export coin3=$(grep 'Resource:' $OUTPUTFILE | cut -d ' ' -f 3)
export airdrop3=1000
echo COIN3 ResourceAddress: $coin3
resim call-method ${component} airdrop ${coin3}:$airdrop3 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export coin3_1=$((($airdrop3 * $stake1) / ${total_stake}))
export coin3_2=$((($airdrop3 * $stake2) / ${total_stake}))
export coin3_3=$((($airdrop3 * $stake3) / ${total_stake}))
echo $airdrop3 COIN3 airdropped

echo
resim set-current-time 2024-08-02T12:00:00Z

echo
export nft_id=${nft_id1}
export ignore_coins="Address(\"$coin3\")"
export max_airdrops=350
export no_fomo=true
resim run manifests/remove_stake.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export events=$(grep Event: $OUTPUTFILE | wc -l)
export fee_paid=$(grep -A 1 resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3 $OUTPUTFILE | grep 'Change: -' | cut -d - -f 2)
echo "NFT #${nft_id}# unstaked, ignore_coins ${ignore_coins}, max_airdrops ${max_airdrops}, no_fomo ${no_fomo}, $events events, XRD ${fee_paid} fee paid "
export fomo_received=$(grep -A 1 "ResAddr: $fomo" $OUTPUTFILE | tail -n 1 | cut -d : -f 2)
export coin2_received=$(grep -A 1 "ResAddr: $coin2" $OUTPUTFILE | tail -n 1 | cut -d : -f 2)
export coin3_received=$(grep -A 1 "ResAddr: $coin3" $OUTPUTFILE | tail -n 1 | cut -d : -f 2)
echo "FOMO received: ${fomo_received} (should be zero), COIN2 received: ${coin2_received} (should be about ${coin2_1}), COIN3 received: ${coin3_received} (should be zero)"

echo
export iterations4=79
export airdrops4=4
for i in $(seq 1 $iterations4)
do
	resim new-token-fixed 5000 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
	export coin=$(grep 'Resource:' $OUTPUTFILE | cut -d ' ' -f 3)
	for j in $(seq 1 $airdrops4)
	do
        	resim call-method ${component} airdrop ${coin}:1 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
	done
        echo -n .
done
echo $iterations4 new coins created, $(($iterations4 * $airdrops4)) airdrops done

echo
export nft_id=${nft_id3}
export ignore_coins=""
export max_airdrops=350
export no_fomo=false
resim run manifests/remove_stake.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export events=$(grep Event: $OUTPUTFILE | wc -l)
export fee_paid=$(grep -A 1 resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3 $OUTPUTFILE | grep 'Change: -' | cut -d - -f 2)
echo "NFT #${nft_id}# unstaked, ignore_coins ${ignore_coins}, max_airdrops ${max_airdrops}, no_fomo ${no_fomo}, $events events, XRD ${fee_paid} fee paid"
export fomo_received=$(grep -A 1 "ResAddr: $fomo" $OUTPUTFILE | tail -n 1 | cut -d : -f 2)
export coin2_received=$(grep -A 1 "ResAddr: $coin2" $OUTPUTFILE | tail -n 1 | cut -d : -f 2)
export coin3_received=$(grep -A 1 "ResAddr: $coin3" $OUTPUTFILE | tail -n 1 | cut -d : -f 2)
echo "FOMO received: ${fomo_received} (should be about $stake3), COIN2 received: ${coin2_received} (should be zero), COIN3 received: ${coin3_received} (should be about ${coin3_3})"

echo
export nft_id=${nft_id2}
export ignore_coins=""
export max_airdrops=350
export no_fomo=false
resim run manifests/remove_stake.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export events=$(grep Event: $OUTPUTFILE | wc -l)
export fee_paid=$(grep -A 1 resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3 $OUTPUTFILE | grep 'Change: -' | cut -d - -f 2)
echo "NFT #${nft_id}# unstaked, ignore_coins ${ignore_coins}, max_airdrops ${max_airdrops}, no_fomo ${no_fomo}, $events events, XRD ${fee_paid} fee paid"
export fomo_received=$(grep -A 1 "ResAddr: $fomo" $OUTPUTFILE | tail -n 1 | cut -d : -f 2)
export coin2_received=$(grep -A 1 "ResAddr: $coin2" $OUTPUTFILE | tail -n 1 | cut -d : -f 2)
export coin3_received=$(grep -A 1 "ResAddr: $coin3" $OUTPUTFILE | tail -n 1 | cut -d : -f 2)
echo "FOMO received: ${fomo_received} (should be zero), COIN2 received: ${coin2_received} (should be about ${coin2_2}), COIN3 received: ${coin3_received} (should be about ${coin3_2})"
resim run manifests/remove_stake.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export fomo_received=$(grep -A 1 "ResAddr: $fomo" $OUTPUTFILE | tail -n 1 | cut -d : -f 2)
export events=$(grep Event: $OUTPUTFILE | wc -l)
export fee_paid=$(grep -A 1 resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3 $OUTPUTFILE | grep 'Change: -' | cut -d - -f 2)
echo "NFT #${nft_id}# unstaked again, ignore_coins ${ignore_coins}, max_airdrops ${max_airdrops}, no_fomo ${no_fomo}, $events events, XRD ${fee_paid} fee paid"
echo "FOMO received: ${fomo_received} (should be about $stake2)"

echo
resim call-method ${component} airdrop_deposited_amount $airdrop1 --proofs "${owner_badge}:#1#" >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export stake1=$(($stake1 + $airdrop1))
echo $airdrop1 FOMO airdropped

echo
export nft_id=${nft_id1}
export ignore_coins=""
export max_airdrops=350
export no_fomo=false
resim run manifests/remove_stake.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export fomo_received=$(grep -A 1 "ResAddr: $fomo" $OUTPUTFILE | tail -n 1 | cut -d : -f 2)
export events=$(grep Event: $OUTPUTFILE | wc -l)
export fee_paid=$(grep -A 1 resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3 $OUTPUTFILE | grep 'Change: -' | cut -d - -f 2)
echo "NFT #${nft_id}# unstaked again, ignore_coins ${ignore_coins}, max_airdrops ${max_airdrops}, no_fomo ${no_fomo}, $events events, XRD ${fee_paid} fee paid"
echo "FOMO received: ${fomo_received} (should be about $stake1)"
