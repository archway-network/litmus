#!/bin/bash

# Setup both chains
GM=/root/.gm/bin/gm
$GM start

sleep 5

# Setup Hermes clients
HERMES=./.hermes/bin/hermes
HERMES_CONFIG=./.hermes/config.toml
CHAIN1="archway-1"
CHAIN2="archway-2"

$HERMES --config $HERMES_CONFIG create client --host-chain $CHAIN1 --reference-chain $CHAIN2
$HERMES --config $HERMES_CONFIG create client --host-chain $CHAIN2 --reference-chain $CHAIN1

# Setup connection
$HERMES --config $HERMES_CONFIG create connection --a-chain $CHAIN1 --a-client 07-tendermint-0 --b-client 07-tendermint-0

# Setup channel-0
$HERMES --config $HERMES_CONFIG create channel --a-chain $CHAIN1 --a-connection connection-0 --a-port transfer --b-port transfer

# Run hermes
$HERMES --config $HERMES_CONFIG start >> $DIR/relayer.log

tail -f /dev/null