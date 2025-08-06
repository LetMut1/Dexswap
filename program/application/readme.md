To fulfill the technical task, it was not necessary to create an entity storing additional control data, and it was enough to implement the swap(...) method, but to demonstrate an understanding of the basics of building contracts on Solana, I increased the amount of code, and the Intermediary entity was introduced.

In order for our contract to be able to determine the availability of the Pool, it was necessary to repeat the implementation of the Pool availability logic implemented on the Dex contract side.
Accordingly, the task was to implement the code from the Dex crates, but, unfortunately, direct connection of the crates led to multiple dependency errors. (To be precise, it was possible to directly connect a set of crates for only 1 of any Dex, but for this it was necessary to downgrade the SolanaProgram version.
It was also possible to make Git-forks of the Dex crates and increase the versions of the crates used inside to ensure further compatibility during implementation.
I chose the option of copying the necessary code from the Dex crates (of course, observing the layout of the structures and their serialization format)
<br>
<br>
<br>
<br>
<br>

To executes the commands it is needed to create and to launch the docker-container from `/$ROOT/program/development_environment/tool/rust/docker/docker-compose.yaml`
<br>
<br>
To deploy contract to Devnet a sequence of commands must be executed:<br>
```
solana config set --url https://api.devnet.solana.com
```
```
cargo build-sbf --features=devnet --manifest-path=/intermediary/program/application/Cargo.toml
```
```
solana program deploy /intermediary/program/application/target/deploy/intermediary.so --program-id=/intermediary/_keypairs/devnet_program_id_keypair.json --keypair=/intermediary/_keypairs/intermediary_manager.json<br>
```