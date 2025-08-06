To create main Intermediary state:
```
cargo run --bin=client --features=intermediary_devnet -- --solana_rpc_url=https://api.devnet.solana.com initialize --intermediary_manager=/intermediary/cli/application/_keypairs/intermediary_manager.json  --intermediary_trader=/intermediary/cli/application/_keypairs/intermediary_trader.json --lamports_to_treasury=?
```
To deposit funds on WSol token account:
```
cargo run --bin=client --features=intermediary_devnet -- --solana_rpc_url=https://api.devnet.solana.com deposit_funds --intermediary=(pubkey) --intermediary_manager=/intermediary/cli/application/_keypairs/intermediary_manager.json --lamports_to_treasury=?
```
To withdraw funds from WSol token account:
```
cargo run --bin=client --features=intermediary_devnet -- --solana_rpc_url=https://api.devnet.solana.com withdraw_funds --intermediary=(pubkey) --intermediary_manager=/intermediary/cli/application/_keypairs/intermediary_manager.json --lamports_from_treasury=?
```
To swap:
cargo run --bin=client --features=intermediary_devnet -- --solana_rpc_url=https://api.devnet.solana.com swap --intermediary=(pubkey)  --intermediary_trader=/intermediary/cli/application/_keypairs/intermediary_trader.json --amount_in=? --min_amount_out=?