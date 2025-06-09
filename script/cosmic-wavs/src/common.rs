use cosmwasm_std::StdError;

pub fn append_0x(content: &str) -> String {
    let mut initializer = String::from("0x");
    initializer.push_str(content);
    initializer
}

pub fn parse_u64_attribute(attributes: &Vec<(String, String)>, key: &str) -> anyhow::Result<u64> {
    attributes
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.parse::<u64>())
        .ok_or(anyhow::anyhow!(format!("Missing {} attribute", key)))?
        .map_err(|_| anyhow::anyhow!(format!("Failed to parse {} attribute to u64", key)))
}

pub fn parse_string_attribute(attributes: &Vec<(String, String)>, key: &str) -> anyhow::Result<String> {
    attributes
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
        .ok_or(anyhow::anyhow!("Missing sender attribute"))
}

///  handle  response from  broadcasting tx
pub fn handle_tx_response(code: u32, raw_log: &str) -> Result<(), StdError> {
    match code {
        0 => Ok(()),
        code => Err(StdError::generic_err(format!(
            "Transaction failed with code {}: {}",
            code, raw_log
        ))),
    }
}
