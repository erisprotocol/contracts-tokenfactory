# Eris Amplified Staking

Kujira liquid staking derivative. Of the community, by the community, for the community.

The version ([v1.1.1](https://github.com/erisprotocol/contracts-terra/releases/tag/v1.1.1)) of the Eris Amplifier Hub on Terra + Terra Classic is audited by [SCV Security](https://twitter.com/TerraSCV) ([link](https://github.com/SCV-Security/PublicReports/blob/main/CW/ErisProtocol/Eris%20Protocol%20-%20Amplified%20Staking%20-%20Audit%20Report%20v1.0.pdf)).

A previous version ([v1.0.0-rc0](https://github.com/st4k3h0us3/steak-contracts/releases/tag/v1.0.0-rc0)) of Steak was audited by [SCV Security](https://twitter.com/TerraSCV) ([link](https://github.com/SCV-Security/PublicReports/blob/main/CW/St4k3h0us3/St4k3h0us3%20-%20Steak%20Contracts%20Audit%20Review%20-%20%20v1.0.pdf)).

## Contracts

| Contract                               | Description                                              |
| -------------------------------------- | -------------------------------------------------------- |
| [`erist-staking-hub`](./contracts/hub) | Manages minting/burning of ampKUJI token and bonded Kuji |

For the routing of the swap the fin-multi router is used. See <https://github.com/Team-Kujira/fin-multi>

## Building

For interacting with the smart contract clone <https://github.com/erisprotocol/liquid-staking-scripts> into the same parent folder.

## Changelog

### Hub Version 1.2.1

Allows splitting Black Whale vault tokens
Allows auto compounding any token received through staking
Will not allow to swap from KUJI or ampKUJI

## License

This repository is a fork of <https://github.com/steak-enjoyers/steak>

Contents of this repository are open source under [GNU General Public License v3](./LICENSE) or later.
