#!/bin/bash

docker_name=secretdev

function secretcli() {
  docker exec "$docker_name" secretcli "$@";
}

function wait_for_tx() {
  until (secretcli q tx "$1"); do
      sleep 5
  done
}

export SGX_MODE=SW

deployer_name=a

deployer_address=$(secretcli keys show -a $deployer_name)
echo "Deployer address: '$deployer_address'"

docker exec -it "$docker_name" secretcli tx compute store "/root/code/build/secret_oracle.wasm" --from a --gas 2000000 -b block -y
token_code_id=$(secretcli query compute list-code | jq '.[-1]."id"')
token_code_hash=$(secretcli query compute list-code | jq '.[-1]."data_hash"')
echo "Stored contract: '$token_code_id', '$token_code_hash'"

echo "Deploying contract..."
label=$(date +"%T")

export STORE_TX_HASH=$(
  secretcli tx compute instantiate $token_code_id '{}' --from $deployer_name --gas 1500000 --label $label -b block -y |
  jq -r .txhash
)
wait_for_tx "$STORE_TX_HASH" "Waiting for instantiate to finish on-chain..."

contract_address=$(docker exec -it $docker_name secretcli query compute list-contract-by-code $token_code_id | jq '.[-1].address')
echo "contract address: '$contract_address'"

echo $(secretcli q staking validators)

# should work
secretcli tx compute execute --label $label '{"register": {"validator_key": "abc"}}' --from $deployer_name -y --gas 1500000 -b block

echo "This will fail.."
# should not work
secretcli tx compute execute --label $label '{"register": {"validator_key": "abc"}}' --from b -y --gas 1500000 -b block

# should work
secretcli tx compute execute --label $label '{"predict_prices": {"prices": [{"symbol": "BTC", "price": "100000000000000000000"}], "validator_key": "abc"}}' --from $deployer_name -y --gas 1500000 -b block

# should work
secretcli tx compute execute --label $label '{"predict_prices": {"prices": [{"symbol": "BTC", "price": "1"}], "validator_key": "abc"}}' --from $deployer_name -y --gas 1500000 -b block


# should not work
secretcli tx compute execute --label $label '{"predict_prices": {"prices": [{"symbol": "BTC", "price": "100000000000000000000"}], "validator_key": "asdasdasdasd"}}' --from b -y --gas 1500000 -b block

secretcli q compute query $(echo "$contract_address" | tr -d '"') '{"get_validators": {}}'

secretcli q compute query $(echo "$contract_address" | tr -d '"') '{"get_symbols": {}}'

secretcli q compute query $(echo "$contract_address" | tr -d '"') '{"get_price": {"symbols": ["BTC"]}}'