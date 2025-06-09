// 1. x/smart-account
// register/unregister account as smart authenticator
// form custom msg for smart account to perform

pub mod common;
pub mod smart_accounts;
pub mod wavs;
pub mod zktls;

// 2. bls12-381 agg sig helper
// create aggregated signature set
// aggregate & verify aggregate signature
// get current signatures
// add to current signatures
// update signature
