#!/bin/sh
# #sample script for testing out working of exec and query functions

# #export node and transaction details
export NODE="--node http://127.0.0.1:26657"
export TXFLAG="${NODE} --chain-id test --gas-prices 0.01stake --gas auto --gas-adjustment 1.3"


#storing the contract on chain
echo -e "---------------storing contract on chain------------------ \n"
export RES=$(comdex tx wasm store ./wordex/artifacts/wordex.wasm --from test --chain-id test --gas-prices 0.01stake --gas auto --gas-adjustment 1.3 -y --output json -b block)

export CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[0].value') 
echo "contract id on chain: $CODE_ID"


#instantiate the contract
echo -e "\n \n --------------- instantiating contract----------------------"
INSTANTIATE='{"denom":"wdx","max_cap":1000000}'
comdex tx wasm instantiate $CODE_ID "$INSTANTIATE" --label "wordex" --from test -y --no-admin --chain-id test

# Check the contract details and account balance
echo -e "\n check few contract details"
comdex query wasm list-contract-by-code $CODE_ID $NODE --output json
CONTRACT=$(comdex query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')
echo $CONTRACT


# Note that keys are hex encoded, and the values are base64 encoded.
# To view the returned data (assuming it is ascii), try something like:
# (Note that in many cases the binary data returned is not in ascii format, thus the encoding)
# comdex query wasm contract-state all $CONTRACT $NODE --output "json" | jq -r '.models[0].key' | xxd -r -ps
#this cmd gives contract state in json format
echo -e "\n\n ------------- check contract state in json ------------- "
comdex query wasm contract-state all $CONTRACT $NODE --output "json" | jq -r '.models[0].value' | base64 -d

# Create a player for the wallet address
CREATE='{"create_player":{"name":"gamer"}}'
echo -e "\n\n -----------------create player named gamer -------------"
comdex tx wasm execute $CONTRACT "$CREATE" --from smartkey $TXFLAG -y

# So, we can also try "smart querying" the contract
echo -e "\n\n ---------------check player details---------------"
PLAYER_QUERY='{"query_player": {"addr": "comdex1qmdzhutg0txx63usrzjq47rascl64spcchhzdz"}}'
comdex query wasm contract-state smart $CONTRACT "$PLAYER_QUERY" $NODE --output json
#player exists or not
echo -e "\n\n -----------------check player exists or not ---------------\n"
PLAYER_QUERY2='{"query_player_exists": {"addr": "comdex1qmdzhutg0txx63usrzjq47rascl64spcchhzdz"}}'
comdex query wasm contract-state smart $CONTRACT "$PLAYER_QUERY2" $NODE --output json

#start game
echo -e "\n------------------------starting the game-------------------\n"
START='{"start_game":{"game_words":["layer","roses","noise","viral","closed"]}}'
comdex tx wasm execute $CONTRACT "$START" --from smartkey $TXFLAG -y

# query for the getting the word at pos
echo -e "\n ---------------querying for word at pos------------------\n"
PLAYER_QUERY3='{"query_player_word": {"addr": "comdex1qmdzhutg0txx63usrzjq47rascl64spcchhzdz", "pos":3}}'
comdex query wasm contract-state smart $CONTRACT "$PLAYER_QUERY3" $NODE --output json

#query for guess - whether each pos in word is correct
echo -e "\n ---------------------- querying whether guess is correct ------------ \n"
PLAYER_QUERY4='{"query_correct_guess": {"addr": "comdex1qmdzhutg0txx63usrzjq47rascl64spcchhzdz","guessed":"noise","pos":2}}'
comdex query wasm contract-state smart $CONTRACT "$PLAYER_QUERY4" $NODE --output json

echo -e "\n --------------------- checking state after start of game ------------------ \n"
comdex query wasm contract-state smart $CONTRACT "$PLAYER_QUERY" $NODE --output json

# #update guess
# echo -e "\n--------------------guess update-------------------"
# UPDATE1='{"update_game":{"game":false,"guess":true,"correct_guess":false,"game_won":false,"wrong_guess":false}}'
# comdex tx wasm execute $CONTRACT "$UPDATE1" --from smartkey $TXFLAG -y

#update all - game, guess , correct guess and games won in set
echo -e "\n ---------------------------game update---------------------\n"
UPDATE2='{"update_game":{"game":5,"guess":5,"correct_guess":5,"game_won":5,"wrong_guess":0}}'
comdex tx wasm execute $CONTRACT "$UPDATE2" --from smartkey $TXFLAG -y

# #update game won
# echo -e "\n ---------------------game won update-----------------------"
# UPDATE3='{"update_game":{"game":false,"guess":false,"correct_guess":false,"game_won":true,"wrong_guess":false}}'
# comdex tx wasm execute $CONTRACT "$UPDATE3" --from smartkey $TXFLAG -y

# echo -e "\n for set 2"
# comdex tx wasm execute $CONTRACT "$UPDATE1" --from smartkey $TXFLAG -y
# comdex tx wasm execute $CONTRACT "$UPDATE2" --from smartkey $TXFLAG -y
# comdex tx wasm execute $CONTRACT "$UPDATE3" --from smartkey $TXFLAG -y

# echo -e "\n for set 3"
# comdex tx wasm execute $CONTRACT "$UPDATE1" --from smartkey $TXFLAG -y
# comdex tx wasm execute $CONTRACT "$UPDATE2" --from smartkey $TXFLAG -y
# comdex tx wasm execute $CONTRACT "$UPDATE3" --from smartkey $TXFLAG -y

# echo -e "\n for set 4"
# comdex tx wasm execute $CONTRACT "$UPDATE1" --from smartkey $TXFLAG -y
# comdex tx wasm execute $CONTRACT "$UPDATE2" --from smartkey $TXFLAG -y
# comdex tx wasm execute $CONTRACT "$UPDATE3" --from smartkey $TXFLAG -y

# echo -e "\n for set 5"
# comdex tx wasm execute $CONTRACT "$UPDATE1" --from smartkey $TXFLAG -y
# comdex tx wasm execute $CONTRACT "$UPDATE2" --from smartkey $TXFLAG -y
# comdex tx wasm execute $CONTRACT "$UPDATE3" --from smartkey $TXFLAG -y


#checking for contract state before rewarding
echo -e "\n\n checking for possible ghaplas before reward"
comdex query wasm contract-state smart $CONTRACT "$PLAYER_QUERY" $NODE --output json

#reward_player
echo -e "\n \n ---------------rewarding the player -------------------"
REWARD='{"reward_player":{}}'
comdex tx wasm execute $CONTRACT "$REWARD" --from smartkey $TXFLAG -y


#checking for contract state before ending
echo -e "\n\n checking for possible ghaplas before end"
comdex query wasm contract-state smart $CONTRACT "$PLAYER_QUERY" $NODE --output json

#end game
echo -e "\n \n -------------------------ending the game -----------------------"
END='{"end_game":{}}'
comdex tx wasm execute $CONTRACT "$END" --from smartkey $TXFLAG -y

#querying again to check whether ending is safe
echo -e "\n\n ---------------querying after end of game -------------\n"
comdex query wasm contract-state smart $CONTRACT "$PLAYER_QUERY" $NODE --output json