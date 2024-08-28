# fun_swap
FunSwap is a simple token swap program built on the Solana blockchain using the Anchor framework. This program allows two parties to swap tokens with a deadline and grace period for approval. The program ensures atomic swaps, allowing both parties to exchange tokens safely within a specified timeframe.

This project was developred for the same private client I made (https://github.com/btorressz/FunSwap) for.

## Features
- **Atomic Token Swap: Ensures both parties exchange tokens simultaneously.**
- **Grace Period: Allows for a grace period after the deadline for the swap to be approved.**
- **Automatic Expiration: Tokens are returned to the original owners if the swap expires.**
- **Deadline Extension: The deadline for the swap can be extended by Party A.**

  ## License
This project is licensed under the MIT License.
