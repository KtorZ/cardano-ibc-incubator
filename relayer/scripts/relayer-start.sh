#!/bin/sh
set -x

# Check if the .env file exists and export environment variables
if [ -f ".env" ]; then
  echo "Found .env file. Exporting environment variables..."
  set -a 
  source .env
  set +a 
else
  echo ".env file does not exist."
fi

path=${RELAYER_PATH_NAME}
configs_dir=".config/${CONFIGS_DIR}"

src_chain_name=${SRC_CHAIN_NAME}
src_port=${SRC_PORT}
src_chain_config_path="${configs_dir}/chains/${src_chain_name}.json"
src_mnemonic="${SRC_MNEMONIC}"

echo "Source Chain vals: " $src_chain_name $src_port $src_mnemonic

dst_chain_name=${DST_CHAIN_NAME}
dst_port=${DST_PORT}
dst_chain_config_path="${configs_dir}/chains/${dst_chain_name}.json"
dst_mnemonic="${DST_MNEMONIC}"

echo "DST chain vals: " $dst_chain_name $dst_port $dst_mnemonic

alias rly=./bin/rly

src_rpc_addr=${SRC_RPC_ADDR}
dst_rpc_addr=${DST_RPC_ADDR}
mithril_endpoint=${MITHRIL_ENDPOINT}

# Update the rpc-addr for the source chain if the SRC_RPC_ADDR variable is defined
if [ ! -z "$src_rpc_addr" ]; then
  if [ -f "$src_chain_config_path" ]; then
    cat $src_chain_config_path | jq --arg rpc_addr "$src_rpc_addr" '.value."rpc-addr" = $rpc_addr' > temp.json && mv temp.json "$src_chain_config_path"
    echo "Updated src_rpc_addr in $src_chain_config_path"
  else
    echo "Configuration file $src_chain_config_path does not exist."
  fi
else
  echo "Environment variable src_rpc_addr is not defined."
fi

# Update the rpc-addr for the destination chain if the DST_RPC_ADDR variable is defined
if [ ! -z "$dst_rpc_addr" ]; then
  if [ -f "$dst_chain_config_path" ]; then
    cat $dst_chain_config_path | jq --arg rpc_addr "$dst_rpc_addr" '.value."rpc-addr" = $rpc_addr' > temp.json && mv temp.json "$dst_chain_config_path"
    echo "Updated dst_rpc_addr in $dst_chain_config_path"
  else
    echo "Configuration file $dst_chain_config_path does not exist."
  fi
else
  echo "Environment variable dst_rpc_addr is not defined."
fi

# Update the mithril endpoint for the source chain if the MITHRIL_ENDPOINT variable is defined
if [ ! -z "$mithril_endpoint" ]; then
  if [ -f "$src_chain_config_path" ]; then
    cat $src_chain_config_path | jq --arg mithril_endpoint "$mithril_endpoint" '.value."mithril-endpoint" = $mithril_endpoint' > temp.json && mv temp.json "$src_chain_config_path"
    echo "Updated mithril_endpoint in $src_chain_config_path"
  else
    echo "Configuration file $src_chain_config_path does not exist."
  fi
else
  echo "Environment variable mithril_endpoint is not defined."
fi

# Initialize configuration and keys
rm -rf ~/.relayer/*
rly config init
rly chains add-dir $configs_dir/chains
rly paths add-dir $configs_dir/paths

# Setup keys for the source chain
rly keys restore $src_chain_name faucet-key "$src_mnemonic"
rly keys use $src_chain_name faucet-key

# Setup keys for the destination chain
rly keys restore $dst_chain_name faucet-key "$dst_mnemonic"
rly keys use $dst_chain_name faucet-key

# Establish connection
rly transact connection $path --block-history 5 --client-tp 36h

# Establish channel
rly transact channel $path --src-port $src_port --dst-port $dst_port --order unordered --version ics20-1

# Start the relayer
rly start $path

