# License

amp-compounder has started as a fork of <https://github.com/spectrumprotocol/spectrum-core> but has significantly evolved.

## Changes

- Maintaining and updating dependencies. Removing unneeded pair proxy.

- fees_collector
  - Support sending funds to a smart contract with a predefined message to wake it up.
  - Support topping up operation wallets to a specified amount
- astroport_farm
  - Use minting / burning of amp[LP] token instead of tracking reward info locally
  - Query interface changed to include exchange_rate and more info
  - Support Astro rewards as native denom, as it is on Neutron
  - Support using TokenFactory as amp[LP]
- compound_proxy
  - Instead of supporting only a single compound LP, supports a list of LPs
  - Supports zapping from any asset to LP.
  - Support integration of TFM based routes
