import { SigningCosmWasmClient, Secp256k1HdWallet, GasPrice, Coin } from "cosmwasm";

import * as fs from 'fs';
import axios from 'axios';
import { ClientRequest } from "http";
import { WritableStreamDefaultWriter } from "node:stream/web";

const rpcEndpoint = "https://juno-testnet-rpc.polkachu.com";

const contract_wasm = fs.readFileSync("../artifacts/cargo_generated_contract.wasm");

const mnemonic =
    "october expect minute knee virus satisfy thing hand speak spot galaxy favorite";

const contract_code_id = 3048;

const contract_address = "juno1lk7exetcvxx86jm67frtn22kxqh5tad8sh0fqgr32sj9fz7r7wwsyuyl93";

async function setupClient(mnemonic: string, rpc: string, gas: string | undefined): Promise<SigningCosmWasmClient> {
    if (gas === undefined) {
        let wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic, { prefix: 'juno'});
        let client = await SigningCosmWasmClient.connectWithSigner(rpc, wallet);
        return client;
    } else {
        let gas_price = GasPrice.fromString(gas);
        let wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic, { prefix: 'juno' });
        let client = await SigningCosmWasmClient.connectWithSigner(rpc, wallet, { gasPrice: gas_price });
        return client;
    }
}

async function getAddress(mnemonic: string, prefix: string = 'juno') {
    let wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic, { prefix });
    let accounts = await wallet.getAccounts();
    return accounts[0].address;
}

describe("Messages Fullstack Test", () => {
    xit("Generate Wallet", async () => {
        let wallet = await Secp256k1HdWallet.generate(12);
        console.log(wallet.mnemonic);
    });

    xit("Get Testnet Tokens", async () => {
        //let wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic, { prefix: 'juno' });
        //console.log(await wallet.getAccounts());
        console.log(await getAddress(mnemonic));
        try {
            let res = await axios.post("https://faucet.uni.juno.deuslabs.fi/credit", { "denom": "ujunox", "address": await getAddress(mnemonic) });
            console.log(res);
        } catch (e) {
            console.log(e);
        }
    }).timeout(100000);

    xit("Balance Testnet Tokens", async () => {
        let client = await setupClient(mnemonic, rpcEndpoint, "0.025ujunox");
        let searchDenom: string = 'ujunox';
        let res = await client.getBalance(await getAddress(mnemonic), "ujunox");
        console.log(res);        
    }).timeout(100000);

    xit("Send Testnet Tokens", async () => {
        let client = await setupClient(mnemonic, rpcEndpoint, "0.025ujunox");
        let receiver = "";
        let res = await client.sendTokens(await getAddress(mnemonic), receiver, [{denom:"ujunox", amount:"1000000"}], "auto");
        console.log(res);
    }).timeout(100000);

    //same as
    //junod tx wasm store artifacts/manager.wasm --from wallet --node https://rpc.uni.juno.deuslabs.fi --chain_id=uni-3 --gas-price=0.025ujunox --gas auto
    xit("Upload manager code to testnet", async () => {
        let client = await setupClient(mnemonic, rpcEndpoint, "0.025ujunox");
        let res = await client.upload(await getAddress(mnemonic), contract_wasm, "auto");
        //calculateFee()
        console.log(JSON.stringify(res.logs[0].events));
    }).timeout(100000);

    xit("Instantiate manager code on testnet", async () => {
        let client = await setupClient(mnemonic, rpcEndpoint, "0.025ujunox");
        let res = await client.instantiate(await getAddress(mnemonic), 
                contract_code_id, { owner: "wladzioo" },
                "messages", "auto");
        console.log(res);
    }).timeout(100000);
    // manager contract addrr: juno1mcl0m3f33g2u250xr9t4kshfjw6ccs34z6pxwqulkmzdc2czwd0s83a09y
    
    xit("New entry", async() => {
        let client = await setupClient(mnemonic, rpcEndpoint, "0.025ujunox");
        let res = await client.execute(await getAddress(mnemonic), 
        contract_address, {  new_entry: { description: "First entry", priority: "low"}},
        "auto", "", 
        [{amount: "10000", denom: "ujunox"}]);
        console.log(res);
    }).timeout(20000);


    xit("Query entry by ID", async () => {
        let client = await setupClient(mnemonic, rpcEndpoint, "0.025ujunox");
        let res = await client.queryContractSmart(contract_address, { query_entry: {id: 1}});
        console.log(res);

        // for (let i = 0; i<res['contracts'].length; i++) {
        //     console.log("------------CONTRACTS[%s]-----------------", i);
        //     console.log(res['contracts'][i]);          
        // }
    }).timeout(20000);

    xit("Second entry", async() => {
        let client = await setupClient(mnemonic, rpcEndpoint, "0.025ujunox");
        let res = await client.execute(await getAddress(mnemonic), 
        contract_address, {  new_entry: { description: "Second entry", priority: "high"}},
        "auto", "", 
        [{amount: "10000", denom: "ujunox"}]);
        console.log(res);
    }).timeout(20000);

    xit("Update second entry", async() => {
        let client = await setupClient(mnemonic, rpcEndpoint, "0.025ujunox");
        let res = await client.execute(await getAddress(mnemonic), 
        contract_address, {  update_entry: { 
            id: 2, 
            description: "Second entry - finished", 
            status: "done"}},
        "auto", "", 
        [{amount: "10000", denom: "ujunox"}]);
        console.log(res);
    }).timeout(20000);

    xit("Delete entry", async() => {
        let client = await setupClient(mnemonic, rpcEndpoint, "0.025ujunox");
        let res = await client.execute(await getAddress(mnemonic), 
        contract_address, {  delete_entry: { 
            id: 1}},
        "auto", "", 
        [{amount: "10000", denom: "ujunox"}]);
        console.log(res);
    }).timeout(20000);

    it("Query all entries", async () => {
        let client = await setupClient(mnemonic, rpcEndpoint, "0.025ujunox");
        let res = await client.queryContractSmart(contract_address, { query_list: {start_after: 0}});
        console.log(res);

        // for (let i = 0; i<res['entries'].length; i++) {
        //     console.log("------------ENTRIES[%s]-----------------", i);
        //     console.log(res['entries'][i]);          
        // }
    }).timeout(20000);
});