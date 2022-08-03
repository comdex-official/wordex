# Wordle game CLI

This is the smart contract for a Wordle game and can be tested out on shell. This is based on the Cosmwasm starter pack codebase. The game gives a player 8hours from the start of the game to complete 5 wordle challenges. The player doesn't need to pay any fees to enter the game. All 5 challenges are to be won to win reward. Player gets 18 guesses to find the 5 words. Also all 5 words will be distinct in a particular game. If at the end of some challenge the 8hour time is exceeded, the game ends. 

## Development perspective for front end

All words (5 letter) are present in the ```all_words.txt```. Those are not stored in smart contract since that made the contract binary more than 200K even after optimized compilation. To get an idea of what's possible with command set of the contract one can run the script file in the repo.

```sh
chmod +x wordex_test.sh && bash wordex_test.sh
```

## What I have missed on :(
I wasn't aware of the thing that I had to write separate contract for the currency used in the game. Anyways that can be included in this contract as well. The cw20-base contract can be used. I was not getting much hold of Cosmos SDK frontend technology. So I hithered to build the frontend. However it was fun to write such a contract that works fine even on CLI. 

## If you have any suggestions ....
If you find any fault in the code or contract kindly write to abhilashxaviers@gmail.com. I will happy to address. Sideways, if you have any ideas to write contracts on, we can team up. 
