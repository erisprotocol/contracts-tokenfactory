# Eris Adapter

This adapter contains the abstraction level for each chain.

## Reference Structure

The reference structure is always like this:

[contracts] -> [eris], [eris-chain-adapter], [eris-chain-shared]

eris-chain-adapter has feature flags for each chain. Based on the setting of the feature flag, a different chain package is used.
[eris-chain-adapter] -> [eris-kujira] -> [eris-chain-shared]

Eris Kujira Test contains all special test cases for the kujira blockchain.

[eris-kujira-test] -> [*]
