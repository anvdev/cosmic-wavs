# Cosmic-WAVS

<div align="center">

[![Bannger](/imgs/readme-banner.png)](https://youtu.be/jyl7kbie41w)

</div>

## TODO:
- **Registeer + Updating Keys For Operator Sets**
- **Allowing New Operator Seervices To Join: Permissionless Servicec**
- **Wire In Fee Distribution/Claiming/Slashing Mecchanisms for operators & consumers**
-**Round Robin + Bls Verification:** Prior to submitting the ipfs result, we can perform the workflow to determine who the msg broadcaster is for a given set of bls operator keys, as well as broadcast the msg, and at minimum include the tx hash that has been deployed,share back to the rest of the operator services.

## Goals
<!-- - Track cosmwasm nft burn events emitted from Cosmos Chain -->
- Authentication action to be performed by a single on-chain acccount, via Aggregated avs operator key consensus
- Broadcast authorized action to a cosmos chain via programmable methods

## Design 

### wavs + x/smart-accounts  
 For this implemenetation, WAVS services will use authentication capabilities provided by the [x/smart-account module](https://github.com/permissionlessweb/go-bitsong/tree/main/x/smart-account) to perform on chain actions. This is implemented by 
registration of an smart-contract authenticator to a secp256k1 key account. This repo contains a few example of making use of this workflow. Our bls12-381 compatible account authentication example can be found here [btsg-wavs](https://github.com/permissionlessweb/bs-accounts/blob/cleanup/contracts/smart-accounts/btsg-wavs/src/contract.rs#L100), and is used to allow a set of operator for a given AVS instance authenticate actions for this account to perform.


### custom AVS logic
 
Here we design our AVS to perform custom logic. This demo has logic that filters any new burn event that has occured on the chain the cw-infusion contract is deployed on, in order to trigger its custom filtering workflow:
```rs
TriggerData::CosmosContractEvent(TriggerDataCosmosContractEvent {event,..}) => {
            // Extract event type and data from Cosmos event
            let event_type = Some(event.ty.clone());
            if let Some(et) = event_type.as_ref() {
                if et.as_str() == "wasm" {
                    // Look for burn action
                    if let Some(action_attr) = event.attributes.iter().find(|(k, _)| k == "action")
                    {
                               if action_attr.1 == "burn" {
                                /// custom logic...
                               }
                    }
                }
            }}
```

We can also implement custom logic, such as deterministic queries to determine any msgs that the AVS should perform:
```rs
 // 2. query a  smart contract with a query clieent to check if operators need to update assigned cw-infuser state
    let res: Vec<cw_infusions::wavs::WavsRecordResponse> = cosm_guery
        .contract_smart(
            &Address::new_cosmos_string(&cw_infuser_addr, None)?,
            &cw_infuser::msg::QueryMsg::WavsRecord {
                nfts: vec![nft_addr.to_string()],
                burner: None,
            },
        )
        .await?;

    // 3. form msgs for operators to sign
    let mut infusions = vec![];
    for record in res {
        if let Some(count) = record.count {
            // implement custom WAVS action here
        }
    }

```
For this demo, any burn event will trigger the AVS to check if any infusion in the cw-infuser address paired to it has the specific nft collection as an eligible collection.

If there are none,no messages are formed, otherwise a message to update the global contract state is signed via the preferred Ecdsa authorization method.
```rs
// - create sha256sum bytes that are being signed by operators for aggregated approval.
// Current implementation signs binary formaated array of Any msgs being authorized.
// let namespace = Some(&b"demo"[..]);
let signature = imported_signer
.sign(
    None,
    &Sha256::digest(to_json_binary(&cosmic_wavs_actions)?.as_ref())
        .to_vec()
        .try_into()
        .unwrap(),
)
.to_vec();

```

We still need to handle error responses, in order to resubmit transactions via governance override.
We still need to implement aggregated consensus if there are more than one operator.

# Implement Aggregated Consensus Workflow
 
## Goals
### Consensus Layer Between Operators: 
*Generate single aggregated bls12-381 signature & pubkey between all operators for a given service.*\
*Allow new operators to join into the aggregated set.*\
Each operator must register their bls12-381 public key, which is shared with every other operator service. This is how msgs are authenticated between operators, and also is the aggregated set  that is used in the x/smart-account workflow to authorize on chain actions from an account registered with a custom authenticator.  

### Round Robin + Action Digest Validation
*Choose aggregator for each block, enable logic for failed aggregation*\
We need to have a single entity from the operator broadcast to the cosmos chain RPC the agreed upon actions. We can provide a few options to accopany users trust assumptions:
- aggregator: single entity set to always validate aggregated consensus before broadcasting. Censorship risk with broadcasting transactions.
- round-robin: randomly choose first entity, and then procceed to randomly determine the next operator to participate also participaate as the aggregator for a given round.
- stake-weighted: randomly choose entity, withweights based on restake power.

### Key Rotation
*Allow existing operators to rotate their public key used in consensus.*\

#### Referee Service
*Return deterministic response for each operator to post to ipfs*
To prevent aggregator entities censoring actions that were agreed to perform, we can implement a referee service that will apply a penalty to the operator that did not broadcast the msg into the cosmos chain as expected. This is implemented via:
- retrieving expected operator pubkey set
- trigger upon each time the cosmic-wavs demo is run 
- expect to recieve info any time msg to be performed by service
- expect wavs managed account to have included tx in mempool (we would not have broadcast if tx was to error)
- apply penalty to operator that did not participate


### Errors On Tx Broadcast 
Each node can simulate the tx prior to signing and performing the aggregated consensus workflow, and if at least more than 2/3 do not expect the message to panic when broadcasted, we assume the message will be successful. If assigned aggregator runs into panic upon actually broadcasting the message, a function where we recursively attempt to have the tx broadcasted succesfully can be implemented where if 2/3 of the attempts fail, we post the failed details to ipfs for that action. We could decide on if to halt the avs service upon failed tx, or continue with listening for trigger events. 

## Implementation 1: Decentralized Aggreagtion
### 1. asyncronous state machine via alto
### 2. seperate avs libarary for referee
### 3. integation into existing config

## Implementation 2: Centralized Aggreagtion
### 1. Customization of expected authentication object for a given



