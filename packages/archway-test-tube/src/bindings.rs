use test_tube::bindings::{GoString, GoUint64};
use std::os::raw::c_char;

// These are patches bindings that work with the new ABCI++ block design
extern "C" {
    pub fn Execute(envId: GoUint64, base64ReqDeliverTx: GoString);
}
extern "C" {
    pub fn EndBlock(envId: GoUint64) -> *mut c_char;
}