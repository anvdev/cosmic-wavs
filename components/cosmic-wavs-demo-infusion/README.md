# Cosmic-WAVS

<div align="center">

[![Bannger](/imgs/readme-banner.png)](https://youtu.be/jyl7kbie41w)

</div>


## Goals
- Track cosmwasm nft burn events emitted from Cosmos Chain
- Authentication action to perform via Wavs operator keys
- Broadcast authorized action to Cosmos Chain

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
 // 2.query contract the check if operators need to update assigned cw-infuser state
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


## Workflow (using cw-orch)
We need to configure our operator to listen for events coming from a cosmos node endpoints, we need to define the endpoints to listen to, or deploy a local node. 

First, we make use of an env variable to specify what the trigger origin is
```
TRIGGER_ORIGIN=cosmos
```

Then we deploy a local cosmos node if testing locally. Otherwise, we connect our client to an endpoint on the netowrk configured.
This step is managed during the `start_all_local` command. 

### St
