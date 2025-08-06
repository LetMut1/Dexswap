To executes the commands it is needed to create and to launch the docker-container from `/$ROOT/cli/development_environment/tool/rust/docker/docker-compose.yaml`
<br>
<br>
<br>
To test the `swap` it is needed:<br>
1 - execute the sequence of commands described in `/$ROOT/program/application/readme.md`.<br>
2 - execute `1` command from the list below, replacing the value of the `--lamports_to_treasury` parameter and get the pubkey of `Intermediary` on the screen.<br>
3 - execute `4` command from the list below, substituting `Intermediary` pubkey into the `--intermediary` parameter and replacing the values of the `--amount_in`, `--min_amount_out` parameters.
<br>
<br>
<br>
Available commands:
<br>
`1` - To create Intermediary state:
```
cargo run --bin=client --features=intermediary_devnet --manifest-path=/intermediary/cli/application/Cargo.toml -- --solana_rpc_url=https://api.devnet.solana.com initialize --intermediary_manager=/intermediary/_keypairs/intermediary_manager.json  --intermediary_trader=/intermediary/_keypairs/intermediary_trader.json --lamports_to_treasury=?
```
`2` - To deposit funds on WSol token account:
```
cargo run --bin=client --features=intermediary_devnet --manifest-path=/intermediary/cli/application/Cargo.toml -- --solana_rpc_url=https://api.devnet.solana.com deposit_funds --intermediary=(pubkey) --intermediary_manager=/intermediary/_keypairs/intermediary_manager.json --lamports_to_treasury=?
```
`3` - To withdraw funds from WSol token account:
```
cargo run --bin=client --features=intermediary_devnet --manifest-path=/intermediary/cli/application/Cargo.toml -- --solana_rpc_url=https://api.devnet.solana.com withdraw_funds --intermediary=(pubkey) --intermediary_manager=/intermediary/_keypairs/intermediary_manager.json --lamports_from_treasury=?
```
`4` - To swap:
```
cargo run --bin=client --features=intermediary_devnet --manifest-path=/intermediary/cli/application/Cargo.toml -- --solana_rpc_url=https://api.devnet.solana.com swap --intermediary=(pubkey)  --intermediary_trader=/intermediary/_keypairs/intermediary_trader.json --amount_in=? --min_amount_out=?
```