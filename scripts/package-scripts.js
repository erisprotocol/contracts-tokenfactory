// const npsUtils = require("nps-utils"); // not required, but handy!

module.exports = {
  scripts: {
    release: {
      default: "bash build_release.sh",
    },
    schema: {
      default:
        "nps schema.create schema.transform  schema.farm schema.generator",

      transform: "ts-node transform.ts",

      create: "bash build_schema.sh",

      hub: "cd .. && json2ts -i contracts/hub/**/*.json -o ../liquid-staking-scripts/types/tokenfactory/hub",

      // ampz: "cd .. && json2ts -i contracts/ampz/schema/*.json -o ../liquid-staking-scripts/types/ampz",
      arb: "cd .. && json2ts -i contracts/arb-vault/schema/*.json -o ../liquid-staking-scripts/types/tokenfactory/arb-vault",

      // token:
      //   "cd .. && json2ts -i contracts/token/**/*.json -o ../liquid-staking-scripts/types/token",
      // ampextractor:
      //   "cd .. && json2ts -i contracts/amp-extractor/**/*.json -o ../liquid-staking-scripts/types/amp-extractor",

      votingescrow:
        "cd .. && json2ts -i contracts/amp-governance/voting_escrow/**/*.json -o ../liquid-staking-scripts/types/tokenfactory/voting_escrow",
      // ampgauges:
      //   "cd .. && json2ts -i contracts/amp-governance/amp_gauges/**/*.json -o ../liquid-staking-scripts/types/amp_gauges",
      // empgauges:
      //   "cd .. && json2ts -i contracts/amp-governance/emp_gauges/**/*.json -o ../liquid-staking-scripts/types/emp_gauges",
      // propgauges:
      //   "cd .. && json2ts -i contracts/amp-governance/prop_gauges/**/*.json -o ../liquid-staking-scripts/types/prop_gauges",

      farm: "cd .. && json2ts -i contracts/amp-compounder/astroport_farm/**/*.json -o ../liquid-staking-scripts/types/tokenfactory/amp-compounder/astroport_farm",
      // compound:
      //   "cd .. && json2ts -i contracts/amp-compounder/compound_proxy/**/*.json -o ../liquid-staking-scripts/types/amp-compounder/compound_proxy",
      // fees: "cd .. && json2ts -i contracts/amp-compounder/fees_collector/**/*.json -o ../liquid-staking-scripts/types/amp-compounder/fees_collector",
      generator:
        "cd .. && json2ts -i contracts/amp-compounder/generator_proxy/**/*.json -o ../liquid-staking-scripts/types/tokenfactory/amp-compounder/generator_proxy",
    },
  },
};
