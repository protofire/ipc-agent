# IPC Quick Start: zero-to-subnet

>💡 Background and detailed are available in the [README](/README.md).

Ready to test the waters with your first subnet? This guide will deploy a subnet with multiple local validators orchestrated by the same IPC agent. This subnet will be anchored to the public Spacenet. This will be a minimal example and may not work on all systems. The full documentation provides more details on each step.

Several steps in this guide involve running long-lived processes. In each of these cases, the guide advises starting a new *session*. Depending on your set-up, you may do this using tools like `screen` or `tmux`, or, if using a graphical environment, by opening a new terminal tab, pane, or window.

>💡A video walkthrough of this guide is also [available](https://www.youtube.com/watch?v=J9Y4_bzGue4). We still encourage you to try it for yourself!

## Step 0: Prepare your system

We assume a Ubuntu Linux instance when discussing prerequisites, but annotate steps with system-specificity and links to detailed multi-OS instructions. Exact procedures will vary for other systems, so please follow the links if running something different. Details on IPC-specific requirements can also be found in the [README](/README.md).

* Install basic dependencies [Ubuntu/Debian] ([details](https://lotus.filecoin.io/lotus/install/prerequisites/#supported-platforms))
```bash
sudo apt update && sudo apt install build-essential libssl-dev mesa-opencl-icd ocl-icd-opencl-dev gcc git bzr jq pkg-config curl clang hwloc libhwloc-dev wget ca-certificates gnupg -y 
```

* Install Rust [Linux] ([details](https://www.rust-lang.org/tools/install))
```bash
curl https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"
rustup target add wasm32-unknown-unknown
```

* Install Go [Linux] ([details](https://go.dev/doc/install))
```bash
curl -fsSL https://golang.org/dl/go1.19.7.linux-amd64.tar.gz | sudo tar -xz -C /usr/local
echo 'export PATH=$PATH:/usr/local/go/bin' >> ~/.bashrc && source ~/.bashrc
```

* Install Docker Engine [Ubuntu] ([details](https://docs.docker.com/engine/install/))
```bash
sudo install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
sudo chmod a+r /etc/apt/keyrings/docker.gpg
echo \
  "deb [arch="$(dpkg --print-architecture)" signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
  "$(. /etc/os-release && echo "$VERSION_CODENAME")" stable" | \
  sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
sudo apt-get update && sudo apt-get install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin -y
sudo usermod -aG docker $USER && newgrp docker
```


## Step 1: Build the IPC stack

Next, we'll download and build the different components (IPC agent, docker images, and eudico).

* Pick a folder where to build the IPC stack. In this example, we'll go with `~/ipc/`.
```bash
mkdir -p ~/ipc/ && cd ~/ipc/ 
```
* Download and compile the IPC Agent (might take a while)
```bash
git clone https://github.com/consensus-shipyard/ipc-agent.git
(cd ipc-agent && make build && make install-infra)
```
* Download and compile eudico (might take a while)
```bash
git clone https://github.com/consensus-shipyard/lotus.git
(cd lotus && make spacenet)
```


## Step 2: Deploy a Spacenet node

Let's deploy a eudico instance on Spacenet and configure the IPC Agent to interact with it.

* [**In a new session**] Start your eudico instance (might take a while to sync the chain)
```bash
./lotus/eudico mir daemon --bootstrap
```
* Get configuration parameters
```bash
./lotus/eudico auth create-token --perm admin
./lotus/eudico wallet new
```
* Configure your IPC Agent
```bash
./ipc-agent/bin/ipc-agent config init
nano ~/.ipc-agent/config.toml
```
* Replace the content of `config.toml` with the text below, substituting the token and wallet retrieved above.
```toml
[server]
json_rpc_address = "0.0.0.0:3030"

[[subnets]]
id = "/root"
gateway_addr = "t064"
network_name = "root"
jsonrpc_api_http = "http://127.0.0.1:1234/rpc/v1"
auth_token = "<AUTH_TOKEN_0>"
accounts = ["<WALLET_0>"]
```
* [**In a new session**] Start your IPC Agent
```bash
./ipc-agent/bin/ipc-agent daemon
```


## Step 3: Fund your account

* Obtain some Spacenet FIL by requesting it from the [faucet](https://faucet.spacenet.ipc.space/), using your wallet address. 


## Step 4: Create the subnet

* The next step is to create a subnet under `/root`
```bash
./ipc-agent/bin/ipc-agent subnet create --parent /root --name andromeda --min-validator-stake 1 --min-validators 2 --bottomup-check-period 30 --topdown-check-period 30
```
* Make a note of the address of the subnet you created (`/root/<SUBNET_ID>`)


## Step 5: Create and export validator wallets

Although we set a minimum of 2 active validators in the previous, we'll deploy 3 validators to add some redundancy. 

* First, we'll need to create a wallet for each validator
```bash
./ipc-agent/bin/ipc-agent wallet new --key-type secp256k1 --subnet /root
./ipc-agent/bin/ipc-agent wallet new --key-type secp256k1 --subnet /root
./ipc-agent/bin/ipc-agent wallet new --key-type secp256k1 --subnet /root
```
* Export each wallet (WALLET_1, WALLET_2, and WALLET_3) by substituting their addresses below
```bash
./lotus/eudico wallet export --lotus-json <WALLET_1> > ~/.ipc-agent/wallet1.key
./lotus/eudico wallet export --lotus-json <WALLET_2> > ~/.ipc-agent/wallet2.key
./lotus/eudico wallet export --lotus-json <WALLET_3> > ~/.ipc-agent/wallet3.key
```
* We also need to fund the wallets with enough collateral to; we'll send the funds from our default wallet 
```bash
./ipc-agent/bin/ipc-agent subnet send-value --subnet /root --to <WALLET_1> 2
./ipc-agent/bin/ipc-agent subnet send-value --subnet /root --to <WALLET_2> 2
./ipc-agent/bin/ipc-agent subnet send-value --subnet /root --to <WALLET_3> 2
```


## Step 6: Deploy the infrastructure

We can deploy the subnet nodes. Note that each node should be importing a different wallet key for their validator, and should be exposing different ports. If these ports are unavailable in your system, please pick different ones.

* Deploy and run a container for each validator, importing the corresponding wallet keys
```bash
./ipc-agent/bin/ipc-infra/run-subnet-docker.sh 1251 1351 /root/<SUBNET_ID> ~/.ipc-agent/wallet1.key
./ipc-agent/bin/ipc-infra/run-subnet-docker.sh 1252 1352 /root/<SUBNET_ID> ~/.ipc-agent/wallet2.key
./ipc-agent/bin/ipc-infra/run-subnet-docker.sh 1253 1353 /root/<SUBNET_ID> ~/.ipc-agent/wallet3.key
```
* If the deployment is successful, each of these nodes should return the following output at the end of their logs. Save the information for the next step.
```
>>> Subnet /root/<SUBNET_ID> daemon running in container: <CONTAINER_ID_#> (friendly name: <CONTAINER_NAME_#>)
>>> Token to /root/<SUBNET_ID> daemon: <AUTH_TOKEN_#>
>>> Default wallet: <WALLET_#>
>>> Subnet subnet validator info:
<VALIDATOR_ADDR_#>
>>> API listening in host port <PORT_#>
>>> Validator listening in host port <VALIDATOR_PORT_#>
```


## Step 7: Configure the IPC agent

For ease of use, we'll import the remaining keys into the first validator, via which the IPC Agent will act on behalf of all.

* Copy the wallet keys into the docker container and import them
```bash
docker cp ~/.ipc-agent/wallet2.key <CONTAINER_NAME_1>:/input.key && docker exec -it <CONTAINER_NAME_1> eudico wallet import --format=json-lotus input.key
docker cp ~/.ipc-agent/wallet3.key <CONTAINER_NAME_1>:/input.key && docker exec -it <CONTAINER_NAME_1> eudico wallet import --format=json-lotus input.key
```
* Edit the IPC agent configuration `config.toml`
```bash
nano ~/.ipc-agent/config.toml
```
* Append the new subnet to the configuration
```toml
[[subnets]]
id = "/root/<SUBNET_ID>"
gateway_addr = "t064"
network_name = "andromeda"
jsonrpc_api_http = "http://127.0.0.1:1251/rpc/v1"
auth_token = "<AUTH_TOKEN_1>"
accounts = ["<WALLET_1>", "<WALLET_2>", "<WALLET_3>"]
```
* Reload the config
```bash 
./ipc-agent/bin/ipc-agent config reload
```


## Step 8: Join the subnet 

All the infrastructure for the subnet is now deployed, and we can join our validators to the subnet. For this, we need to send a `join` command from each of our validators from their validator wallet addresses providing the validators multiaddress. 

* Join the subnet with each validator
```bash
./ipc-agent/bin/ipc-agent subnet join --from <WALLET_1> --subnet /root/<SUBNET_ID> --collateral 1 --validator-net-addr <VALIDATOR_ADDR_1>
./ipc-agent/bin/ipc-agent subnet join --from <WALLET_2> --subnet /root/<SUBNET_ID> --collateral 1 --validator-net-addr <VALIDATOR_ADDR_2>
./ipc-agent/bin/ipc-agent subnet join --from <WALLET_3> --subnet /root/<SUBNET_ID> --collateral 1 --validator-net-addr <VALIDATOR_ADDR_3>
```


## Step 9: Start validating! 

We have everything in place now to start validating. Run the following script for each of the validators [**each in a new session**], passing the container names:
```bash
./ipc-agent/bin/ipc-infra/mine-subnet.sh <CONTAINER_NAME_1> 
./ipc-agent/bin/ipc-infra/mine-subnet.sh <CONTAINER_NAME_2> 
./ipc-agent/bin/ipc-infra/mine-subnet.sh <CONTAINER_NAME_3> 
```


## Step 10: What now?

* Check that the subnet is running
```bash
./ipc-agent/bin/ipc-agent subnet list --gateway-address t064 --subnet /root
```
* If something went wrong, please have a look at the [README](https://github.com/consensus-shipyard/ipc-agent). If it doesn't help, please join us in #ipc-help. In either case, let us know your experience!
* Please note that to repeat this guide or spawn a new subnet, you may need to change the parameters or reset your system.
